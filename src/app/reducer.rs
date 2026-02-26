use super::{
    events::AppEvent,
    state::{AppState, ExplorerContext, KeyMode},
};

const PLAYLISTS: &[&str] = &["Workout Mix", "Chill Vibes", "Focus Mode"];

const LIKED: &[&str] = &["Liked Songs"];

const ARTISTS: &[&str] = &["Daft Punk", "Radiohead", "Arctic Monkeys"];

pub fn reduce(state: &mut AppState, event: AppEvent) {
    match event {
        AppEvent::Quit => {
            state.should_quit = true;
        }

        AppEvent::NavigateDown => {
            let total = PLAYLISTS.len() + LIKED.len() + ARTISTS.len();
            state.navigation.selected_index = (state.navigation.selected_index + 1).min(total - 1);

            update_explorer(state);
        }

        AppEvent::NavigateUp => {
            state.navigation.selected_index = state.navigation.selected_index.saturating_sub(1);

            update_explorer(state);
        }

        AppEvent::JumpToPlaylists => {
            state.navigation.selected_index = 0;
            update_explorer(state);
            state.key_mode = KeyMode::Normal;
        }

        AppEvent::JumpToLiked => {
            state.navigation.selected_index = PLAYLISTS.len();
            update_explorer(state);
            state.key_mode = KeyMode::Normal;
        }

        AppEvent::JumpToArtists => {
            state.navigation.selected_index = PLAYLISTS.len() + LIKED.len();
            update_explorer(state);
            state.key_mode = KeyMode::Normal;
        }

        AppEvent::EnterGMode => {
            state.key_mode = KeyMode::AwaitingG;
        }

        AppEvent::ExitGMode => {
            state.key_mode = KeyMode::Normal;
        }
    }
}

fn update_explorer(state: &mut AppState) {
    let idx = state.navigation.selected_index;

    if idx < PLAYLISTS.len() {
        state.explorer = ExplorerContext::Playlist(PLAYLISTS[idx].to_string());
    } else if idx < PLAYLISTS.len() + LIKED.len() {
        state.explorer = ExplorerContext::LikedSongs;
    } else {
        let artist_index = idx - PLAYLISTS.len() - LIKED.len();
        state.explorer = ExplorerContext::Artist(ARTISTS[artist_index].to_string());
    }
}
