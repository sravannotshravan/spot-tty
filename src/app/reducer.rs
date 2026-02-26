use super::{events::AppEvent, state::AppState};

pub fn reduce(state: &mut AppState, event: AppEvent) {
    match event {
        AppEvent::Quit => {
            state.should_quit = true;
        }

        AppEvent::NavigateDown => {
            state.navigation.selected_index = state.navigation.selected_index.saturating_add(1);
        }

        AppEvent::NavigateUp => {
            state.navigation.selected_index = state.navigation.selected_index.saturating_sub(1);
        }
    }
}
