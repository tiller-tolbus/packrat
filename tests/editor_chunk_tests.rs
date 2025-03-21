use anyhow::Result;
use packrat::editor::Editor;
use packrat::viewer::Viewer;
use packrat::storage::ChunkStorage;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::{File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

/// Setup a test environment for chunk editing tests
fn setup_test_environment() -> Result<(tempfile::TempDir, PathBuf, ChunkStorage)> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    
    // Create a CSV file for chunks
    let csv_path = root_path.join("chunks.csv");
    let chunk_storage = ChunkStorage::new(&csv_path)?;
    
    // Create a test file
    let test_file_path = root_path.join("test_file.txt");
    let mut test_file = File::create(&test_file_path)?;
    
    // Write 20 lines to the test file
    for i in 1..=20 {
        writeln!(test_file, "Line {}: This is test content for line {}.", i, i)?;
    }
    
    Ok((temp_dir, root_path, chunk_storage))
}


