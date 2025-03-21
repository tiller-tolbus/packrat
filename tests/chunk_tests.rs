use anyhow::Result;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Read, Write};
use tempfile::tempdir;

// Import types from the main crate
use packrat::viewer::Viewer;
use packrat::storage::ChunkStorage;

// Helper function to create test files and directory structure
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



#[test]
fn test_chunking_with_edited_content() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Select lines 5-7 (indexes 4-6)
    viewer.toggle_selection_mode();
    for _ in 0..4 {
        viewer.cursor_down(); // Move to line 5 (index 4)
    }
    viewer.cursor_down(); // Move to line 6 (index 5)
    viewer.cursor_down(); // Move to line 7 (index 6)
    
    // Verify we have a selection
    assert!(viewer.selection_range().is_some(), "Should have a selection range");
    
    // Prepare edited content
    let edited_content = vec![
        "Line 5: EDITED content for testing.".to_string(),
        "Line 6: EDITED content for testing.".to_string(),
        "Line 7: EDITED content for testing.".to_string(),
    ];
    
    // Update the selected content with edited version
    assert!(viewer.update_selected_content(edited_content.clone()));
    
    // Save the chunk
    let _chunk_id = viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Get the saved chunk
    let chunks = chunk_storage.get_chunks();
    assert_eq!(chunks.len(), 1, "Should have one chunk in storage");
    
    // Verify the chunk content contains the edited content
    let chunk = &chunks[0];
    assert!(chunk.content.contains("Line 5: EDITED content for testing."));
    assert!(chunk.content.contains("Line 6: EDITED content for testing."));
    assert!(chunk.content.contains("Line 7: EDITED content for testing."));
    
    // Verify has_edited_content flag
    assert!(viewer.has_edited_content(), "has_edited_content should be true");
    
    // Verify the edited flag in the chunk
    assert!(chunk.edited, "Chunk edited flag should be true");
    
    Ok(())
}


