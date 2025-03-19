use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::WalkDir;

/// Representation of a directory entry
#[derive(Clone)]
pub struct DirectoryEntry {
    /// Name of the entry
    pub name: String,
    /// Full path of the entry
    pub path: PathBuf,
    /// Whether the entry is a directory
    pub is_dir: bool,
    /// Chunking progress percentage (0-100)
    pub chunking_progress: f64,
}

/// File explorer component
pub struct Explorer {
    /// Current directory path
    current_dir: PathBuf,
    /// Root directory path (can't navigate above this)
    root_dir: PathBuf,
    /// List of entries in the current directory
    entries: Vec<DirectoryEntry>,
    /// Currently selected entry index
    selected_index: usize,
    /// Cache of chunking progress by file path
    chunking_progress: HashMap<PathBuf, f64>,
}

impl Explorer {
    /// Create a new explorer with the given root directory
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Self> {
        let root_dir = PathBuf::from(root_dir.as_ref())
            .canonicalize()
            .context("Failed to canonicalize root directory")?;
        
        let current_dir = root_dir.clone();
        
        let mut explorer = Self {
            current_dir,
            root_dir,
            entries: Vec::new(),
            selected_index: 0,
            chunking_progress: HashMap::new(),
        };
        
        // Load initial entries
        explorer.load_entries()?;
        
        Ok(explorer)
    }
    
    /// Initialize chunking progress data by scanning chunks directory
    pub fn init_chunking_progress(&mut self, chunk_dir: &Path) -> Result<()> {
        // If the chunks directory doesn't exist, there are no chunks
        if !chunk_dir.exists() {
            return Ok(());
        }
        
        // Create a map of file path patterns to their chunk ranges
        let mut file_chunks: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
        
        // Iterate over all files in the chunks directory
        for entry in std::fs::read_dir(chunk_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Skip non-txt chunk files
            if path.extension().map_or(false, |ext| ext == "txt") {
                let filename = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
                
                // Parse the filename to extract file path and chunk range
                // Format is: path_from_root_converted_to_underscores_START-END.txt
                if let Some(underscore_pos) = filename.rfind('_') {
                    if let Some(range_part) = filename.get(underscore_pos + 1..).and_then(|s| s.strip_suffix(".txt")) {
                        if let Some((start_str, end_str)) = range_part.split_once('-') {
                            if let (Ok(start), Ok(end)) = (start_str.parse::<usize>(), end_str.parse::<usize>()) {
                                // Adjust from 1-indexed (in filename) to 0-indexed
                                let start = start.saturating_sub(1);
                                let end = end.saturating_sub(1);
                                
                                // Now we need to convert the path part back to a real path
                                let path_part = &filename[0..underscore_pos];
                                
                                // Add this range to the file's chunks
                                file_chunks.entry(path_part.to_string())
                                    .or_insert_with(Vec::new)
                                    .push((start, end));
                            }
                        }
                    }
                }
            }
        }
        
        // Process each file pattern and update chunking progress
        for (path_pattern, ranges) in file_chunks {
            self.calculate_chunking_progress_for_pattern(&path_pattern, ranges)?;
        }
        
        // Refresh entries with the updated chunking progress
        self.load_entries()?;
        
        Ok(())
    }
    
    /// Calculate chunking progress for a file matching the given pattern
    fn calculate_chunking_progress_for_pattern(&mut self, path_pattern: &str, ranges: Vec<(usize, usize)>) -> Result<()> {
        // Find the file that matches this pattern
        let mut matched_files = Vec::new();
        
        for entry in walkdir::WalkDir::new(&self.root_dir) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            
            // Only consider files
            if !entry.file_type().is_file() {
                continue;
            }
            
            let path = entry.path();
            
            // Skip if it's a chunk file
            if path.to_string_lossy().contains("chunks/") {
                continue;
            }
            
            // Get the relative path and sanitize it
            let relative_path = match path.strip_prefix(&self.root_dir) {
                Ok(rel_path) => rel_path,
                Err(_) => continue,
            };
            
            // Convert to string and sanitize the same way the chunk filename was created
            let path_str = relative_path.to_string_lossy();
            let sanitized_path = path_str
                .replace(['/', '\\'], "_")
                .replace(['.', ' ', '-', ':', '+'], "_")
                .trim_start_matches('_')
                .to_string();
            
            // Check if this file matches the pattern
            if sanitized_path == path_pattern {
                matched_files.push(path.to_path_buf());
            }
        }
        
        // Update chunking progress for each matched file
        for file_path in matched_files {
            // Read the file to count lines
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let total_lines = content.lines().count();
                
                if total_lines > 0 {
                    // Count unique chunked lines using a boolean vector
                    let mut chunked_lines = vec![false; total_lines];
                    
                    for (start, end) in &ranges {
                        for i in *start..=(*end).min(total_lines - 1) {
                            chunked_lines[i] = true;
                        }
                    }
                    
                    // Calculate percentage
                    let chunked_count = chunked_lines.iter().filter(|&&chunked| chunked).count();
                    let percentage = (chunked_count as f64 / total_lines as f64) * 100.0;
                    
                    // Update the chunking progress
                    self.update_chunking_progress(&file_path, percentage);
                }
            }
        }
        
        Ok(())
    }
    
    /// Reload entries in the current directory
    fn load_entries(&mut self) -> Result<()> {
        self.entries.clear();
        self.selected_index = 0;
        
        // Add parent directory entry if not at root
        if self.current_dir != self.root_dir {
            self.entries.push(DirectoryEntry {
                name: "..".to_string(),
                path: self.current_dir.join(".."),
                is_dir: true,
                chunking_progress: 0.0,
            });
        }
        
        // Add entries from current directory
        for entry in WalkDir::new(&self.current_dir)
            .max_depth(1)
            .min_depth(1)
            .sort_by_file_name()
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path().to_path_buf();
            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string();
            let is_dir = entry.file_type().is_dir();
            
            // Get chunking progress if we have it cached
            let chunking_progress = if !is_dir {
                *self.chunking_progress.get(&path).unwrap_or(&0.0)
            } else {
                0.0
            };
            
            self.entries.push(DirectoryEntry {
                name,
                path,
                is_dir,
                chunking_progress,
            });
        }
        
        // Sort directories first, then files
        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        Ok(())
    }
    
    /// Get a reference to the entries
    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }
    
    /// Get the current directory path
    pub fn current_path(&self) -> &Path {
        &self.current_dir
    }
    
    /// Get the root directory path
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }
    
    /// Get the current selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
    
    /// Select the previous entry
    pub fn select_previous(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }
    
    /// Select the next entry
    pub fn select_next(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.entries.len() - 1);
        }
    }
    
    /// Select the entry one page up (or to the top if less than a page)
    pub fn select_page_up(&mut self, page_size: usize) {
        if !self.entries.is_empty() {
            self.selected_index = self.selected_index.saturating_sub(page_size);
        }
    }
    
    /// Select the entry one page down (or to the bottom if less than a page)
    pub fn select_page_down(&mut self, page_size: usize) {
        if !self.entries.is_empty() {
            self.selected_index = (self.selected_index + page_size).min(self.entries.len() - 1);
        }
    }
    
    /// Select the first entry
    pub fn select_first(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = 0;
        }
    }
    
    /// Select the last entry
    pub fn select_last(&mut self) {
        if !self.entries.is_empty() {
            self.selected_index = self.entries.len() - 1;
        }
    }
    
    /// Open the selected entry (directory only)
    pub fn open_selected(&mut self) -> Result<()> {
        if self.entries.is_empty() {
            return Ok(());
        }
        
        let selected = &self.entries[self.selected_index];
        
        if selected.is_dir {
            // Change to the selected directory
            self.current_dir = selected.path.clone();
            self.load_entries()?;
        }
        // File handling is now done in the App struct
        
        Ok(())
    }
    
    /// Go to the parent directory
    pub fn go_to_parent(&mut self) -> Result<()> {
        // Don't go above the root directory
        if self.current_dir == self.root_dir {
            return Ok(());
        }
        
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.load_entries()?;
        }
        
        Ok(())
    }
    
    /// Update the chunking progress for a file
    pub fn update_chunking_progress(&mut self, file_path: &Path, progress: f64) {
        self.chunking_progress.insert(file_path.to_path_buf(), progress);
        
        // Update the entry if it's in the current view
        for entry in &mut self.entries {
            if entry.path == file_path {
                entry.chunking_progress = progress;
                break;
            }
        }
    }
    
    /// Get the chunking progress for a file
    pub fn get_chunking_progress(&self, file_path: &Path) -> f64 {
        *self.chunking_progress.get(file_path).unwrap_or(&0.0)
    }
}