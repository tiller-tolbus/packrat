use anyhow::{Context, Result};
use csv;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Represents a single text chunk with metadata
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Unique identifier for the chunk
    pub id: String,
    
    /// Original file path (relative to root)
    pub file_path: PathBuf,
    
    /// Starting line number (1-indexed)
    pub start_line: usize,
    
    /// Ending line number (1-indexed)
    pub end_line: usize,
    
    /// The actual chunk text content
    pub content: String,
    
    /// Timestamp when the chunk was created
    pub timestamp: u64,
    
    /// Whether the chunk was edited before saving
    pub edited: bool,
    
    /// Optional user-provided labels
    pub labels: Vec<String>,
}

// Custom serialization for Chunk to handle Vec<String> labels field
impl Serialize for Chunk {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        
        // Convert labels Vec<String> to a single string with vertical bar separator
        // Using a non-comma separator to better handle labels containing commas
        let labels_str = self.labels.join("|");
        
        let mut state = serializer.serialize_struct("Chunk", 8)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("file_path", &self.file_path)?;
        state.serialize_field("start_line", &self.start_line)?;
        state.serialize_field("end_line", &self.end_line)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("timestamp", &self.timestamp)?;
        state.serialize_field("edited", &self.edited)?;
        state.serialize_field("labels", &labels_str)?;
        state.end()
    }
}

// Custom deserialization for Chunk to handle separator-delimited labels string
impl<'de> Deserialize<'de> for Chunk {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ChunkHelper {
            id: String,
            file_path: PathBuf,
            start_line: usize,
            end_line: usize,
            content: String,
            timestamp: u64,
            edited: bool,
            labels: String,
        }
        
        let helper = ChunkHelper::deserialize(deserializer)?;
        
        // Parse the labels string back to Vec<String>
        // Using vertical bar separator to better handle labels containing commas
        let labels = if helper.labels.is_empty() {
            Vec::new()
        } else {
            helper.labels.split('|').map(String::from).collect()
        };
        
        Ok(Chunk {
            id: helper.id,
            file_path: helper.file_path,
            start_line: helper.start_line,
            end_line: helper.end_line,
            content: helper.content,
            timestamp: helper.timestamp,
            edited: helper.edited,
            labels,
        })
    }
}

impl Chunk {
    /// Create a new chunk
    pub fn new(
        file_path: PathBuf, 
        start_line: usize, 
        end_line: usize, 
        content: String,
        edited: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            file_path,
            start_line,
            end_line,
            content,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            edited,
            labels: Vec::new(),
        }
    }
}

/// Manages chunk storage using CSV
pub struct ChunkStorage {
    /// Path to the CSV file
    csv_path: PathBuf,
    
    /// In-memory cache of chunks
    chunks: Vec<Chunk>,
}

impl ChunkStorage {
    /// Create a new storage manager
    pub fn new<P: AsRef<Path>>(csv_path: P) -> Result<Self> {
        let csv_path = csv_path.as_ref().to_path_buf();
        let chunks = if csv_path.exists() {
            Self::load_chunks(&csv_path)?
        } else {
            // Create parent directories if they don't exist
            if let Some(parent) = csv_path.parent() {
                fs::create_dir_all(parent).context("Failed to create parent directories for CSV file")?;
            }
            // Return empty chunks for new file
            Vec::new()
        };
        
        Ok(Self {
            csv_path,
            chunks,
        })
    }
    
    /// Add a new chunk to storage
    pub fn add_chunk(&mut self, chunk: Chunk) -> Result<()> {
        self.chunks.push(chunk);
        self.save()
    }
    
    /// Get all chunks
    pub fn get_chunks(&self) -> &[Chunk] {
        &self.chunks
    }
    
    /// Get chunks for a specific file
    pub fn get_chunks_for_file<P: AsRef<Path>>(&self, file_path: P) -> Vec<&Chunk> {
        let path = file_path.as_ref();
        self.chunks
            .iter()
            .filter(|chunk| chunk.file_path == path)
            .collect()
    }
    
    /// Get ranges of chunked lines for a specific file
    pub fn get_chunked_ranges<P: AsRef<Path>>(&self, file_path: P) -> Vec<(usize, usize)> {
        let path = file_path.as_ref();
        self.chunks
            .iter()
            .filter(|chunk| chunk.file_path == path)
            .map(|chunk| (chunk.start_line, chunk.end_line))
            .collect()
    }
    
    /// Calculate chunking percentage for a file
    pub fn calculate_chunking_percentage<P: AsRef<Path>>(&self, file_path: P, total_lines: usize) -> f64 {
        if total_lines == 0 {
            return 0.0;
        }
        
        let path = file_path.as_ref();
        
        // Get all chunks for this file
        let file_chunks: Vec<_> = self.chunks
            .iter()
            .filter(|chunk| chunk.file_path == path)
            .collect();
        
        if file_chunks.is_empty() {
            return 0.0;
        }
        
        // Count unique chunked lines
        let mut chunked_lines = vec![false; total_lines];
        
        for chunk in file_chunks {
            for i in chunk.start_line..=chunk.end_line.min(total_lines - 1) {
                chunked_lines[i] = true;
            }
        }
        
        // Calculate percentage
        let chunked_count = chunked_lines.iter().filter(|&&chunked| chunked).count();
        (chunked_count as f64 / total_lines as f64) * 100.0
    }
    
    /// Save all chunks to the CSV file
    pub fn save(&self) -> Result<()> {
        // Create writer with BufWriter for better performance
        let writer = BufWriter::new(File::create(&self.csv_path)?);
        
        // Create a CSV writer with custom options for better quoting
        let mut csv_writer = csv::WriterBuilder::new()
            .quote_style(csv::QuoteStyle::Always)  // Always quote all fields
            .double_quote(true)                    // Ensure quotes inside fields are escaped properly
            .from_writer(writer);
        
        // Write each chunk to CSV
        for chunk in &self.chunks {
            csv_writer.serialize(chunk)?;
        }
        
        // Flush writer
        csv_writer.flush()?;
        
        Ok(())
    }
    
    /// Load chunks from CSV file
    fn load_chunks(csv_path: &Path) -> Result<Vec<Chunk>> {
        // If file doesn't exist, return empty vector
        if !csv_path.exists() {
            return Ok(Vec::new());
        }
        
        // Open file with BufReader for better performance
        let reader = BufReader::new(File::open(csv_path)?);
        
        // Create a CSV reader with custom options to match our writer
        let mut csv_reader = csv::ReaderBuilder::new()
            .flexible(true)              // Be more lenient with parsing
            .double_quote(true)          // Handle double-quoted quotes
            .from_reader(reader);
        
        // Parse CSV into Chunk records
        let mut chunks = Vec::new();
        for result in csv_reader.deserialize() {
            let chunk: Chunk = result?;
            chunks.push(chunk);
        }
        
        Ok(chunks)
    }
}

