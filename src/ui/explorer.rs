use ratatui::{
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::state::{AppState, ExplorerNode, Focus};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
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
            let number: usize = if state.focus == Focus::Explorer {
                (row as isize - state.explorer_selected_index as isize).abs() as usize
            } else {
                row
            };

            rows.push(ListItem::new(format!("{:>3} │ {}", number, raw_items[row])));
        } else {
            rows.push(ListItem::new("    │"));
        }
    }

    let mut list_state = ListState::default();
    list_state.select(Some(state.explorer_selected_index));

    let highlight_style = if state.focus == Focus::Explorer {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::DIM)
    };

    let list = List::new(rows)
        .block(Block::default().title(" Explorer ").borders(Borders::ALL))
        .highlight_style(highlight_style);

    frame.render_stateful_widget(list, area, &mut list_state);
}
