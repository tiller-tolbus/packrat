pub mod state;
mod events;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::fs::{self, File};
use std::io::{self, Stdout, Write};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use self::events::EventHandler;
use self::state::{AppMode, AppState};
use crate::config::Config;
use packrat::editor::Editor;
use crate::explorer::Explorer;
use crate::ui::{render, UiSerializer};
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
    /// Text editor
    editor: Editor,
    /// Application configuration
    config: Config,
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
        let editor = Editor::new();
        let config = Config::default();
        
        // Create debug directory if enabled
        if config.enable_debug {
            fs::create_dir_all(&config.debug_dir)
                .with_context(|| format!("Failed to create debug directory: {:?}", config.debug_dir))?;
        }

        Ok(Self {
            terminal,
            state,
            events,
            explorer,
            viewer,
            editor,
            config,
        })
    }

    /// Run the application main loop
    pub fn run(&mut self) -> Result<()> {
        // Configure debug message auto-clear duration (in seconds)
        const DEBUG_MESSAGE_DURATION: u64 = 5;
        
        // Main loop
        while !self.state.should_quit {
            // Check if we should clear any debug messages
            if self.state.should_clear_debug_message(DEBUG_MESSAGE_DURATION) {
                self.state.clear_debug_message();
            }
            
            // Draw the UI
            self.terminal.draw(|frame| {
                render(frame, &self.state, &self.explorer, &self.viewer, &self.editor);
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
        // Handle debug shortcuts if enabled, regardless of mode
        if self.config.enable_debug && event.modifiers.contains(KeyModifiers::CONTROL) {
            match event.code {
                // Ctrl+D: Dump UI state
                KeyCode::Char('d') => {
                    if let Err(e) = self.dump_ui_state() {
                        eprintln!("Error dumping UI state: {}", e);
                    }
                    return;
                },
                _ => {}
            }
        }

        match self.state.mode {
            AppMode::Explorer => self.handle_explorer_key_event(event),
            AppMode::Viewer => self.handle_viewer_key_event(event),
            AppMode::Editor => self.handle_editor_key_event(event),
        }
    }
    
    /// Dump the current UI state to a file in the debug directory
    fn dump_ui_state(&mut self) -> Result<()> {
        // Generate a timestamp for the filename
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        // Create the debug file path
        let debug_file_path = self.config.debug_dir.join(format!("ui_state_{}.txt", timestamp));
        
        // Create a new file
        let mut file = File::create(&debug_file_path)
            .with_context(|| format!("Failed to create debug file: {:?}", debug_file_path))?;
            
        // Capture the current UI state
        let ui_state = match self.state.mode {
            AppMode::Explorer => {
                UiSerializer::capture_explorer(&self.state, &self.explorer)
            },
            AppMode::Viewer => {
                UiSerializer::capture_viewer(&self.state, &self.viewer)
            },
            AppMode::Editor => {
                UiSerializer::capture_editor(&self.state)
            },
        };
        
        // Write the UI state to the file
        file.write_all(ui_state.as_bytes())
            .with_context(|| "Failed to write UI state to file")?;
            
        // Show the debug message in the UI overlay instead of printing to stdout
        let debug_message = format!("Debug information saved to: {}", debug_file_path.display());
        self.state.set_debug_message(debug_message, 5);
        
        Ok(())
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
            KeyCode::Char('q') | KeyCode::Esc => self.state.should_quit = true,
            
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
            
            // Toggle selection mode with Space
            KeyCode::Char(' ') => {
                self.viewer.toggle_selection_mode();
                let message = if self.viewer.is_selection_mode() {
                    "Selection mode activated - Use cursor keys to select text"
                } else {
                    "Selection mode deactivated"
                };
                self.state.set_debug_message(message.to_string(), 2);
            },
            
            // Enter editor mode with 'E' key
            KeyCode::Char('e') => {
                // Only enter editor mode if there is a selection
                if let Some((start, end)) = self.viewer.selection_range() {
                    let content = self.viewer.content();
                    // Extract the selected lines
                    let selected_lines = content[start..=end].to_vec();
                    
                    // Set the editor content with the selected lines
                    self.editor.set_content(selected_lines);
                    
                    // Switch to editor mode
                    self.state.mode = AppMode::Editor;
                    self.state.set_debug_message("Editing selected text".to_string(), 2);
                } else {
                    self.state.set_debug_message("No text selected for editing".to_string(), 2);
                }
            },
            
            // Line-based cursor movement
            KeyCode::Up | KeyCode::Char('k') => self.viewer.cursor_up(),
            KeyCode::Down | KeyCode::Char('j') => self.viewer.cursor_down(),
            
            // Page scrolling - keeps cursor in view
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
    
    /// Handle key events in editor mode
    fn handle_editor_key_event(&mut self, event: event::KeyEvent) {
        use crossterm::event::KeyCode;
        
        // Special key handling
        match event.code {
            // Exit editor and return to viewer
            KeyCode::Esc => {
                // Warn user if they have unsaved changes
                if self.editor.is_modified() {
                    self.state.set_debug_message("Exiting editor without saving changes".to_string(), 3);
                }
                self.state.mode = AppMode::Viewer;
            },
            
            // Save changes and return to viewer
            KeyCode::Char('s') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Get the edited content
                let _edited_content = self.editor.content();
                
                // TODO: Update viewer with the edited content (this will be implemented in Phase 4)
                
                // Switch back to viewer mode
                self.state.mode = AppMode::Viewer;
                self.state.set_debug_message("Changes saved".to_string(), 2);
            },
            
            // Handle the key event with the text editor
            _ => {
                // Let the editor handle the key event
                let handled = self.editor.handle_key_event(event);
                if !handled {
                    // If the editor didn't handle it, check for our custom keys
                    match event.code {
                        // Toggle help panel
                        KeyCode::Char('?') => {
                            self.state.show_help = !self.state.show_help;
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}