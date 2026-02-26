use super::{
    events::AppEvent,
    state::{AppState, ExplorerNode, Focus, KeyMode},
};

const PLAYLISTS: &[&str] = &["Workout Mix", "Chill Vibes", "Focus Mode"];

const LIKED: &[&str] = &["Liked Songs"];

const ARTISTS: &[&str] = &["Daft Punk", "Radiohead", "Arctic Monkeys"];

pub fn reduce(state: &mut AppState, event: AppEvent) {
    match event {
        AppEvent::Quit => state.should_quit = true,

        AppEvent::MoveDown(count) => {
            move_cursor(state, count as isize);
        }

        AppEvent::MoveUp(count) => {
            move_cursor(state, -(count as isize));
        }

        AppEvent::GoTop => set_cursor(state, 0),
        AppEvent::GoBottom => {
            let max = max_index(state);
            set_cursor(state, max);
        }

        AppEvent::GoMiddle => {
            let max = max_index(state);
            set_cursor(state, max / 2);
        }

        AppEvent::Enter => {
            state.focus = Focus::Explorer;
        }

        AppEvent::Back => {
            state.focus = Focus::Sidebar;
        }

        AppEvent::JumpToPlaylists => {
            state.navigation.selected_index = 0;
            update_sidebar_selection(state);
            state.key_mode = KeyMode::Normal;
        }

        AppEvent::JumpToLiked => {
            state.navigation.selected_index = PLAYLISTS.len();
            update_sidebar_selection(state);
            state.key_mode = KeyMode::Normal;
        }

        AppEvent::JumpToArtists => {
            state.navigation.selected_index = PLAYLISTS.len() + LIKED.len();
            update_sidebar_selection(state);
            state.key_mode = KeyMode::Normal;
        }

        _ => {}
    }
}

fn move_cursor(state: &mut AppState, delta: isize) {
    let max = max_index(state) as isize;

    match state.focus {
        Focus::Sidebar => {
            let mut idx = state.navigation.selected_index as isize;
            idx = (idx + delta).clamp(0, max);
            state.navigation.selected_index = idx as usize;
            update_sidebar_selection(state);
        }
        Focus::Explorer => {
            let mut idx = state.explorer_selected_index as isize;
            idx = (idx + delta).clamp(0, max);
            state.explorer_selected_index = idx as usize;
        }
    }
}

fn set_cursor(state: &mut AppState, idx: usize) {
    match state.focus {
        Focus::Sidebar => {
            state.navigation.selected_index = idx.min(max_index(state));
            update_sidebar_selection(state);
        }
        Focus::Explorer => {
            state.explorer_selected_index = idx.min(max_index(state));
        }
    }
}

fn max_index(state: &AppState) -> usize {
    match state.focus {
        Focus::Sidebar => PLAYLISTS.len() + LIKED.len() + ARTISTS.len() - 1,
        Focus::Explorer => match state.explorer_stack.last() {
            Some(ExplorerNode::PlaylistTracks(_)) => 2,
            Some(ExplorerNode::ArtistAlbums(_)) => 1,
            Some(ExplorerNode::LikedTracks) => 1,
            None => 0,
        },
    }
}

fn update_sidebar_selection(state: &mut AppState) {
    let idx = state.navigation.selected_index;

    state.explorer_selected_index = 0;
    state.explorer_stack.clear();

    if idx < PLAYLISTS.len() {
        state
            .explorer_stack
            .push(ExplorerNode::PlaylistTracks(PLAYLISTS[idx].into()));
    } else if idx < PLAYLISTS.len() + LIKED.len() {
        state.explorer_stack.push(ExplorerNode::LikedTracks);
    } else {
        let artist_index = idx - PLAYLISTS.len() - LIKED.len();
        state
            .explorer_stack
            .push(ExplorerNode::ArtistAlbums(ARTISTS[artist_index].into()));
    }
}
