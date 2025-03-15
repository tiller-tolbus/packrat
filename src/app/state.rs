/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// File explorer mode
    Explorer,
    /// Text viewer mode
    Viewer,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Explorer
    }
}

/// Application state
#[derive(Default)]
pub struct AppState {
    /// Flag to indicate if the application should quit
    pub should_quit: bool,
    /// Current application mode
    pub mode: AppMode,
}