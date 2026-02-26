use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::state::{AppState, ExplorerNode, Focus};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let count = state
        .pending_count
        .map(|c| c.to_string())
        .unwrap_or_else(|| "".to_string());

    let focus = match state.focus {
        Focus::Sidebar => "SIDEBAR",
        Focus::Explorer => "EXPLORER",
    };

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

    let line = Line::from(vec![
        Span::styled(focus, Style::default().fg(Color::Cyan)),
        Span::raw("   "),
        Span::styled(count, Style::default().fg(Color::Yellow)),
        Span::raw("   "),
        Span::raw(breadcrumb),
    ]);

    let paragraph = Paragraph::new(line).block(
        Block::default().borders(Borders::ALL), // full rectangle
    );

    frame.render_widget(paragraph, area);
}
