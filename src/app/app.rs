use super::state::{AppState, NavigationState};
use crate::navigation::node::Node;

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState {
                should_quit: false,
                navigation: NavigationState {
                    stack: vec![Node::Library],
                    selected_index: 0,
                },
            },
        }
    }

    pub fn handle_event(&mut self, event: super::events::AppEvent) {
        super::reducer::reduce(&mut self.state, event);
    }
}
