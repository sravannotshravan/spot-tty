use crate::services::spotify::{PlaylistSummary, TrackSummary};
use crate::ui::cover::{CoverImage, ImageProtocol, RenderCache};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

#[derive(Clone)]
pub enum ExplorerNode {
    PlaylistTracks(String, String, bool),
    LikedTracks,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppStatus {
    Loading,
    Ready,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyMode {
    Normal,
    AwaitingG,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    Sidebar,
    Explorer,
}

pub struct NavigationState {
    pub selected_index: usize,
}

pub struct AppState {
    pub status: AppStatus,
    pub should_quit: bool,
    pub image_protocol: ImageProtocol,
    pub render_cache: RenderCache,

    pub loaded_user: bool,
    pub loaded_playlists: bool,
    pub loaded_liked: bool,
    pub explorer_fetch_pending: bool,

    pub user_name: Option<String>,
    pub playlists: Vec<PlaylistSummary>,
    pub liked_tracks: Vec<TrackSummary>,
    pub explorer_items: Vec<TrackSummary>,

    /// url → fully loaded CoverImage (in memory + uploaded to terminal)
    pub cover_cache: HashMap<String, CoverImage>,
    /// URLs currently being fetched — prevents duplicate requests
    pub cover_fetching: HashSet<String>,

    pub navigation: NavigationState,
    pub explorer_stack: Vec<ExplorerNode>,
    pub explorer_selected_index: usize,
    pub key_mode: KeyMode,
    pub focus: Focus,
    pub pending_count: Option<usize>,

    pub error_message: Option<String>,
    pub playback_progress: f64,
    pub visualizer_phase: usize,

    /// Debounce: timestamp of last navigation move.
    /// Detail panel image only renders once this is >120ms ago.
    pub last_nav_move: Option<Instant>,
}

impl AppState {
    /// True if the user has stopped scrolling long enough to render the large cover.
    pub fn scroll_settled(&self) -> bool {
        self.last_nav_move
            .map(|t| t.elapsed().as_millis() >= 120)
            .unwrap_or(true)
    }
}
