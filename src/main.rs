use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use rspotify::AuthCodePkceSpotify;
use std::{io, time::Instant};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

mod app;
mod cache;
mod config;
mod navigation;
mod services;
mod ui;

use app::{
    app::App,
    events::AppEvent,
    state::{ExplorerNode, KeyMode},
};
use config::settings::Settings;
use services::{auth, spotify as svc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log = std::fs::File::create("/tmp/spot-tty.log")?;
    tracing_subscriber::fmt()
        .with_writer(log)
        .with_ansi(false)
        .init();

    let settings = Settings::load()?;
    let spotify = auth::authenticate(
        &settings.client_id,
        &settings.client_secret,
        &settings.redirect_uri,
    )
    .await?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();
    let mut app = App::new();
    spawn_initial_fetches(spotify.clone(), tx.clone());

    let tick_rate = Duration::from_millis(150);
    let mut last_tick = Instant::now();
    let mut last_node: Option<ExplorerNode> = None;
    let mut fetch_in_progress = false;

    loop {
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            app.state.playback_progress += 0.01;
            if app.state.playback_progress > 1.0 {
                app.state.playback_progress = 0.0;
            }
            app.state.visualizer_phase = (app.state.visualizer_phase + 1) % 1000;
        }

        // ── Lazy cover fetching ───────────────────────────────────────────────
        // Each frame: collect URLs visible on screen right now, fetch only what's
        // missing and not already in-flight. This limits concurrent fetches and
        // avoids hammering 200 URLs at startup.
        {
            let size = terminal.size().unwrap_or_default();
            let areas = ui::layout::split(size);

            // Sidebar: visible playlist image URLs
            let sidebar_urls: Vec<String> = {
                let sel = app.state.navigation.selected_index;
                let h = areas.sidebar.height.saturating_sub(8); // minus borders/header/liked
                let vis = (h / 4) as usize; // COVER_H = 4
                let scroll = sel.saturating_sub(vis.saturating_sub(1));
                app.state
                    .playlists
                    .iter()
                    .skip(scroll)
                    .take(vis + 2)
                    .filter_map(|p| p.image_url.clone())
                    .collect()
            };

            // Explorer: visible track URLs
            let explorer_urls = ui::explorer::visible_cover_urls(&app.state, areas.main);

            // Merge: selected track first (highest priority), then visible rows
            let mut all_urls: Vec<String> = explorer_urls;
            for u in sidebar_urls {
                if !all_urls.contains(&u) {
                    all_urls.push(u);
                }
            }

            for url in all_urls {
                if !app.state.cover_cache.contains_key(&url)
                    && !app.state.cover_fetching.contains(&url)
                {
                    app.state.cover_fetching.insert(url.clone());
                    let tx2 = tx.clone();
                    tokio::spawn(async move {
                        if let Some(img) = ui::cover::fetch_cover(&url).await {
                            let _ = tx2.send(AppEvent::CoverLoaded(url, img));
                        }
                    });
                }
            }
        }

        // ── Render ────────────────────────────────────────────────────────────
        app.state.render_cache.begin_frame();
        let cache_ptr = &mut app.state.render_cache as *mut _;
        terminal.draw(|f| {
            // SAFETY: render_cache not aliased; AppState fields are read-only here.
            let cache = unsafe { &mut *cache_ptr };
            let areas = ui::layout::split(f.size());
            ui::sidebar::render(f, areas.sidebar, &app.state, cache);
            ui::explorer::render(f, areas.main, &app.state, cache);
            ui::player::render(f, areas.control, &app.state);
        })?;
        // One stdout write for all queued Kitty/iTerm2 sequences
        app.state.render_cache.flush();

        // ── Events ────────────────────────────────────────────────────────────
        while let Ok(ev) = rx.try_recv() {
            match &ev {
                // On track load: do NOT bulk-spawn covers — lazy loop above handles it
                AppEvent::ExplorerTracksLoaded(_) => {
                    fetch_in_progress = false;
                }
                AppEvent::LoadError(_) => {
                    fetch_in_progress = false;
                }
                _ => {}
            }
            app.handle_event(ev);
        }

        if app.state.explorer_fetch_pending && !fetch_in_progress {
            last_node = None;
        }
        maybe_fetch_explorer(
            &app.state.explorer_stack.last().cloned(),
            &mut last_node,
            &mut fetch_in_progress,
            spotify.clone(),
            tx.clone(),
        );

        // ── Input ─────────────────────────────────────────────────────────────
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char(c) = key.code {
                    if c.is_ascii_digit() {
                        let d = c.to_digit(10).unwrap() as usize;
                        app.state.pending_count =
                            Some(app.state.pending_count.unwrap_or(0) * 10 + d);
                        continue;
                    }
                }
                let count = app.state.pending_count.take().unwrap_or(1);
                match app.state.key_mode {
                    KeyMode::Normal => match key.code {
                        KeyCode::Char('j') | KeyCode::Down => tx.send(AppEvent::MoveDown(count))?,
                        KeyCode::Char('k') | KeyCode::Up => tx.send(AppEvent::MoveUp(count))?,
                        KeyCode::Char('G') => tx.send(AppEvent::GoBottom)?,
                        KeyCode::Char('M') => tx.send(AppEvent::GoMiddle)?,
                        KeyCode::Char('g') => app.state.key_mode = KeyMode::AwaitingG,
                        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                            tx.send(AppEvent::Enter)?
                        }
                        KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
                            tx.send(AppEvent::Back)?
                        }
                        KeyCode::Char('q') => tx.send(AppEvent::Quit)?,
                        _ => {}
                    },
                    KeyMode::AwaitingG => {
                        match key.code {
                            KeyCode::Char('g') => tx.send(AppEvent::GoTop)?,
                            KeyCode::Char('p') => tx.send(AppEvent::JumpToPlaylists)?,
                            KeyCode::Char('l') => tx.send(AppEvent::JumpToLiked)?,
                            _ => {}
                        }
                        app.state.key_mode = KeyMode::Normal;
                    }
                }
            }
        }
        if app.state.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn spawn_initial_fetches(spotify: AuthCodePkceSpotify, tx: mpsc::UnboundedSender<AppEvent>) {
    {
        let (sp, tx) = (spotify.clone(), tx.clone());
        tokio::spawn(async move {
            match svc::fetch_user(&sp).await {
                Ok(user) => {
                    let uid = user.id.clone();
                    let _ = tx.send(AppEvent::UserLoaded(user.display_name));
                    match svc::fetch_playlists(&sp, &uid).await {
                        Ok(pl) => {
                            let _ = tx.send(AppEvent::PlaylistsLoaded(pl));
                        }
                        Err(e) => {
                            tracing::error!("playlists: {e:#}");
                            let _ = tx.send(AppEvent::PlaylistsLoaded(vec![]));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("user: {e:#}");
                    let _ = tx.send(AppEvent::PlaylistsLoaded(vec![]));
                }
            }
        });
    }
    {
        let (sp, tx) = (spotify.clone(), tx.clone());
        tokio::spawn(async move {
            sleep(Duration::from_millis(300)).await;
            match svc::fetch_liked_tracks(&sp).await {
                Ok(t) => {
                    let _ = tx.send(AppEvent::LikedTracksLoaded(t));
                }
                Err(e) => {
                    tracing::error!("liked: {e:#}");
                    let _ = tx.send(AppEvent::LikedTracksLoaded(vec![]));
                }
            }
        });
    }
}

fn maybe_fetch_explorer(
    current: &Option<ExplorerNode>,
    last: &mut Option<ExplorerNode>,
    in_prog: &mut bool,
    spotify: AuthCodePkceSpotify,
    tx: mpsc::UnboundedSender<AppEvent>,
) {
    if *in_prog {
        return;
    }
    let should = match (current, last.as_ref()) {
        (None, _) | (Some(ExplorerNode::LikedTracks), None) => false,
        (Some(c), None) => !matches!(c, ExplorerNode::LikedTracks),
        (Some(c), Some(p)) => !nodes_equal(c, p),
    };
    if !should {
        return;
    }
    *last = current.clone();
    *in_prog = true;
    if let Some(ExplorerNode::PlaylistTracks(id, _, _)) = current {
        let id = id.clone();
        tokio::spawn(async move {
            match svc::fetch_playlist_tracks(&spotify, &id).await {
                Ok(t) => {
                    let _ = tx.send(AppEvent::ExplorerTracksLoaded(t));
                }
                Err(e) => {
                    tracing::error!("tracks: {e:#}");
                    let _ = tx.send(AppEvent::ExplorerTracksLoaded(vec![]));
                }
            }
        });
    } else {
        *in_prog = false;
    }
}

fn nodes_equal(a: &ExplorerNode, b: &ExplorerNode) -> bool {
    match (a, b) {
        (ExplorerNode::PlaylistTracks(id1, ..), ExplorerNode::PlaylistTracks(id2, ..)) => {
            id1 == id2
        }
        (ExplorerNode::LikedTracks, ExplorerNode::LikedTracks) => true,
        _ => false,
    }
}
