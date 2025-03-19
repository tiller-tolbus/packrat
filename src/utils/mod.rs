// Utility functions for the application
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::fs::{self, File};
use std::io::{self, Write};
use anyhow::{Result, Context};

/// Truncate a string to a maximum length, adding ellipsis if truncated
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Make sure we have room for the ellipsis
        if max_len < 3 {
            // For very short max_len, just truncate without ellipsis
            return s.chars().take(max_len).collect();
        }
        
        // Truncate and add ellipsis
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

/// Generate a chunk filename from a file path, start line, and end line
/// 
/// The filename follows the format: path_from_root_converted_to_underscores_START-END.txt
/// 
/// For example:
/// - /foo/bar.py with lines 3-10 becomes foo_bar_py_3-10.txt
/// - /projects/example/data.csv with lines 15-20 becomes projects_example_data_csv_15-20.txt
/// 
/// All paths are relative to the root directory provided to the function.
/// 
/// # Arguments
/// * `file_path` - Path to the file (absolute or relative)
/// * `root_path` - Root directory path for the application
/// * `start_line` - Starting line number (0-indexed internally, converted to 1-indexed for filename)
/// * `end_line` - Ending line number (0-indexed internally, converted to 1-indexed for filename)
/// 
/// # Returns
/// The generated chunk filename as a String
pub fn generate_chunk_filename(file_path: &std::path::Path, root_path: &std::path::Path, start_line: usize, end_line: usize) -> String {
    // Convert file_path to be relative to root_path
    let relative_path = if file_path.starts_with(root_path) {
        match file_path.strip_prefix(root_path) {
            Ok(rel_path) => rel_path,
            Err(_) => file_path,
        }
    } else {
        file_path
    };
    
    // Convert path separators and special characters to underscores
    let path_str = relative_path.to_string_lossy();
    let sanitized_path = path_str
        .replace(['/', '\\'], "_") // Replace path separators with underscores
        .replace(['.', ' ', '-', ':', '+'], "_"); // Replace other special characters
    
    // Remove leading underscore if present (from absolute paths)
    let sanitized_path = sanitized_path.trim_start_matches('_');
    
    // Handle empty path (should be impossible but being defensive)
    let sanitized_path = if sanitized_path.is_empty() {
        "unnamed_file"
    } else {
        sanitized_path
    };
    
    // Add line range (converting from 0-indexed to 1-indexed for user-facing numbers)
    format!("{}_{}-{}.txt", sanitized_path, start_line + 1, end_line + 1)
}

/// Metadata for a chunk file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    /// Original file path (relative to root directory)
    pub source_file: PathBuf,
    /// Start line in the source file (0-indexed)
    pub start_line: usize,
    /// End line in the source file (0-indexed)
    pub end_line: usize,
    /// Whether the chunk content was edited
    pub was_edited: bool,
    /// Timestamp when the chunk was created
    pub created_at: SystemTime,
    /// Optional description or notes about the chunk
    pub description: Option<String>,
}

impl ChunkMetadata {
    /// Create new chunk metadata
    pub fn new(
        source_file: PathBuf,
        start_line: usize,
        end_line: usize,
        was_edited: bool,
    ) -> Self {
        Self {
            source_file,
            start_line,
            end_line,
            was_edited,
            created_at: SystemTime::now(),
            description: None,
        }
    }
    
    /// Add a description to the metadata
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Save metadata to a file alongside the chunk
    pub fn save_to_file(&self, chunk_path: &Path) -> Result<()> {
        // Create the metadata filename by replacing .txt with .meta.json
        let meta_filename = chunk_path.with_extension("meta.json");
        
        // Serialize the metadata to JSON
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize chunk metadata")?;
        
        // Write to file
        let mut file = File::create(&meta_filename)
            .context("Failed to create metadata file")?;
        file.write_all(json.as_bytes())
            .context("Failed to write metadata to file")?;
        
        Ok(())
    }
    
    /// Load metadata from a file
    pub fn load_from_file(chunk_path: &Path) -> Result<Self> {
        // Get the metadata filename
        let meta_filename = chunk_path.with_extension("meta.json");
        
        // Debug print
        eprintln!("Attempting to load metadata from: {}", meta_filename.display());
        
        // Check if the file exists
        if !meta_filename.exists() {
            // This is expected if we don't have metadata for a chunk
            eprintln!("Metadata file doesn't exist");
            return Err(anyhow::anyhow!("Metadata file doesn't exist"));
        }
        
        // Read the file
        let json = match fs::read_to_string(&meta_filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading metadata file: {}", e);
                return Err(anyhow::anyhow!("Failed to read metadata file: {}", e));
            }
        };
        
        // Deserialize from JSON
        let metadata = match serde_json::from_str(&json) {
            Ok(parsed) => parsed,
            Err(e) => {
                eprintln!("Error parsing metadata JSON: {}", e);
                eprintln!("JSON content: {}", json);
                return Err(anyhow::anyhow!("Failed to parse metadata JSON: {}", e));
            }
        };
        
        eprintln!("Successfully loaded metadata");
        
        Ok(metadata)
    }
    
    /// Get all chunk metadata for a source file
    pub fn get_all_for_source(source_file: &Path, chunk_dir: &Path) -> Result<Vec<Self>> {
        let mut result = Vec::new();
        
        // Ensure the chunks directory exists
        if !chunk_dir.exists() {
            return Ok(result);
        }
        
        // Read all metadata files in the chunk directory
        for entry in fs::read_dir(chunk_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Skip non-metadata files
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(metadata) = Self::load_from_file(&path.with_extension("txt")) {
                    // Check if this metadata is for the requested source file
                    if metadata.source_file == source_file {
                        result.push(metadata);
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Check if a line range overlaps with this chunk
    pub fn overlaps(&self, start_line: usize, end_line: usize) -> bool {
        // Check for any overlap between the two ranges
        !(end_line < self.start_line || start_line > self.end_line)
    }
}