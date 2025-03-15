// This module will be fully implemented in later phases
// For now, it just has some default settings

use std::path::PathBuf;

/// Application configuration
#[derive(Clone, Debug)]
pub struct Config {
    /// Directory where chunks are stored
    pub chunk_dir: PathBuf,
    /// Maximum chunk size in bytes (0 = no limit)
    pub max_chunk_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Default to "chunks" subdirectory in current directory
            chunk_dir: PathBuf::from("chunks"),
            // No default chunk size limit
            max_chunk_size: 0,
        }
    }
}