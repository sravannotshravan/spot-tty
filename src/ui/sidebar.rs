use crate::app::state::{AppState, Focus};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

const PLAYLISTS: &[&str] = &["Workout Mix", "Chill Vibes", "Focus Mode"];
const LIKED: &[&str] = &["Liked Songs"];
const ARTISTS: &[&str] = &["Daft Punk", "Radiohead", "Arctic Monkeys"];

// ─────────────────────────────────────────────
// Highlight styles
// ─────────────────────────────────────────────

/// Active panel: bright, bold, coloured background
fn active_highlight() -> Style {
    Style::default()
        .bg(Color::Rgb(60, 65, 80))
        .fg(Color::Rgb(245, 224, 220))
        .add_modifier(Modifier::BOLD)
}

/// Inactive panel: very muted — dim text, near-invisible background
fn inactive_highlight() -> Style {
    Style::default()
        .bg(Color::Rgb(35, 35, 40))
        .fg(Color::Rgb(120, 120, 130))
        .add_modifier(Modifier::DIM)
}

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    // ── Outer vertical split: user box at top, library sections below ──
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // user name box
            Constraint::Min(0),    // library sections
        ])
        .split(area);

    render_user_box(frame, outer[0], state);

    // ── Three library sections fill the remaining space ──
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ])
        .split(outer[1]);

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

fn render_user_box(frame: &mut Frame, area: ratatui::layout::Rect, state: &AppState) {
    // In future this will come from AppState once the Spotify service is wired up.
    // For now we read a placeholder; swap `state.user_name.as_deref()` in later.
    let name = state.user_name.as_deref().unwrap_or("─────────────");

    let paragraph = Paragraph::new(format!(" {}", name))
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
) {
    let is_sidebar_active = state.focus == Focus::Sidebar;
    let highlight = if is_sidebar_active {
        active_highlight()
    } else {
        inactive_highlight()
    };

    let height = area.height as usize;
    let mut rows: Vec<ListItem> = Vec::new();

    for row in 0..height {
        if row < items.len() {
            let absolute_index = offset + row;
            let number =
                (absolute_index as isize - state.navigation.selected_index as isize).abs() as usize;
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

    let list = List::new(rows)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(highlight);

    frame.render_stateful_widget(list, area, &mut list_state);
}
