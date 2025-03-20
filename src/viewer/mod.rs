use anyhow::{Context, Result, anyhow};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use crate::utils::{generate_chunk_filename, count_tokens, count_tokens_in_lines, format_token_count};

/// Text viewer component
pub struct Viewer {
    /// Current file path
    file_path: Option<PathBuf>,
    /// Content of the current file
    content: Vec<String>,
    /// Original content of the current file (used to track if content was edited)
    original_content: Vec<String>,
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
    /// Whether the current selection contains edited content
    has_edited_content: bool,
    /// Total token count for the entire file
    total_tokens: usize,
    /// Token counts per line
    tokens_per_line: Vec<usize>,
    /// Maximum tokens allowed per chunk (configurable)
    max_tokens_per_chunk: usize,
}

impl Viewer {
    /// Create a new viewer
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: Vec::new(),
            original_content: Vec::new(),
            scroll_position: 0,
            selection_mode: false,
            selection_start: None,
            cursor_position: 0,
            chunked_ranges: Vec::new(),
            has_edited_content: false,
            total_tokens: 0,
            tokens_per_line: Vec::new(),
            max_tokens_per_chunk: 8192, // Default max tokens (configurable)
        }
    }
    
    /// Set the maximum tokens per chunk
    pub fn set_max_tokens_per_chunk(&mut self, max_tokens: usize) {
        self.max_tokens_per_chunk = max_tokens;
    }
    
    /// Get the maximum tokens per chunk
    pub fn max_tokens_per_chunk(&self) -> usize {
        self.max_tokens_per_chunk
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
        self.file_path = Some(path.clone());
        self.content = content.clone();
        self.original_content = content;
        self.scroll_position = 0;
        self.cursor_position = 0;
        self.selection_mode = false;
        self.selection_start = None;
        self.chunked_ranges = Vec::new();
        self.has_edited_content = false;
        
        // Count tokens
        self.update_token_counts();
        
        // Load existing chunks for this file if any exist
        // Note: This is a placeholder - to fully implement this would require passing the chunk_dir
        // as a parameter to open_file, which would require changing the method signature.
        // For now, we'll leave it as a placeholder.
        
        Ok(())
    }
    
    /// Update token counts for the entire file and per line
    fn update_token_counts(&mut self) {
        // Count tokens for the whole file
        self.total_tokens = count_tokens_in_lines(&self.content);
        
        // Count tokens per line
        self.tokens_per_line = Vec::with_capacity(self.content.len());
        for line in &self.content {
            self.tokens_per_line.push(count_tokens(line));
        }
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
    
    /// Get token count for the current selection
    pub fn selection_token_count(&self) -> Option<usize> {
        self.selection_range().map(|(start, end)| {
            if start >= self.content.len() || end >= self.content.len() {
                0
            } else {
                let selected_lines = &self.content[start..=end];
                count_tokens_in_lines(selected_lines)
            }
        })
    }
    
    /// Check if the current selection exceeds the token limit
    pub fn selection_exceeds_token_limit(&self) -> bool {
        if let Some(token_count) = self.selection_token_count() {
            token_count > self.max_tokens_per_chunk
        } else {
            false
        }
    }
    
    /// Get token count for the entire file
    pub fn total_token_count(&self) -> usize {
        self.total_tokens
    }
    
    /// Get a formatted string with token count for the current selection
    pub fn formatted_selection_token_count(&self) -> String {
        if let Some(count) = self.selection_token_count() {
            let percentage = if self.max_tokens_per_chunk > 0 {
                (count as f64 / self.max_tokens_per_chunk as f64) * 100.0
            } else {
                0.0
            };
            
            if percentage > 100.0 {
                format!("{} ({}% OVER LIMIT!)", format_token_count(count), percentage as usize)
            } else {
                format!("{} ({}%)", format_token_count(count), percentage as usize)
            }
        } else {
            "No selection".to_string()
        }
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
    pub fn visible_content(&self, height: usize) -> Vec<String> {
        if self.content.is_empty() {
            return Vec::new();
        }
        
        // Calculate the visible range
        let start = self.scroll_position;
        let end = (start + height).min(self.content.len());
        
        // Return sliced content without modifying empty lines
        self.content[start..end].to_vec()
    }
    
    /// Check if a line is empty or contains only whitespace
    pub fn is_whitespace_line(&self, line_index: usize) -> bool {
        if line_index >= self.content.len() {
            return false;
        }
        self.content[line_index].trim().is_empty()
    }
    
    /// Save current selection as a chunk
    pub fn save_selection_as_chunk(&mut self, chunk_dir: &Path, root_dir: &Path) -> Result<PathBuf> {
        // Get selected range
        let range = self.selection_range().ok_or_else(|| anyhow!("No text selected"))?;
        
        // Check if the selection is valid
        if range.0 >= self.content.len() || range.1 >= self.content.len() {
            return Err(anyhow!("Invalid selection range"));
        }
        
        // Check for overlap with existing chunks
        let _has_overlap = self.check_chunk_overlap(range.0, range.1);
        
        // Extract the lines from the current in-memory content (which may have been edited)
        let selected_content = &self.content[range.0..=range.1];
        
        // Create chunk filename
        let file_path = self.file_path().ok_or_else(|| anyhow!("No file opened"))?;
        // We don't use this value directly, only for creating chunk filename through the helper
        let _relative_path = if file_path.starts_with(root_dir) {
            match file_path.strip_prefix(root_dir) {
                Ok(rel_path) => rel_path.to_path_buf(),
                Err(_) => file_path.to_path_buf(),
            }
        } else {
            file_path.to_path_buf()
        };
        
        // Check if content has been edited
        let _was_edited = self.has_edited_content;
        
        // Everything we need is in the filename - no need for separate metadata
        
        // Generate chunk filename
        let chunk_filename = generate_chunk_filename(file_path, root_dir, range.0, range.1);
        let chunk_path = chunk_dir.join(chunk_filename);
        
        // Ensure chunk directory exists
        fs::create_dir_all(chunk_dir)?;
        
        // Write chunk to file
        let mut file = File::create(&chunk_path)?;
        for line in selected_content {
            writeln!(file, "{}", line)?;
        }
        
        // We're not using metadata files anymore - the filename contains all we need
        
        // Add to chunked ranges
        self.chunked_ranges.push(range);
        
        // Return chunk path and overlap status
        Ok(chunk_path)
    }
    
    /// Check if a range overlaps with existing chunks
    pub fn check_chunk_overlap(&self, start_line: usize, end_line: usize) -> bool {
        for (chunk_start, chunk_end) in &self.chunked_ranges {
            // Check for any overlap
            if !(end_line < *chunk_start || start_line > *chunk_end) {
                return true;
            }
        }
        false
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
    
    /// Load chunked ranges by parsing chunk filenames
    pub fn load_chunked_ranges(&mut self, chunk_dir: &Path, root_dir: &Path) -> Result<()> {
        // Only proceed if we have a file path
        let file_path = match &self.file_path {
            Some(path) => path.clone(),
            None => return Ok(()),
        };
        
        // Clear existing ranges
        self.chunked_ranges.clear();
        
        // Exit early if chunk directory doesn't exist
        if !chunk_dir.exists() {
            return Ok(());
        }
        
        // Generate the file prefix we're looking for
        // First convert the path to be relative to the root
        let relative_path = if file_path.starts_with(root_dir) {
            match file_path.strip_prefix(root_dir) {
                Ok(rel_path) => rel_path.to_path_buf(),
                Err(_) => file_path.clone(),
            }
        } else {
            file_path.clone()
        };
        
        // Convert path separators and special characters to underscores (same as in generate_chunk_filename)
        let path_str = relative_path.to_string_lossy();
        let sanitized_path = path_str
            .replace(['/', '\\'], "_") // Replace path separators with underscores
            .replace(['.', ' ', '-', ':', '+'], "_"); // Replace other special characters
        
        // Remove leading underscore if present (from absolute paths)
        let sanitized_path = sanitized_path.trim_start_matches('_');
        
        // Handle empty path
        let file_prefix = if sanitized_path.is_empty() {
            "unnamed_file".to_string()
        } else {
            sanitized_path.to_string()
        };
        
        // Now check all files in the chunk directory
        for entry in fs::read_dir(chunk_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only look at .txt files
            if path.extension().map_or(false, |ext| ext == "txt") {
                // Get the filename as a string
                let filename = path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                
                // Check if the filename starts with our prefix
                if filename.starts_with(&file_prefix) {
                    // The filename format is: path_from_root_converted_to_underscores_START-END.txt
                    // Extract the START-END part
                    if let Some(range_part) = filename.strip_prefix(&format!("{}_", file_prefix)) {
                        if let Some(range_part) = range_part.strip_suffix(".txt") {
                            // Parse the range values (START-END)
                            if let Some((start_str, end_str)) = range_part.split_once('-') {
                                if let (Ok(start), Ok(end)) = (start_str.parse::<usize>(), end_str.parse::<usize>()) {
                                    // Adjust from 1-indexed (in filename) to 0-indexed (in code)
                                    let start = start.saturating_sub(1);
                                    let end = end.saturating_sub(1);
                                    
                                    // Add the range
                                    self.chunked_ranges.push((start, end));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
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
            
            // Check if content has actually been edited by comparing with original
            let original_selection = &self.original_content[start..=end];
            let original_slice: Vec<&String> = original_selection.iter().collect();
            let edited_slice: Vec<&String> = edited_content.iter().collect();
            
            self.has_edited_content = original_slice.len() != edited_content.len() || 
                original_slice.iter().zip(edited_slice.iter()).any(|(a, b)| *a != *b);
            
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
    
    /// Check if the selected content has been edited
    pub fn has_edited_content(&self) -> bool {
        self.has_edited_content
    }
}