use super::events::AppEvent;
use super::reducer::reduce;
use super::state::{AppState, Focus, KeyMode, NavigationState};

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState {
                should_quit: false,

                navigation: NavigationState { selected_index: 0 },

                explorer_stack: vec![],
                explorer_selected_index: 0,

                key_mode: KeyMode::Normal,
                focus: Focus::Sidebar,

                // 🔥 Initialize new vim state fields
                pending_count: None,
                awaiting_gg: false,
            },
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        reduce(&mut self.state, event);
    }
}
