use crate::navigation::node::Node;

pub struct NavigationState {
    pub stack: Vec<Node>,
    pub selected_index: usize,
}

pub struct AppState {
    pub should_quit: bool,
    pub navigation: NavigationState,
}
