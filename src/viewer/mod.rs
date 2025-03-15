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
}

impl Viewer {
    /// Create a new viewer
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: Vec::new(),
            scroll_position: 0,
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
        
        Ok(())
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
    
    /// Scroll up one line
    pub fn scroll_up(&mut self) {
        self.scroll_position = self.scroll_position.saturating_sub(1);
    }
    
    /// Scroll down one line
    pub fn scroll_down(&mut self) {
        if !self.content.is_empty() {
            self.scroll_position = (self.scroll_position + 1).min(self.content.len().saturating_sub(1));
        }
    }
    
    /// Scroll up one page
    pub fn scroll_page_up(&mut self, page_size: usize) {
        self.scroll_position = self.scroll_position.saturating_sub(page_size);
    }
    
    /// Scroll down one page
    pub fn scroll_page_down(&mut self, page_size: usize) {
        if !self.content.is_empty() {
            self.scroll_position = (self.scroll_position + page_size).min(self.content.len().saturating_sub(1));
        }
    }
    
    /// Scroll to the top of the file
    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
    }
    
    /// Scroll to the bottom of the file
    pub fn scroll_to_bottom(&mut self) {
        if !self.content.is_empty() {
            self.scroll_position = self.content.len() - 1;
        }
    }
    
    /// Scroll to a specific position
    pub fn scroll_to_position(&mut self, position: usize) {
        if !self.content.is_empty() {
            self.scroll_position = position.min(self.content.len() - 1);
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