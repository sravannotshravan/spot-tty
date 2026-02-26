use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct LayoutAreas {
    pub sidebar: Rect,
    pub main: Rect,
    pub player: Rect,
}

pub fn split(area: Rect) -> LayoutAreas {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(30),
            Constraint::Min(1),
        ])
        .split(vertical[0]);

    LayoutAreas {
        sidebar: horizontal[0],
        main: horizontal[1],
        player: vertical[1],
    }
}
