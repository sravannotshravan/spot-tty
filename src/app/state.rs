pub struct NavigationState {
    pub selected_index: usize,
}

#[derive(Clone)]
pub enum ExplorerContext {
    Playlist(String),
    LikedSongs,
    Artist(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    Normal,
    AwaitingG,
}

pub struct AppState {
    pub should_quit: bool,
    pub navigation: NavigationState,
    pub explorer: ExplorerContext,
    pub key_mode: KeyMode,
}
