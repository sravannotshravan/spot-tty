use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{backend::CrosstermBackend, Terminal};

use tokio::sync::mpsc;

mod app;
mod cache;
mod config;
mod navigation;
mod services;
mod ui;

use app::{app::App, events::AppEvent, state::KeyMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut app = App::new();

    loop {
        // ─────────────────────────────────────────────
        // DRAW
        // ─────────────────────────────────────────────
        terminal.draw(|frame| {
            let areas = ui::layout::split(frame.size());

            ui::sidebar::render(frame, areas.sidebar, &app.state);
            ui::explorer::render(frame, areas.main, &app.state);
            ui::player::render(frame, areas.control, &app.state);
        })?;

        // ─────────────────────────────────────────────
        // INPUT
        // ─────────────────────────────────────────────
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Numeric prefix handling (e.g., 33j)
                if let KeyCode::Char(c) = key.code {
                    if c.is_ascii_digit() {
                        let digit = c.to_digit(10).unwrap() as usize;
                        let current = app.state.pending_count.unwrap_or(0);
                        app.state.pending_count = Some(current * 10 + digit);
                        continue;
                    }
                }

                let count = app.state.pending_count.take().unwrap_or(1);

                match app.state.key_mode {
                    KeyMode::Normal => {
                        match key.code {
                            // Movement
                            KeyCode::Char('j') | KeyCode::Down => {
                                tx.send(AppEvent::MoveDown(count))?
                            }

                            KeyCode::Char('k') | KeyCode::Up => tx.send(AppEvent::MoveUp(count))?,

                            // Go bottom
                            KeyCode::Char('G') => tx.send(AppEvent::GoBottom)?,

                            // Go middle
                            KeyCode::Char('M') => tx.send(AppEvent::GoMiddle)?,

                            // g prefix handling
                            KeyCode::Char('g') => {
                                app.state.key_mode = KeyMode::AwaitingG;
                            }

                            // Focus right
                            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                                tx.send(AppEvent::Enter)?
                            }

                            // Focus left
                            KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
                                tx.send(AppEvent::Back)?
                            }

                            // Quit
                            KeyCode::Char('q') => tx.send(AppEvent::Quit)?,

                            _ => {}
                        }
                    }

                    KeyMode::AwaitingG => {
                        match key.code {
                            KeyCode::Char('g') => tx.send(AppEvent::GoTop)?,

                            KeyCode::Char('p') => tx.send(AppEvent::JumpToPlaylists)?,

                            KeyCode::Char('l') => tx.send(AppEvent::JumpToLiked)?,

                            KeyCode::Char('a') => tx.send(AppEvent::JumpToArtists)?,

                            _ => {}
                        }

                        app.state.key_mode = KeyMode::Normal;
                    }
                }
            }
        }

        // ─────────────────────────────────────────────
        // EVENT PROCESSING
        // ─────────────────────────────────────────────
        while let Ok(event) = rx.try_recv() {
            app.handle_event(event);
        }

        if app.state.should_quit {
            break;
        }
    }

    // ─────────────────────────────────────────────
    // CLEANUP
    // ─────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
