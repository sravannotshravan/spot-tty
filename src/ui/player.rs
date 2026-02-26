use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state::{AppState, ExplorerNode};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    // ─────────────────────────────────────────────
    // Title
    // ─────────────────────────────────────────────

    let breadcrumb = match state.explorer_stack.last() {
        Some(ExplorerNode::PlaylistTracks(name)) => {
            format!("Library › Playlist › {}", name)
        }
        Some(ExplorerNode::ArtistAlbums(name)) => {
            format!("Library › Artist › {}", name)
        }
        Some(ExplorerNode::LikedTracks) => "Library › Liked Songs".to_string(),
        None => "Library".to_string(),
    };

    let current_track = "Track Name";

    let title = Line::from(vec![
        Span::raw(breadcrumb),
        Span::raw(" › "),
        Span::styled(
            current_track,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let block = Block::default().borders(Borders::ALL).title(title);

    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // ─────────────────────────────────────────────
    // Padding
    // ─────────────────────────────────────────────

    let padded = Rect {
        x: inner.x + 2,
        y: inner.y,
        width: inner.width.saturating_sub(4),
        height: inner.height,
    };

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(14), // playhead
            Constraint::Min(10),    // sweep
            Constraint::Length(4),  // gap
            Constraint::Length(16), // visualizer
        ])
        .split(padded);

    let playhead_area = layout[0];
    let sweep_area = layout[1];
    let visualizer_area = layout[3];

    // ─────────────────────────────────────────────
    // Subtle Base Sweep Background
    // ─────────────────────────────────────────────

    frame.render_widget(
        Block::default().style(
            Style::default().bg(Color::Rgb(25, 40, 25)), // subtle dark green base
        ),
        sweep_area,
    );

    // ─────────────────────────────────────────────
    // Filled Progress
    // ─────────────────────────────────────────────

    let progress = state.playback_progress.clamp(0.0, 1.0);
    let fill_width = (sweep_area.width as f64 * progress) as u16;

    if fill_width > 0 {
        let progress_rect = Rect {
            x: sweep_area.x,
            y: sweep_area.y,
            width: fill_width,
            height: sweep_area.height,
        };

        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Green)),
            progress_rect,
        );
    }

    // ─────────────────────────────────────────────
    // Foreground
    // ─────────────────────────────────────────────

    frame.render_widget(
        Paragraph::new("▶ 1:23 / 3:45").style(Style::default().fg(Color::White)),
        playhead_area,
    );

    frame.render_widget(
        Paragraph::new("▂▅▇▆▃▂▇▅▃▂").style(Style::default().fg(Color::Green)),
        visualizer_area,
    );
}
