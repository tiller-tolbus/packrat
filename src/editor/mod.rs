use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::{Widget, Style, Color, Modifier};
use edtui::{EditorEventHandler, EditorState, EditorTheme, EditorView, EditorMode, RowIndex};
use crate::utils::tokenizer::count_tokens;

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
    /// The name of the file being edited
    file_name: Option<String>,
    /// Maximum tokens per chunk
    max_tokens: usize,
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
            file_name: None,
            max_tokens: 8192, // Default max tokens, same as default config
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
        
        // Reset command buffer and command mode when opening editor
        self.command_buffer.clear();
        self.command_mode = false;
    }
    
    /// Set the file name for the content being edited
    pub fn set_file_name(&mut self, name: String) {
        self.file_name = Some(name);
    }
    
    /// Get the file name being edited
    pub fn file_name(&self) -> Option<String> {
        self.file_name.clone()
    }
    
    /// Set maximum tokens for this editor session
    pub fn set_max_tokens(&mut self, max_tokens: usize) {
        self.max_tokens = max_tokens;
    }
    
    /// Get the maximum token limit
    pub fn max_tokens(&self) -> usize {
        self.max_tokens
    }
    
    /// Count tokens in the current content
    pub fn token_count(&self) -> usize {
        let content = self.content();
        let text = content.join("\n");
        count_tokens(&text)
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
    
    /// Check if a command is intended to quit without saving
    pub fn is_quit_command(&self) -> bool {
        self.command_buffer == ":q"
    }
    
    /// Check if a command is intended to force quit without saving
    pub fn is_force_quit_command(&self) -> bool {
        self.command_buffer == ":q!"
    }
    
    /// Check if we're in command mode
    pub fn is_in_command_mode(&self) -> bool {
        self.command_mode
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
                } else if self.state.mode == EditorMode::Normal {
                    // In normal mode, let the app handle it
                    return false;
                } else {
                    // For any other modes, handle here
                    self.event_handler.on_key_event(key, &mut self.state);
                    return true;
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
        // Trim any leading colon
        let cmd = command.trim_start_matches(':');
        
        // Parse command components (command and arguments)
        let mut parts = cmd.split_whitespace();
        let cmd_name = parts.next().unwrap_or("");
        
        match cmd_name {
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
            
            // :w - Write (mark as saved)
            "w" => {
                // This would normally save the file, but we're not directly writing files
                // Instead, we just mark the content as no longer modified
                self.modified = false;
                return true;
            },
            
            // :wq or :x - Write and quit
            "wq" | "x" => {
                // Signal app to save and exit
                return false;
            },
            
            // :set - Set options (supporting a subset of Vim's :set commands)
            "set" => {
                // Get the option(s) to set
                let options = parts.collect::<Vec<&str>>();
                for opt in options {
                    match opt {
                        // Common Vim settings that users might try
                        "number" | "nu" => {
                            // Already enabled by default in EdTUI, but would handle here
                        },
                        "nonumber" | "nonu" => {
                            // Would disable line numbers if implemented
                        },
                        "wrap" => {
                            // Already enabled by default, but would handle here
                        },
                        "nowrap" => {
                            // Would disable wrapping if implemented
                        },
                        _ => {
                            // Ignore unknown settings
                        }
                    }
                }
                return true;
            },
            
            // :e - Edit file (not supported in our implementation)
            "e" | "edit" => {
                // We don't support file operations, but handle the command gracefully
                return true;
            },
            
            // :split, :vsplit - Split window (not supported)
            "sp" | "split" | "vs" | "vsplit" => {
                // We don't support splits, but handle gracefully
                return true;
            },
            
            // :h, :help - Show help (would show help in a real Vim)
            "h" | "help" => {
                // We'd show help if implemented
                return true;
            },
            
            // :syntax - Syntax highlighting (not fully implemented)
            "syntax" => {
                // We would handle syntax highlighting settings here
                return true;
            },
            
            // :%s - Substitution (not implemented but commonly used)
            "s" | "%s" => {
                // We'd implement substitutions here
                return true;
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

