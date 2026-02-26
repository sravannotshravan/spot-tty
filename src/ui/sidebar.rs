use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::state::AppState;

const PLAYLISTS: &[&str] = &["Workout Mix", "Chill Vibes", "Focus Mode"];

const LIKED: &[&str] = &["Liked Songs"];

const ARTISTS: &[&str] = &["Daft Punk", "Radiohead", "Arctic Monkeys"];

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((PLAYLISTS.len() + 2) as u16),
            Constraint::Length((LIKED.len() + 2) as u16),
            Constraint::Min(0),
        ])
        .split(area);

    render_section(
        frame,
        sections[0],
        " Playlists ",
        PLAYLISTS,
        state.navigation.selected_index,
        0,
    );

    render_section(
        frame,
        sections[1],
        " Liked ",
        LIKED,
        state.navigation.selected_index,
        PLAYLISTS.len(),
    );

    render_section(
        frame,
        sections[2],
        " Artists ",
        ARTISTS,
        state.navigation.selected_index,
        PLAYLISTS.len() + LIKED.len(),
    );
}

fn render_section(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    title: &str,
    items: &[&str],
    global_index: usize,
    offset: usize,
) {
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|item| ListItem::new(format!(" {}", item)))
        .collect();

    let mut list_state = ListState::default();

    if global_index >= offset && global_index < offset + items.len() {
        list_state.select(Some(global_index - offset));
    }

    let list = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_stateful_widget(list, area, &mut list_state);
}
