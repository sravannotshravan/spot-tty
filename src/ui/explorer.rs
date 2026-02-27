use crate::app::state::{AppState, ExplorerNode, Focus};
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

// ─────────────────────────────────────────────
// Highlight styles (mirrors sidebar.rs)
// ─────────────────────────────────────────────

fn active_highlight() -> Style {
    Style::default()
        .bg(Color::Rgb(60, 65, 80))
        .fg(Color::Rgb(245, 224, 220))
        .add_modifier(Modifier::BOLD)
}

fn inactive_highlight() -> Style {
    Style::default()
        .bg(Color::Rgb(35, 35, 40))
        .fg(Color::Rgb(120, 120, 130))
        .add_modifier(Modifier::DIM)
}

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let is_explorer_active = state.focus == Focus::Explorer;
    let highlight = if is_explorer_active {
        active_highlight()
    } else {
        inactive_highlight()
    };

    let height = area.height as usize;

    let raw_items: Vec<String> = match state.explorer_stack.last() {
        Some(ExplorerNode::PlaylistTracks(name)) => vec![
            format!("Track 1 - {}", name),
            format!("Track 2 - {}", name),
            format!("Track 3 - {}", name),
        ],
        Some(ExplorerNode::ArtistAlbums(name)) => {
            vec![format!("Album A - {}", name), format!("Album B - {}", name)]
        }
        Some(ExplorerNode::LikedTracks) => {
            vec!["Liked Track 1".to_string(), "Liked Track 2".to_string()]
        }
        None => vec!["No Content".to_string()],
    };

    let mut rows: Vec<ListItem> = Vec::new();
    for row in 0..height {
        if row < raw_items.len() {
            let number = (row as isize - state.explorer_selected_index as isize).abs() as usize;
            rows.push(ListItem::new(format!("{:>3} │ {}", number, raw_items[row])));
        } else {
            rows.push(ListItem::new("    │"));
        }
    }

    // Only show a selection cursor when the explorer panel is active.
    // When inactive, the muted highlight on whatever was last selected is
    // enough of a ghost — no need for a blinking phantom cursor.
    let mut list_state = ListState::default();
    if is_explorer_active {
        list_state.select(Some(state.explorer_selected_index));
    }

    let block = Block::default()
        .title(" Explorer ")
        .borders(Borders::ALL)
        .border_style(if is_explorer_active {
            Style::default().fg(Color::Rgb(137, 180, 130)) // soft green tint when active
        } else {
            Style::default().fg(Color::Rgb(88, 91, 112)) // muted when inactive
        });

    let list = List::new(rows).block(block).highlight_style(highlight);

    frame.render_stateful_widget(list, area, &mut list_state);
}
