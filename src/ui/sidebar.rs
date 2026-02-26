use ratatui::{widgets::{Block, Borders}, Frame};
use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, _state: &AppState) {
    let block = Block::default()
        .title(" Library ")
        .borders(Borders::ALL);

    frame.render_widget(block, area);
}
