use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use walkdir::WalkDir;
use crate::storage::ChunkStorage;

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
    
    /// Initialize chunking progress data from CSV storage
    pub fn init_chunking_progress(&mut self, chunk_storage: &ChunkStorage) -> Result<()> {
        // Get all chunks from storage
        let chunks = chunk_storage.get_chunks();
        
        // If there are no chunks, nothing to do
        if chunks.is_empty() {
            return Ok(());
        }
        
        // Process each file path in the chunks and build a map of file paths to lines
        let mut files_to_process: HashMap<PathBuf, Vec<(usize, usize)>> = HashMap::new();
        
        for chunk in chunks {
            // Get the file path and range
            let file_path = chunk.file_path.clone();
            let start_line = chunk.start_line;
            let end_line = chunk.end_line;
            
            // Add this range to the file's chunks
            files_to_process
                .entry(file_path)
                .or_insert_with(Vec::new)
                .push((start_line, end_line));
        }
        
        // Calculate the chunking progress for each file
        for (file_path, ranges) in files_to_process.iter() {
            // Read the file to count lines
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                let total_lines = content.lines().count();
                
                if total_lines > 0 {
                    // Count unique chunked lines using a boolean vector
                    let mut chunked_lines = vec![false; total_lines];
                    
                    for &(start, end) in ranges {
                        for i in start..=end.min(total_lines - 1) {
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
        
        // Refresh entries with the updated chunking progress
        self.load_entries()?;
        
        Ok(())
    }
    
    // The previous calculate_chunking_progress_for_pattern method is no longer needed
    // as we now directly process chunks from the CSV storage
    
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