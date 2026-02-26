use ratatui::{
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::state::{AppState, ExplorerContext};

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let items: Vec<ListItem> = match &state.explorer {
        ExplorerContext::Playlist(name) => vec![
            ListItem::new(format!("Tracks in {}", name)),
            ListItem::new("Track 1"),
            ListItem::new("Track 2"),
            ListItem::new("Track 3"),
        ],
        ExplorerContext::LikedSongs => vec![
            ListItem::new("Liked Track 1"),
            ListItem::new("Liked Track 2"),
        ],
        ExplorerContext::Artist(name) => vec![
            ListItem::new(format!("Albums by {}", name)),
            ListItem::new("Album A"),
            ListItem::new("Album B"),
        ],
    };

    let list = List::new(items).block(Block::default().title(" Explorer ").borders(Borders::ALL));

    frame.render_widget(list, area);
}
