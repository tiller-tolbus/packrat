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
    
    /// Starting line number (0-indexed)
    pub start_line: usize,
    
    /// Ending line number (0-indexed)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_chunk_creation() {
        let file_path = PathBuf::from("test/path.txt");
        let content = "Test content\nLine 2\nLine 3".to_string();
        
        let chunk = Chunk::new(
            file_path.clone(),
            5,
            10,
            content.clone(),
            false,
        );
        
        assert_eq!(chunk.file_path, file_path);
        assert_eq!(chunk.start_line, 5);
        assert_eq!(chunk.end_line, 10);
        assert_eq!(chunk.content, content);
        assert_eq!(chunk.edited, false);
        assert!(chunk.id.len() > 0);
        assert!(chunk.labels.is_empty());
    }
    
    #[test]
    fn test_storage_basic_operations() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = tempdir()?;
        let csv_path = temp_dir.path().join("chunks.csv");
        
        // Create a new storage
        let mut storage = ChunkStorage::new(&csv_path)?;
        
        // Initially, there should be no chunks
        assert!(storage.get_chunks().is_empty());
        
        // Add a chunk
        let chunk = Chunk::new(
            PathBuf::from("test.txt"),
            0,
            5,
            "Test content".to_string(),
            false,
        );
        
        storage.add_chunk(chunk)?;
        
        // Verify the chunk was added
        assert_eq!(storage.get_chunks().len(), 1);
        
        // Create a new storage instance that should load from the CSV file
        let loaded_storage = ChunkStorage::new(&csv_path)?;
        
        // Verify the chunk was loaded
        assert_eq!(loaded_storage.get_chunks().len(), 1);
        assert_eq!(loaded_storage.get_chunks()[0].start_line, 0);
        assert_eq!(loaded_storage.get_chunks()[0].end_line, 5);
        
        Ok(())
    }
    
    #[test]
    fn test_get_chunks_for_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let csv_path = temp_dir.path().join("chunks.csv");
        
        let mut storage = ChunkStorage::new(&csv_path)?;
        
        // Add chunks for different files
        let file1 = PathBuf::from("file1.txt");
        let file2 = PathBuf::from("file2.txt");
        
        storage.add_chunk(Chunk::new(
            file1.clone(),
            0,
            5,
            "Content from file 1, chunk 1".to_string(),
            false,
        ))?;
        
        storage.add_chunk(Chunk::new(
            file1.clone(),
            10,
            15,
            "Content from file 1, chunk 2".to_string(),
            false,
        ))?;
        
        storage.add_chunk(Chunk::new(
            file2.clone(),
            0,
            5,
            "Content from file 2".to_string(),
            false,
        ))?;
        
        // Test filtering
        let file1_chunks = storage.get_chunks_for_file(&file1);
        assert_eq!(file1_chunks.len(), 2);
        
        let file2_chunks = storage.get_chunks_for_file(&file2);
        assert_eq!(file2_chunks.len(), 1);
        
        // Test chunked ranges
        let file1_ranges = storage.get_chunked_ranges(&file1);
        assert_eq!(file1_ranges.len(), 2);
        assert!(file1_ranges.contains(&(0, 5)));
        assert!(file1_ranges.contains(&(10, 15)));
        
        Ok(())
    }
    
    #[test]
    fn test_chunking_percentage() -> Result<()> {
        let temp_dir = tempdir()?;
        let csv_path = temp_dir.path().join("chunks.csv");
        
        let mut storage = ChunkStorage::new(&csv_path)?;
        let file_path = PathBuf::from("test.txt");
        
        // Set up a file with 100 lines
        let total_lines = 100;
        
        // Add chunks covering lines 0-9 and 20-29 (20 lines total)
        storage.add_chunk(Chunk::new(
            file_path.clone(),
            0,
            9,
            "Chunk 1".to_string(),
            false,
        ))?;
        
        storage.add_chunk(Chunk::new(
            file_path.clone(),
            20,
            29,
            "Chunk 2".to_string(),
            false,
        ))?;
        
        // Calculate percentage - should be 20%
        let percentage = storage.calculate_chunking_percentage(&file_path, total_lines);
        assert!((percentage - 20.0).abs() < 0.001, "Expected 20%, got {}", percentage);
        
        // Add another chunk with some overlap (lines 5-15)
        storage.add_chunk(Chunk::new(
            file_path.clone(),
            5,
            15,
            "Chunk 3 with overlap".to_string(),
            false,
        ))?;
        
        // Re-calculate - should now be 26% (lines 0-15 and 20-29)
        let percentage = storage.calculate_chunking_percentage(&file_path, total_lines);
        assert!((percentage - 26.0).abs() < 0.001, "Expected 26%, got {}", percentage);
        
        Ok(())
    }
    
    #[test]
    fn test_multiline_content_csv_roundtrip() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = tempdir()?;
        let csv_path = temp_dir.path().join("chunks.csv");
        
        // Create a new storage
        let mut storage = ChunkStorage::new(&csv_path)?;
        
        // Create a chunk with multi-line content including special characters and code-like content
        let multi_line_content = "First line\nSecond line with \"quotes\"\nThird line with commas, semicolons; and tabs\t\nFourth line with special chars: &*(){}[]\nCode with brackets: let chunks = Layout::default()\n    .direction(Direction::Vertical)\n    .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())".to_string();
        
        let chunk = Chunk::new(
            PathBuf::from("multiline_test.txt"),
            1,
            5,
            multi_line_content.clone(),
            true,
        );
        
        // Add the chunk and save to CSV
        storage.add_chunk(chunk)?;
        
        // Verify the chunk was added
        assert_eq!(storage.get_chunks().len(), 1);
        
        // Create a new storage instance to load from the CSV file
        let loaded_storage = ChunkStorage::new(&csv_path)?;
        
        // Verify the chunk was loaded correctly
        assert_eq!(loaded_storage.get_chunks().len(), 1);
        
        // Check that the multi-line content is preserved exactly
        let loaded_chunk = &loaded_storage.get_chunks()[0];
        assert_eq!(loaded_chunk.content, multi_line_content);
        
        Ok(())
    }
    
    #[test]
    fn test_labels_serialization() -> Result<()> {
        // Create a temporary directory for the test
        let temp_dir = tempdir()?;
        let csv_path = temp_dir.path().join("chunks.csv");
        
        // Create a new storage
        let mut storage = ChunkStorage::new(&csv_path)?;
        
        // Create a chunk 
        let mut chunk = Chunk::new(
            PathBuf::from("labels_test.txt"),
            1,
            5,
            "Content with labels".to_string(),
            false,
        );
        
        // Add some labels with special characters
        // Note: We're avoiding commas in the labels since our serialization
        // uses commas as the separator
        chunk.labels = vec![
            "label1".to_string(),
            "label with spaces".to_string(),
            "label-with-dashes".to_string(),
            "label_with_underscores".to_string()
        ];
        
        // Add the chunk and save to CSV
        storage.add_chunk(chunk)?;
        
        // Verify the chunk was added
        assert_eq!(storage.get_chunks().len(), 1);
        
        // Create a new storage instance to load from the CSV file
        let loaded_storage = ChunkStorage::new(&csv_path)?;
        
        // Verify the chunk was loaded correctly
        assert_eq!(loaded_storage.get_chunks().len(), 1);
        
        // Check that the labels are preserved correctly
        let loaded_chunk = &loaded_storage.get_chunks()[0];
        assert_eq!(loaded_chunk.labels.len(), 4);
        assert_eq!(loaded_chunk.labels[0], "label1");
        assert_eq!(loaded_chunk.labels[1], "label with spaces");
        assert_eq!(loaded_chunk.labels[2], "label-with-dashes");
        assert_eq!(loaded_chunk.labels[3], "label_with_underscores");
        
        Ok(())
    }
}