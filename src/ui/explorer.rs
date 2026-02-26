use ratatui::{
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let items = vec![
        ListItem::new(" Playlists "),
        ListItem::new(" Liked Songs "),
        ListItem::new(" Albums "),
        ListItem::new(" Artists "),
    ];

    let mut list_state = ListState::default();
    list_state.select(Some(state.navigation.selected_index));

    let list = List::new(items)
        .block(Block::default().title(" Explorer ").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut list_state);
}
