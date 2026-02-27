use crate::app::state::{AppState, Focus};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

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
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    render_user_box(frame, outer[0], state);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Playlists
            Constraint::Length(3), // Liked Songs
        ])
        .split(outer[1]);

    let pl_len = state.playlists.len();

    render_section(
        frame,
        sections[0],
        " Playlists ",
        &state
            .playlists
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        state,
        0,
        state.loaded_playlists,
    );

    render_section(
        frame,
        sections[1],
        " Liked Songs ",
        &["Liked Songs"],
        state,
        pl_len,
        state.loaded_liked,
    );
}

fn render_user_box(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    let name = state.user_name.as_deref().unwrap_or("Connecting…");
    let paragraph = Paragraph::new(format!(" {name}"))
        .style(Style::default().fg(Color::Rgb(205, 214, 244)))
        .block(
            Block::default()
                .title(" Account ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(88, 91, 112))),
        );
    frame.render_widget(paragraph, area);
}

fn render_section(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    title: &str,
    items: &[&str],
    state: &AppState,
    offset: usize,
    loaded: bool,
) {
    let is_active = state.focus == Focus::Sidebar;
    let highlight = if is_active {
        active_highlight()
    } else {
        inactive_highlight()
    };

    if !loaded {
        let p = Paragraph::new(" Loading…")
            .style(Style::default().fg(Color::Rgb(100, 100, 110)))
            .block(Block::default().title(title).borders(Borders::ALL));
        frame.render_widget(p, area);
        return;
    }

    let rows: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let abs = offset + i;
            let rel = (abs as isize - state.navigation.selected_index as isize).unsigned_abs();
            ListItem::new(format!("{rel:>3} │ {name}"))
        })
        .collect();

    let mut list_state = ListState::default();
    if state.navigation.selected_index >= offset
        && state.navigation.selected_index < offset + items.len()
    {
        list_state.select(Some(state.navigation.selected_index - offset));
    }

    let list = List::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(highlight);

    frame.render_stateful_widget(list, area, &mut list_state);
}
