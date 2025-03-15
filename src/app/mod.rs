pub mod state;
mod events;

use anyhow::Result;
use crossterm::event::{self, Event};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::Duration;

use self::events::EventHandler;
use self::state::{AppMode, AppState};
use crate::explorer::Explorer;
use crate::ui::render;
use crate::viewer::Viewer;

/// Main application struct
pub struct App {
    /// Terminal backend
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Application state
    state: AppState,
    /// Event handler
    events: EventHandler,
    /// File explorer
    explorer: Explorer,
    /// Text viewer
    viewer: Viewer,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create app components
        let state = AppState::default();
        let events = EventHandler::new(Duration::from_millis(100));
        let explorer = Explorer::new(".")?; // Start in current directory
        let viewer = Viewer::new();

        Ok(Self {
            terminal,
            state,
            events,
            explorer,
            viewer,
        })
    }

    /// Run the application main loop
    pub fn run(&mut self) -> Result<()> {
        // Main loop
        while !self.state.should_quit {
            // Draw the UI
            self.terminal.draw(|frame| {
                render(frame, &self.state, &self.explorer, &self.viewer);
            })?;

            // Handle events
            if let Ok(event) = self.events.next() {
                if let Event::Key(key_event) = event {
                    self.handle_key_event(key_event);
                }
            }
        }

        // Cleanup terminal
        terminal::disable_raw_mode()?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    /// Handle key events
    fn handle_key_event(&mut self, event: event::KeyEvent) {

        match self.state.mode {
            AppMode::Explorer => self.handle_explorer_key_event(event),
            AppMode::Viewer => self.handle_viewer_key_event(event),
        }
    }

    /// Handle key events in explorer mode
    fn handle_explorer_key_event(&mut self, event: event::KeyEvent) {
        use crossterm::event::KeyCode;

        // If help panel is shown, any key dismisses it (except '?' which toggles)
        if self.state.show_help && event.code != KeyCode::Char('?') {
            self.state.show_help = false;
            return;
        }

        match event.code {
            // Toggle help panel
            KeyCode::Char('?') => {
                self.state.show_help = !self.state.show_help;
            },
            
            // Quit application
            KeyCode::Char('q') => self.state.should_quit = true,
            
            // Basic navigation in explorer
            KeyCode::Up | KeyCode::Char('k') => self.explorer.select_previous(),
            KeyCode::Down | KeyCode::Char('j') => self.explorer.select_next(),
            
            // Page navigation
            KeyCode::PageUp => {
                // Estimate page size as terminal height minus headers/footers (approx 10 lines)
                let page_size = self.terminal.size().unwrap_or_default().height as usize;
                let effective_page_size = if page_size > 10 { page_size - 10 } else { 1 };
                self.explorer.select_page_up(effective_page_size);
            },
            KeyCode::PageDown => {
                let page_size = self.terminal.size().unwrap_or_default().height as usize;
                let effective_page_size = if page_size > 10 { page_size - 10 } else { 1 };
                self.explorer.select_page_down(effective_page_size);
            },
            
            // Home/End navigation
            KeyCode::Home => self.explorer.select_first(),
            KeyCode::End => self.explorer.select_last(),
            
            // Directory/file navigation
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
                if self.explorer.entries().is_empty() {
                    return;
                }
                
                let selected = &self.explorer.entries()[self.explorer.selected_index()];
                
                if selected.is_dir {
                    // Open directory
                    if let Err(e) = self.explorer.open_selected() {
                        eprintln!("Error: {}", e);
                    }
                } else {
                    // Open file in viewer
                    if let Err(e) = self.viewer.open_file(&selected.path) {
                        eprintln!("Error opening file: {}", e);
                    } else {
                        // Switch to viewer mode
                        self.state.mode = AppMode::Viewer;
                    }
                }
            },
            KeyCode::Char('h') | KeyCode::Left => {
                // Go back to parent directory
                if let Err(e) = self.explorer.go_to_parent() {
                    eprintln!("Error: {}", e);
                }
            },
            _ => {}
        }
    }

    /// Handle key events in viewer mode
    fn handle_viewer_key_event(&mut self, event: event::KeyEvent) {
        use crossterm::event::KeyCode;

        // If help panel is shown, any key dismisses it (except '?' which toggles)
        if self.state.show_help && event.code != KeyCode::Char('?') {
            self.state.show_help = false;
            return;
        }

        match event.code {
            // Toggle help panel
            KeyCode::Char('?') => {
                self.state.show_help = !self.state.show_help;
            },
            
            // Exit viewer and return to explorer
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state.mode = AppMode::Explorer;
            },
            
            // Basic scrolling
            KeyCode::Up | KeyCode::Char('k') => self.viewer.scroll_up(),
            KeyCode::Down | KeyCode::Char('j') => self.viewer.scroll_down(),
            
            // Page scrolling
            KeyCode::PageUp => {
                let page_size = self.terminal.size().unwrap_or_default().height as usize;
                let effective_page_size = if page_size > 10 { page_size - 10 } else { 1 };
                self.viewer.scroll_page_up(effective_page_size);
            },
            KeyCode::PageDown => {
                let page_size = self.terminal.size().unwrap_or_default().height as usize;
                let effective_page_size = if page_size > 10 { page_size - 10 } else { 1 };
                self.viewer.scroll_page_down(effective_page_size);
            },
            
            // Jump to top/bottom
            KeyCode::Home => self.viewer.scroll_to_top(),
            KeyCode::End => self.viewer.scroll_to_bottom(),
            
            _ => {}
        }
    }
}