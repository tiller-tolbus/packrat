use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Text viewer component
pub struct Viewer {
    /// Current file path
    file_path: Option<PathBuf>,
    /// Content of the current file
    content: Vec<String>,
    /// Current scroll position (line number)
    scroll_position: usize,
    /// Whether selection mode is active
    selection_mode: bool,
    /// The line where selection started
    selection_start: Option<usize>,
    /// The current cursor position (used for selection)
    cursor_position: usize,
}

impl Viewer {
    /// Create a new viewer
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: Vec::new(),
            scroll_position: 0,
            selection_mode: false,
            selection_start: None,
            cursor_position: 0,
        }
    }
    
    /// Open a file in the viewer
    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();
        
        // Open and read the file
        let file = File::open(&path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        
        let reader = BufReader::new(file);
        
        // Read the file line by line
        let mut content = Vec::new();
        for line in reader.lines() {
            let line = line.context("Failed to read line from file")?;
            content.push(line);
        }
        
        // Update viewer state
        self.file_path = Some(path);
        self.content = content;
        self.scroll_position = 0;
        self.cursor_position = 0;
        self.selection_mode = false;
        self.selection_start = None;
        
        Ok(())
    }
    
    /// Toggle selection mode
    pub fn toggle_selection_mode(&mut self) {
        if !self.content.is_empty() {
            if !self.selection_mode {
                // Entering selection mode - set selection start
                self.selection_mode = true;
                self.selection_start = Some(self.cursor_position);
            } else {
                // Exiting selection mode - clear the selection
                self.selection_mode = false;
                self.selection_start = None;
            }
        }
    }
    
    /// Check if selection mode is active
    pub fn is_selection_mode(&self) -> bool {
        self.selection_mode
    }
    
    /// Get the current cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }
    
    /// Get the current selection range (if any)
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            let end = self.cursor_position;
            if start <= end {
                (start, end)
            } else {
                (end, start)
            }
        })
    }
    
    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selection_mode = false;
        self.selection_start = None;
    }
    
    /// Get the current file path
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }
    
    /// Get the file content
    pub fn content(&self) -> &[String] {
        &self.content
    }
    
    /// Get the current scroll position
    pub fn scroll_position(&self) -> usize {
        self.scroll_position
    }
    
    /// Move cursor up one line
    pub fn cursor_up(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
        
        // Ensure cursor is visible by scrolling if needed
        if self.cursor_position < self.scroll_position {
            self.scroll_position = self.cursor_position;
        }
    }
    
    /// Move cursor down one line
    pub fn cursor_down(&mut self) {
        if !self.content.is_empty() {
            self.cursor_position = (self.cursor_position + 1).min(self.content.len().saturating_sub(1));
            
            // Ensure cursor is visible by scrolling if needed
            if self.cursor_position >= self.scroll_position + 20 { // Arbitrary threshold assuming 20 visible lines
                self.scroll_position = (self.cursor_position - 19).min(self.content.len().saturating_sub(1));
            }
        }
    }
    
    /// Scroll up one line
    pub fn scroll_up(&mut self) {
        self.scroll_position = self.scroll_position.saturating_sub(1);
        
        // If cursor is above scroll position, move it too
        if self.cursor_position > self.scroll_position + 20 { // Arbitrary threshold
            self.cursor_position = self.cursor_position.saturating_sub(1);
        }
    }
    
    /// Scroll down one line
    pub fn scroll_down(&mut self) {
        if !self.content.is_empty() {
            self.scroll_position = (self.scroll_position + 1).min(self.content.len().saturating_sub(1));
            
            // If cursor falls off visible area, move it too
            if self.cursor_position < self.scroll_position {
                self.cursor_position = self.scroll_position;
            }
        }
    }
    
    /// Scroll up one page
    pub fn scroll_page_up(&mut self, page_size: usize) {
        let old_position = self.scroll_position;
        self.scroll_position = self.scroll_position.saturating_sub(page_size);
        
        // Move cursor by the same amount scroll moved, up to the current scrolling position
        let scroll_delta = old_position - self.scroll_position;
        self.cursor_position = self.cursor_position.saturating_sub(scroll_delta).max(self.scroll_position);
    }
    
    /// Scroll down one page
    pub fn scroll_page_down(&mut self, page_size: usize) {
        if !self.content.is_empty() {
            let old_position = self.scroll_position;
            self.scroll_position = (self.scroll_position + page_size).min(self.content.len().saturating_sub(1));
            
            // Move cursor by the same amount scroll moved, but stay within the file boundary
            let scroll_delta = self.scroll_position - old_position;
            if scroll_delta > 0 {
                self.cursor_position = (self.cursor_position + scroll_delta).min(self.content.len().saturating_sub(1));
            }
        }
    }
    
    /// Scroll to the top of the file
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
        self.cursor_position = 0;
    }
    
    /// Scroll to the bottom of the file
    pub fn scroll_to_bottom(&mut self) {
        if !self.content.is_empty() {
            // For compatibility with tests, set scroll position to content size - 1
            self.scroll_position = self.content.len() - 1;
            self.cursor_position = self.content.len() - 1;
        }
    }
    
    /// Scroll to a specific position
    pub fn scroll_to_position(&mut self, position: usize) {
        if !self.content.is_empty() {
            self.scroll_position = position.min(self.content.len() - 1);
            // Only adjust cursor if it falls out of view
            if self.cursor_position < self.scroll_position {
                self.cursor_position = self.scroll_position;
            } else if self.cursor_position > self.scroll_position + 20 { // Arbitrary threshold
                self.cursor_position = self.scroll_position + 20;
            }
        }
    }
    
    /// Get the visible content for display
    pub fn visible_content(&self, height: usize) -> Vec<&String> {
        if self.content.is_empty() {
            return Vec::new();
        }
        
        // Calculate the visible range
        let start = self.scroll_position;
        let end = (start + height).min(self.content.len());
        
        // Return sliced content
        self.content[start..end].iter().collect()
    }
}