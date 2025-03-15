use std::time::Instant;

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// File explorer mode
    Explorer,
    /// Text viewer mode
    Viewer,
    /// Text editor mode
    Editor,
}

impl Default for AppMode {
    fn default() -> Self {
        Self::Explorer
    }
}

/// Application state
pub struct AppState {
    /// Flag to indicate if the application should quit
    pub should_quit: bool,
    /// Current application mode
    pub mode: AppMode,
    /// Whether to show the help panel
    pub show_help: bool,
    /// Optional debug message to display on screen
    pub debug_message: Option<String>,
    /// Timestamp when debug message was set (for auto-clearing)
    pub debug_message_time: Option<Instant>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            should_quit: false,
            mode: AppMode::default(),
            show_help: false,
            debug_message: None,
            debug_message_time: None,
        }
    }
}

impl AppState {
    /// Set a debug message to be displayed at the bottom of the screen
    /// The message will be automatically cleared after the specified duration (in seconds)
    pub fn set_debug_message(&mut self, message: String, _duration_secs: u64) {
        self.debug_message = Some(message);
        self.debug_message_time = Some(std::time::Instant::now());
        
        // The message will be cleared in the app's main loop after the duration expires
    }
    
    /// Clear the current debug message
    pub fn clear_debug_message(&mut self) {
        self.debug_message = None;
        self.debug_message_time = None;
    }
    
    /// Check if the debug message should be cleared based on its duration
    pub fn should_clear_debug_message(&self, duration_secs: u64) -> bool {
        if let Some(time) = self.debug_message_time {
            let elapsed = time.elapsed().as_secs();
            return elapsed >= duration_secs;
        }
        false
    }
}