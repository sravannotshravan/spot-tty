use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Areas {
    pub sidebar: Rect,
    pub main: Rect,
    pub player: Rect,
}

pub fn split(area: Rect) -> Areas {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(vertical[0]);

    Areas {
        sidebar: horizontal[0],
        main: horizontal[1],
        player: vertical[1],
    }
}
