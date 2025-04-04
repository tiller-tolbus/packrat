pub mod state;
mod events;

use anyhow::{Context, Result};
use ratatui::crossterm::event::{self, Event, KeyModifiers};
use ratatui::crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::crossterm::ExecutableCommand;
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
use crate::storage::ChunkStorage;

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
    /// Chunk storage
    chunk_storage: ChunkStorage,
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

        // Load configuration
        let config = Config::load()?;
        
        // Create app components
        let state = AppState::default();
        let events = EventHandler::new(Duration::from_millis(100));
        let source_dir = config.absolute_source_dir();
        let mut explorer = Explorer::new(&source_dir)?;
        let mut viewer = Viewer::new();
        let editor = Editor::new();
        
        // Configure viewer with token limit from config
        viewer.set_max_tokens_per_chunk(config.max_tokens_per_chunk);
        
        // Create debug directory if enabled
        if config.enable_debug {
            fs::create_dir_all(&config.debug_dir)
                .with_context(|| format!("Failed to create debug directory: {:?}", config.debug_dir))?;
        }
        
        // Initialize chunk storage
        let chunk_file = config.absolute_chunk_file();
        let chunk_storage = ChunkStorage::new(&chunk_file)
            .with_context(|| format!("Failed to initialize chunk storage at: {:?}", chunk_file))?;
        
        // Initialize chunking progress for files in the explorer
        if let Err(e) = explorer.init_chunking_progress(&chunk_storage) {
            eprintln!("Warning: Failed to initialize chunking progress: {}", e);
        }

        Ok(Self {
            terminal,
            state,
            events,
            explorer,
            viewer,
            editor,
            config,
            chunk_storage,
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
                render(frame, &self.state, &self.explorer, &self.viewer, &mut self.editor);
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
        use ratatui::crossterm::event::KeyCode;
        
        // Check if we're in editor insert mode - pass all non-control keys directly to editor
        let in_insert_mode = self.state.mode == AppMode::Editor && self.editor.is_in_insert_mode();
        
        // If help panel is shown, any key dismisses it (except '?' which toggles)
        if self.state.show_help && event.code != KeyCode::Char('?') {
            self.state.show_help = false;
            return;
        }

        // Handle '?' key to toggle help BUT NOT in insert mode
        if !in_insert_mode && event.code == KeyCode::Char('?') {
            self.state.show_help = !self.state.show_help;
            return;
        }

        // Handle debug shortcuts if enabled, regardless of mode
        // (Control keys still work in insert mode)
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
        use ratatui::crossterm::event::KeyCode;

        match event.code {
            
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
                        // Load any existing chunk data
                        if let Err(e) = self.viewer.load_chunked_ranges(&self.chunk_storage, &self.explorer.root_dir()) {
                            self.state.set_debug_message(format!("Error loading chunks: {}", e), 3);
                        }
                        
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
        use ratatui::crossterm::event::KeyCode;

        match event.code {
            
            // Exit viewer and return to explorer (q, Esc, h, or left arrow)
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => {
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
                    
                    // Set the file name for the editor (extract from the path)
                    if let Some(file_path) = self.viewer.file_path() {
                        if let Some(file_name) = file_path.file_name() {
                            self.editor.set_file_name(file_name.to_string_lossy().to_string());
                        }
                    }
                    
                    // Set the max tokens from the viewer
                    self.editor.set_max_tokens(self.viewer.max_tokens_per_chunk());
                    
                    // Switch to editor mode
                    self.state.mode = AppMode::Editor;
                    
                    // Clear any existing debug messages to ensure bottom status line is visible
                    self.state.clear_debug_message();
                } else {
                    self.state.set_debug_message("No text selected for editing".to_string(), 2);
                }
            },
            
            // Save chunk with 'S' key
            KeyCode::Char('s') => {
                // Only save if there's a selection
                if let Some((start, end)) = self.viewer.selection_range() {
                    // Check for overlap with existing chunks
                    let has_overlap = self.viewer.check_chunk_overlap(start, end);
                    
                    // If there's an overlap, warn the user but proceed
                    if has_overlap {
                        self.state.set_debug_message(
                            "Warning: Selected text overlaps with existing chunks".to_string(), 
                            2
                        );
                    }
                    
                    // Store the selection range to reference after saving
                    let selection_range = self.viewer.selection_range();
                    
                    // Save the chunk to CSV storage
                    match self.viewer.save_selection_as_chunk(&mut self.chunk_storage, &self.explorer.root_dir()) {
                        Ok(chunk_id) => {
                            // Set cursor to the end of the saved chunk - makes it easier to select next block
                            if let Some((_, end)) = selection_range {
                                let next_line = (end + 1).min(self.viewer.content().len().saturating_sub(1));
                                
                                // Move cursor to the next line after the saved chunk
                                if next_line > end {
                                    // Position cursor at the next line
                                    for _ in 0..(next_line - self.viewer.cursor_position()) {
                                        self.viewer.cursor_down();
                                    }
                                }
                            }
                            
                            // Clear selection after saving
                            self.viewer.clear_selection();
                            let percent = self.viewer.chunking_percentage();
                            
                            // Update the explorer chunking progress
                            if let Some(file_path) = self.viewer.file_path() {
                                self.explorer.update_chunking_progress(file_path, percent);
                            }
                            
                            if has_overlap {
                                self.state.set_debug_message(
                                    format!("Chunk saved with overlaps (ID: {}) ({:.1}% chunked)", 
                                             chunk_id, percent), 
                                    3
                                );
                            } else {
                                self.state.set_debug_message(
                                    format!("Chunk saved (ID: {}) ({:.1}% chunked)", 
                                             chunk_id, percent), 
                                    3
                                );
                            }
                        },
                        Err(e) => {
                            self.state.set_debug_message(format!("Error saving chunk: {}", e), 3);
                        }
                    }
                } else {
                    self.state.set_debug_message("No text selected for chunking".to_string(), 2);
                }
            },
            
            // Line-based cursor movement
            KeyCode::Up | KeyCode::Char('k') => {
                if event.modifiers.contains(event::KeyModifiers::SHIFT) {
                    // Fast scroll - move 5 lines at a time
                    for _ in 0..5 {
                        self.viewer.cursor_up();
                    }
                } else {
                    self.viewer.cursor_up();
                }
            },
            KeyCode::Down | KeyCode::Char('j') => {
                if event.modifiers.contains(event::KeyModifiers::SHIFT) {
                    // Fast scroll - move 5 lines at a time
                    for _ in 0..5 {
                        self.viewer.cursor_down();
                    }
                } else {
                    self.viewer.cursor_down();
                }
            },
            
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
        use ratatui::crossterm::event::KeyCode;
        
        // Special key handling
        match event.code {
            // Handle Q key to exit editor mode (when in normal mode)
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                // Only exit editor if we're in normal mode
                if self.editor.mode() == "NORMAL" {
                    // Warn user if they have unsaved changes
                    if self.editor.is_modified() {
                        self.state.set_debug_message("Exiting editor without saving changes".to_string(), 3);
                    }
                    self.state.mode = AppMode::Viewer;
                } else {
                    // Otherwise, let the editor handle it
                    self.editor.handle_key_event(event);
                }
            },
            
            // Handle Escape key based on editor mode
            KeyCode::Esc => {
                // Only exit editor if we're already in normal mode
                if self.editor.mode() == "NORMAL" {
                    // Warn user if they have unsaved changes
                    if self.editor.is_modified() {
                        self.state.set_debug_message("Exiting editor without saving changes".to_string(), 3);
                    }
                    self.state.mode = AppMode::Viewer;
                } else {
                    // Otherwise, let the editor handle it (to switch from insert/visual to normal mode)
                    self.editor.handle_key_event(event);
                }
            },
            
            // Handle Enter key for Vim commands (e.g., ":wq", ":q!", ":q")
            KeyCode::Enter => {
                // Only process if we're in command mode
                if self.editor.is_in_command_mode() {
                    if self.editor.is_save_command() {
                        // User typed :wq or :x - save the content as a chunk before exiting
                        // Get the edited content
                        let edited_content = self.editor.content();
                        
                        // Check if content was modified
                        let is_modified = self.editor.is_modified();
                        
                        // Update viewer with the edited content if a selection exists
                        if let Some((_start, _end)) = self.viewer.selection_range() {
                            // Replace the selected lines with the edited content
                            if self.viewer.update_selected_content(edited_content) {
                                // Save the updated content as a chunk
                                match self.viewer.save_selection_as_chunk(&mut self.chunk_storage, &self.explorer.root_dir()) {
                                    Ok(chunk_id) => {
                                        // Clear selection after saving
                                        self.viewer.clear_selection();
                                        let percent = self.viewer.chunking_percentage();
                                        
                                        // Update the explorer chunking progress
                                        if let Some(file_path) = self.viewer.file_path() {
                                            self.explorer.update_chunking_progress(file_path, percent);
                                        }
                                        
                                        if is_modified {
                                            self.state.set_debug_message(
                                                format!("Edited content saved (ID: {}) ({:.1}% chunked)", 
                                                         chunk_id, percent), 
                                                3
                                            );
                                        } else {
                                            self.state.set_debug_message(
                                                format!("Chunk saved (ID: {}) ({:.1}% chunked)", 
                                                         chunk_id, percent), 
                                                3
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        self.state.set_debug_message(format!("Error saving chunk: {}", e), 3);
                                    }
                                }
                            } else {
                                // Show error message if replacement failed
                                self.state.set_debug_message("Failed to update content - selection range may be invalid".to_string(), 3);
                            }
                        }
                        
                        // Return to viewer mode
                        self.state.mode = AppMode::Viewer;
                    } else if self.editor.is_quit_command() {
                        // User typed :q - quit without saving if no unsaved changes
                        if self.editor.is_modified() {
                            self.state.set_debug_message("No write since last change (use :q! to override)".to_string(), 3);
                            // Do not exit the editor - pass the Enter key to the editor
                            self.editor.handle_key_event(event);
                            return;
                        } else {
                            // No unsaved changes, exit to viewer mode
                            self.state.mode = AppMode::Viewer;
                        }
                    } else if self.editor.is_force_quit_command() {
                        // User typed :q! - force quit without saving
                        if self.editor.is_modified() {
                            self.state.set_debug_message("Exiting editor without saving changes".to_string(), 3);
                        }
                        self.state.mode = AppMode::Viewer;
                    } else {
                        // Pass the Enter key to the editor for other commands
                        self.editor.handle_key_event(event);
                        return;
                    }
                } else {
                    // Pass the Enter key to the editor if not in command mode
                    self.editor.handle_key_event(event);
                    return;
                }
            },
            
            // Save changes, create chunk, and return to viewer
            KeyCode::Char('s') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Get the edited content
                let edited_content = self.editor.content();
                
                // Reset the modified flag on the editor (to match behavior of :w command)
                let is_modified = self.editor.is_modified();
                
                // Update viewer with the edited content if a selection exists
                if let Some((_start, _end)) = self.viewer.selection_range() {
                    // Store the selection range to reference after saving
                    let selection_range = self.viewer.selection_range();
                    
                    // Replace the selected lines with the edited content
                    if self.viewer.update_selected_content(edited_content) {
                        // Save the updated content as a chunk
                        match self.viewer.save_selection_as_chunk(&mut self.chunk_storage, &self.explorer.root_dir()) {
                            Ok(chunk_id) => {
                                // Set cursor to the end of the saved chunk - makes it easier to select next block
                                if let Some((_, end)) = selection_range {
                                    let next_line = (end + 1).min(self.viewer.content().len().saturating_sub(1));
                                    
                                    // Move cursor to the next line after the saved chunk
                                    if next_line > end {
                                        // Position cursor at the next line
                                        for _ in 0..(next_line - self.viewer.cursor_position()) {
                                            self.viewer.cursor_down();
                                        }
                                    }
                                }
                                
                                // Clear selection after saving
                                self.viewer.clear_selection();
                                let percent = self.viewer.chunking_percentage();
                                
                                if is_modified {
                                    self.state.set_debug_message(
                                        format!("Edited content saved (ID: {}) ({:.1}% chunked)", 
                                                 chunk_id, percent), 
                                        3
                                    );
                                } else {
                                    self.state.set_debug_message(
                                        format!("Chunk saved (ID: {}) ({:.1}% chunked)", 
                                                 chunk_id, percent), 
                                        3
                                    );
                                }
                            },
                            Err(e) => {
                                self.state.set_debug_message(format!("Error saving chunk: {}", e), 3);
                            }
                        }
                    } else {
                        // Show error message if replacement failed
                        self.state.set_debug_message("Failed to update content - selection range may be invalid".to_string(), 3);
                    }
                } else {
                    // This should not normally happen (we'd need a selection to enter editor mode)
                    self.state.set_debug_message("No selection to update".to_string(), 3);
                }
                
                // Switch back to viewer mode
                self.state.mode = AppMode::Viewer;
            },
            
            // Handle the key event with the text editor
            _ => {
                // Let the editor handle the key event
                let handled = self.editor.handle_key_event(event);
                if !handled {
                    // If the editor didn't handle it, check for our custom keys
                    // BUT only if not in insert mode
                    if !self.editor.is_in_insert_mode() {
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
}