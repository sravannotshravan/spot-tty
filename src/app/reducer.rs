use super::{
    events::AppEvent,
    state::{AppState, AppStatus, ExplorerNode, Focus, KeyMode},
};

pub fn reduce(state: &mut AppState, event: AppEvent) {
    match event {
        // ── Lifecycle ─────────────────────────────────────────────────────
        AppEvent::Quit => state.should_quit = true,

        // ── Data loaded from async tasks ──────────────────────────────────
        AppEvent::UserLoaded(name) => {
            state.user_name = Some(name);
            state.loaded_user = true;
            check_ready(state);
        }

        AppEvent::PlaylistsLoaded(playlists) => {
            state.playlists = playlists;
            state.loaded_playlists = true;
            update_sidebar_selection(state);
            check_ready(state);
        }

        AppEvent::LikedTracksLoaded(tracks) => {
            state.liked_tracks = tracks;
            state.loaded_liked = true;
            // If liked songs is currently selected, refresh explorer content
            if let Some(ExplorerNode::LikedTracks) = state.explorer_stack.last() {
                state.explorer_items = state.liked_tracks.clone();
            }
            check_ready(state);
        }

        AppEvent::ArtistsLoaded(artists) => {
            state.artists = artists;
            state.loaded_artists = true;
            check_ready(state);
        }

        AppEvent::ExplorerTracksLoaded(tracks) => {
            state.explorer_items = tracks;
            state.explorer_selected_index = 0;
        }

        AppEvent::ExplorerAlbumsLoaded(albums) => {
            state.explorer_albums = albums;
            state.explorer_selected_index = 0;
        }

        // Don't kill the whole app on a single load error — log it and keep going
        AppEvent::LoadError(msg) => {
            tracing::error!("Load error: {}", msg);
            state.error_message = Some(msg);
            // Still mark partial loads as done so the UI isn't stuck on Loading
            check_ready(state);
        }

        // ── Navigation ────────────────────────────────────────────────────
        AppEvent::MoveDown(count) => move_cursor(state, count as isize),
        AppEvent::MoveUp(count) => move_cursor(state, -(count as isize)),
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
            state.navigation.selected_index = state.playlists.len();
            update_sidebar_selection(state);
            state.key_mode = KeyMode::Normal;
        }
        AppEvent::JumpToArtists => {
            state.navigation.selected_index = state.playlists.len() + 1;
            update_sidebar_selection(state);
            state.key_mode = KeyMode::Normal;
        }

        _ => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Move helpers
// ─────────────────────────────────────────────────────────────────────────────

fn move_cursor(state: &mut AppState, delta: isize) {
    let max = max_index(state) as isize;
    match state.focus {
        Focus::Sidebar => {
            let idx = (state.navigation.selected_index as isize + delta).clamp(0, max);
            state.navigation.selected_index = idx as usize;
            update_sidebar_selection(state);
        }
        Focus::Explorer => {
            let idx = (state.explorer_selected_index as isize + delta).clamp(0, max);
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
        Focus::Sidebar => {
            // playlists + 1 liked songs row + artists, minus 1 for 0-based
            (state.playlists.len() + 1 + state.artists.len()).saturating_sub(1)
        }
        Focus::Explorer => match state.explorer_stack.last() {
            Some(ExplorerNode::PlaylistTracks(_, _)) => {
                state.explorer_items.len().saturating_sub(1)
            }
            Some(ExplorerNode::ArtistAlbums(_, _)) => state.explorer_albums.len().saturating_sub(1),
            Some(ExplorerNode::LikedTracks) => state.explorer_items.len().saturating_sub(1),
            None => 0,
        },
    }
}

fn update_sidebar_selection(state: &mut AppState) {
    let idx = state.navigation.selected_index;
    state.explorer_selected_index = 0;
    state.explorer_stack.clear();
    state.explorer_items.clear();
    state.explorer_albums.clear();

    let pl_len = state.playlists.len();

    if idx < pl_len {
        let pl = &state.playlists[idx];
        state
            .explorer_stack
            .push(ExplorerNode::PlaylistTracks(pl.id.clone(), pl.name.clone()));
    } else if idx == pl_len {
        state.explorer_stack.push(ExplorerNode::LikedTracks);
        state.explorer_items = state.liked_tracks.clone();
    } else {
        let artist_idx = idx - pl_len - 1;
        if let Some(artist) = state.artists.get(artist_idx) {
            state.explorer_stack.push(ExplorerNode::ArtistAlbums(
                artist.id.clone(),
                artist.name.clone(),
            ));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Transition to Ready once all four fetches have settled (success or error)
// ─────────────────────────────────────────────────────────────────────────────

fn check_ready(state: &mut AppState) {
    if state.status == AppStatus::Loading
        && state.loaded_user
        && state.loaded_playlists
        && state.loaded_liked
        && state.loaded_artists
    {
        state.status = AppStatus::Ready;
    }
}
