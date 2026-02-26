use ratatui::{widgets::{Block, Borders, Paragraph}, Frame};
use crate::app::state::AppState;

pub fn render(frame: &mut Frame, area: ratatui::layout::Rect, _state: &AppState) {
    let paragraph = Paragraph::new(" No track playing ")
        .block(
            Block::default()
                .title(" Player ")
                .borders(Borders::ALL),
        );

    frame.render_widget(paragraph, area);
}
