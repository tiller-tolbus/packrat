use crossterm::event::{self, KeyCode, KeyModifiers};
use tui_textarea::TextArea;

/// Text editor component
pub struct Editor {
    /// Text area widget for editing
    textarea: TextArea<'static>,
    /// Whether the content has been modified
    modified: bool,
}

impl Editor {
    /// Create a new editor
    pub fn new() -> Self {
        Self {
            textarea: TextArea::new(Vec::new()),
            modified: false,
        }
    }
    
    /// Initialize the editor with selected lines from the viewer
    pub fn set_content(&mut self, lines: Vec<String>) {
        self.textarea = TextArea::new(lines);
        self.modified = false;
    }
    
    /// Get the current content as lines
    pub fn content(&self) -> Vec<String> {
        self.textarea.lines().to_vec()
    }
    
    /// Check if the content has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }
    
    /// Handle key event and update the modified flag if content changes
    pub fn handle_key_event(&mut self, key: event::KeyEvent) -> bool {
        let old_content = self.content();
        
        // For Ctrl+S, we don't want the TextArea to handle it
        if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }
        
        // For ? (help key), we don't want the TextArea to handle it
        if key.code == KeyCode::Char('?') && !key.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        }
        
        // Handle Escape key at application level
        if key.code == KeyCode::Esc {
            return false;
        }
        
        // Let TextArea handle the input directly - it has built-in handling for most editing operations
        let handled = self.textarea.input(key);
        
        if handled && old_content != self.content() {
            self.modified = true;
        }
        
        handled
    }
    
    /// Get a reference to the text area for rendering
    pub fn textarea(&self) -> &TextArea<'static> {
        &self.textarea
    }
    
    /// Get a mutable reference to the text area for editing
    pub fn textarea_mut(&mut self) -> &mut TextArea<'static> {
        &mut self.textarea
    }
}