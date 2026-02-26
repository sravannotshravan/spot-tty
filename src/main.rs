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

use app::{app::App, events::AppEvent};

use ui::{explorer, layout, player, sidebar};

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
        terminal.draw(|frame| {
            let areas = layout::split(frame.size());

            sidebar::render(frame, areas.sidebar, &app.state);
            explorer::render(frame, areas.main, &app.state);
            player::render(frame, areas.player, &app.state);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        tx.send(AppEvent::Quit)?;
                    }

                    // Down movement
                    KeyCode::Char('j') | KeyCode::Down => {
                        tx.send(AppEvent::NavigateDown)?;
                    }

                    // Up movement
                    KeyCode::Char('k') | KeyCode::Up => {
                        tx.send(AppEvent::NavigateUp)?;
                    }

                    _ => {}
                }
            }
        }

        while let Ok(event) = rx.try_recv() {
            app.handle_event(event);
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
