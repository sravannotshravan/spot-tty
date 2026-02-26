use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::state::{AppState, Focus};

const PLAYLISTS: &[&str] = &["Workout Mix", "Chill Vibes", "Focus Mode"];

const LIKED: &[&str] = &["Liked Songs"];

const ARTISTS: &[&str] = &["Daft Punk", "Radiohead", "Arctic Monkeys"];

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .split(area);

    render_section(frame, sections[0], " Playlists ", PLAYLISTS, state, 0);
    render_section(frame, sections[1], " Liked ", LIKED, state, PLAYLISTS.len());
    render_section(
        frame,
        sections[2],
        " Artists ",
        ARTISTS,
        state,
        PLAYLISTS.len() + LIKED.len(),
    );
}

fn render_section(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    title: &str,
    items: &[&str],
    state: &AppState,
    offset: usize,
) {
    let height = area.height as usize;
    let mut rows: Vec<ListItem> = Vec::new();

    for row in 0..height {
        if row < items.len() {
            let absolute_index = offset + row;

            let number: usize = if state.focus == Focus::Sidebar {
                (absolute_index as isize - state.navigation.selected_index as isize).abs() as usize
            } else {
                absolute_index
            };

            rows.push(ListItem::new(format!("{:>3} │ {}", number, items[row])));
        } else {
            rows.push(ListItem::new("    │"));
        }
    }

    let mut list_state = ListState::default();

    if state.navigation.selected_index >= offset
        && state.navigation.selected_index < offset + items.len()
    {
        list_state.select(Some(state.navigation.selected_index - offset));
    }

    let highlight_style = if state.focus == Focus::Sidebar {
        Style::default().bg(Color::White).fg(Color::Black)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let list = List::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(highlight_style);

    frame.render_stateful_widget(list, area, &mut list_state);
}
