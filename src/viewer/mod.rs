use anyhow::{Context, Result, anyhow};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use crate::utils::generate_chunk_filename;

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
    /// Ranges of lines that have been chunked (start, end)
    chunked_ranges: Vec<(usize, usize)>,
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
            chunked_ranges: Vec::new(),
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
        self.chunked_ranges = Vec::new();
        
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
    
    /// Save current selection as a chunk
    pub fn save_selection_as_chunk(&mut self, chunk_dir: &Path, root_dir: &Path) -> Result<PathBuf> {
        // Get selected range
        let range = self.selection_range().ok_or_else(|| anyhow!("No text selected"))?;
        
        // Check if the selection is valid
        if range.0 >= self.content.len() || range.1 >= self.content.len() {
            return Err(anyhow!("Invalid selection range"));
        }
        
        // Extract the lines from the current in-memory content (which may have been edited)
        let selected_content = &self.content[range.0..=range.1];
        
        // Create chunk filename
        let file_path = self.file_path().ok_or_else(|| anyhow!("No file opened"))?;
        let chunk_filename = generate_chunk_filename(file_path, root_dir, range.0, range.1);
        let chunk_path = chunk_dir.join(chunk_filename);
        
        // Ensure chunk directory exists
        fs::create_dir_all(chunk_dir)?;
        
        // Write chunk to file
        let mut file = File::create(&chunk_path)?;
        for line in selected_content {
            writeln!(file, "{}", line)?;
        }
        
        // Add to chunked ranges
        self.chunked_ranges.push(range);
        
        Ok(chunk_path)
    }
    
    /// Check if a line is part of a saved chunk
    pub fn is_line_chunked(&self, line_number: usize) -> bool {
        self.chunked_ranges.iter().any(|(start, end)| {
            line_number >= *start && line_number <= *end
        })
    }
    
    /// Get all chunked ranges
    pub fn chunked_ranges(&self) -> &[(usize, usize)] {
        &self.chunked_ranges
    }
    
    /// Calculate the percentage of file that has been chunked
    pub fn chunking_percentage(&self) -> f64 {
        if self.content.is_empty() {
            return 0.0;
        }
        
        // Count unique lines that are chunked
        let mut chunked_lines = vec![false; self.content.len()];
        
        for (start, end) in &self.chunked_ranges {
            for i in *start..=*end {
                if i < chunked_lines.len() {
                    chunked_lines[i] = true;
                }
            }
        }
        
        // Calculate percentage
        let total_chunked = chunked_lines.iter().filter(|&&chunked| chunked).count();
        (total_chunked as f64 / self.content.len() as f64) * 100.0
    }
    
    /// Update the selected text content with edited content
    pub fn update_selected_content(&mut self, edited_content: Vec<String>) -> bool {
        // Get the selection range
        if let Some((start, end)) = self.selection_range() {
            // Validate the range is within bounds
            if start >= self.content.len() || end >= self.content.len() {
                return false;
            }
            
            // Replace content in the selected range
            let range_len = end - start + 1;
            let replacement_len = edited_content.len();
            
            // Remove the selected lines and insert the edited content
            self.content.splice(start..=end, edited_content);
            
            // If the number of lines has changed, we need to adjust chunked ranges
            if range_len != replacement_len {
                let line_diff = replacement_len as isize - range_len as isize;
                
                // Update chunked ranges that come after the edit
                for i in 0..self.chunked_ranges.len() {
                    let (chunk_start, chunk_end) = self.chunked_ranges[i];
                    
                    // If the chunk is entirely after the edit, shift it
                    if chunk_start > end {
                        self.chunked_ranges[i] = (
                            (chunk_start as isize + line_diff) as usize,
                            (chunk_end as isize + line_diff) as usize
                        );
                    }
                    // If the chunk overlaps with the edit, we might need more complex logic
                    // For now, we'll consider those chunks invalid and remove them
                    else if chunk_end >= start {
                        // Mark for removal
                        self.chunked_ranges[i] = (0, 0);
                    }
                }
                
                // Remove invalid chunks (those marked as (0,0))
                self.chunked_ranges.retain(|&range| range != (0, 0));
            }
            
            // Update cursor position if needed (e.g., if content shrinks)
            if self.cursor_position >= self.content.len() {
                self.cursor_position = self.content.len().saturating_sub(1);
            }
            
            return true;
        }
        
        false
    }
}