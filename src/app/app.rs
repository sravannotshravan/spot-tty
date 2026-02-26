use super::events::AppEvent;
use super::reducer::reduce;
use super::state::{AppState, ExplorerContext, KeyMode, NavigationState};

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState {
                should_quit: false,
                navigation: NavigationState { selected_index: 0 },
                explorer: ExplorerContext::Playlist("Workout Mix".into()),
                key_mode: KeyMode::Normal,
            },
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        reduce(&mut self.state, event);
    }
}
