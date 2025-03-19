use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::{Widget, Style, Color, Modifier};
use edtui::{EditorEventHandler, EditorState, EditorTheme, EditorView, EditorMode, RowIndex};

/// Text editor component 
pub struct Editor {
    /// EdTUI editor state
    state: EditorState,
    /// Event handler for key events
    event_handler: EditorEventHandler,
    /// Whether the content has been modified
    modified: bool,
    /// Original content for modification detection
    original_content: Vec<String>,
    /// Command buffer for Vim commands (e.g. ":wq")
    command_buffer: String,
    /// Whether we're in command mode (after typing ":")
    command_mode: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    /// Create a new editor
    pub fn new() -> Self {
        Self {
            state: EditorState::default(),
            event_handler: EditorEventHandler::default(),
            modified: false,
            original_content: Vec::new(),
            command_buffer: String::new(),
            command_mode: false,
        }
    }
    
    /// Initialize the editor with selected lines from the viewer
    pub fn set_content(&mut self, lines: Vec<String>) {
        // Create a new editor state
        let mut new_state = EditorState::default();
        
        // EdTUI handles content internally - we need to add each line as a vec<char>
        for line in &lines {
            let char_vec: Vec<char> = line.chars().collect();
            new_state.lines.push(char_vec);
        }
        
        self.state = new_state;
        self.original_content = lines;
        self.modified = false;
    }
    
    /// Get the current content as lines
    pub fn content(&self) -> Vec<String> {
        // Convert the Jagged<char> structure back to Vec<String>
        let mut result = Vec::new();
        let num_rows = self.state.lines.len();
        
        for i in 0..num_rows {
            // Use RowIndex to access rows in the Jagged structure
            if let Some(row) = self.state.lines.get(RowIndex::new(i)) {
                // Specify that we're collecting characters into a String
                let line: String = row.iter().collect::<String>();
                result.push(line);
            }
        }
        
        result
    }
    
    /// Check if the content has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    
    /// Get a string representation of the current mode
    pub fn mode(&self) -> String {
        if self.command_mode {
            self.command_buffer.clone()
        } else {
            match self.state.mode {
                EditorMode::Normal => "NORMAL".to_string(),
                EditorMode::Insert => "INSERT".to_string(),
                EditorMode::Visual => "VISUAL".to_string(),
                _ => "UNKNOWN".to_string(),
            }
        }
    }
    
    /// Check if a command is intended to save content
    pub fn is_save_command(&self) -> bool {
        self.command_buffer == ":wq" || self.command_buffer == ":x"
    }
    
    /// Handle key event and update the modified flag if content changes
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        // If we're in command mode (after typing ":" in normal mode)
        if self.command_mode {
            match key.code {
                // Escape exits command mode
                KeyCode::Esc => {
                    self.command_mode = false;
                    self.command_buffer.clear();
                    return true;
                },
                
                // Enter processes the command
                KeyCode::Enter => {
                    let command = self.command_buffer.clone();
                    let command = command.trim();
                    let result = self.process_command(command);
                    self.command_mode = false;
                    self.command_buffer.clear();
                    return result;
                },
                
                // Backspace removes characters
                KeyCode::Backspace => {
                    self.command_buffer.pop();
                    return true;
                },
                
                // Add typed characters to command buffer
                KeyCode::Char(c) => {
                    self.command_buffer.push(c);
                    return true;
                },
                
                // Ignore other keys
                _ => return true,
            }
        }
        
        // Special handling for specific keys
        match key.code {
            // For Ctrl+S, handle at application level
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return false;
            }
            
            // For ? (help key), handle at application level
            KeyCode::Char('?') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                return false;
            },
            
            // Handle colon key (enter command mode) in Normal mode
            KeyCode::Char(':') if self.state.mode == EditorMode::Normal => {
                self.command_mode = true;
                self.command_buffer.clear();
                self.command_buffer.push(':');
                return true;
            },
            
            // Handle Escape key specially
            KeyCode::Esc => {
                // Let EdTUI handle Esc for mode changes
                if matches!(self.state.mode, EditorMode::Insert | EditorMode::Visual) {
                    self.event_handler.on_key_event(key, &mut self.state);
                    return true;
                } else {
                    // In normal mode, exit the editor
                    return false;
                }
            }
            
            // Let EdTUI handle other keys
            _ => {
                // Track content changes by checking before and after
                let content_before = self.content();
                
                // Let EdTUI handle the key event
                self.event_handler.on_key_event(key, &mut self.state);
                
                // Check if content has changed
                let content_after = self.content();
                if content_before != content_after {
                    self.modified = true;
                }
                
                return true;
            }
        }
    }
    
    /// Process a command entered in command mode (after typing ":")
    fn process_command(&mut self, command: &str) -> bool {
        match command.trim_start_matches(':') {
            // :q - Quit without saving
            "q" => {
                // If there are unsaved changes, don't quit
                if self.modified {
                    // In a real Vim implementation, we'd show a message like
                    // "No write since last change (add ! to override)"
                    return true;
                }
                // Signal app to exit the editor
                return false;
            },
            
            // :q! - Force quit without saving
            "q!" => {
                // Signal app to exit the editor
                return false;
            },
            
            // :w - Write (in our case, does nothing since we don't write until exit)
            "w" => {
                // This would normally save the file, but we're not directly writing files
                self.modified = false;
                return true;
            },
            
            // :wq or :x - Write and quit
            "wq" | "x" => {
                // Signal app to save and exit
                return false;
            },
            
            // Unknown command - would normally show an error in Vim
            _ => {
                // For now, just ignore unknown commands
                return true;
            }
        }
    }
    
    /// Get editor view for rendering
    pub fn view<'a, 'b>(&'a mut self) -> EditorView<'a, 'b> {
        // Create a theme with proper Vim-like cursor styling
        let theme = EditorTheme::default()
            // Use a block cursor (high contrast reversal) for normal mode
            .cursor_style(Style::default().bg(Color::White).fg(Color::Black).add_modifier(Modifier::BOLD))
            // Keep the selection style but make it more prominent
            .selection_style(Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD));
            
        EditorView::new(&mut self.state)
            .theme(theme)
            .wrap(true)
    }
    
    /// Render the editor directly to the buffer
    pub fn render_to_buffer(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        // Create the editor view and render it directly
        let view = self.view();
        Widget::render(view, area, buf);
    }
    
    /// For compatibility with the UI code - always returns None as we don't use TextArea
    pub fn textarea(&self) -> Option<&()> {
        None
    }
}

