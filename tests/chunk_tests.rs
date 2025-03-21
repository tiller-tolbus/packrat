use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

use packrat::viewer::Viewer;
use packrat::storage::ChunkStorage;

fn setup_test_environment() -> Result<(tempfile::TempDir, PathBuf, ChunkStorage)> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    
    let csv_path = root_path.join("chunks.csv");
    let chunk_storage = ChunkStorage::new(&csv_path)?;
    
    let test_file_path = root_path.join("test_file.txt");
    let mut test_file = File::create(&test_file_path)?;
    
    for i in 1..=20 {
        writeln!(test_file, "Line {}: This is test content for line {}.", i, i)?;
    }
    
    Ok((temp_dir, root_path, chunk_storage))
}

#[test]
fn test_chunking_with_edited_content() -> Result<()> {
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Create a selection spanning a few lines
    viewer.toggle_selection_mode();
    
    // Move the cursor to select several lines
    for _ in 0..6 {
        viewer.cursor_down();
    }
    
    // Verify selection was created
    assert!(viewer.selection_range().is_some(), "Selection should be created");
    
    // Create some edited content to replace the selection
    let edited_content = vec![
        "EDITED Line: This replaces the original content.".to_string(),
        "Another EDITED line with custom content.".to_string(),
        "A third EDITED line with different text.".to_string(),
    ];
    
    // Update the selected content
    let updated = viewer.update_selected_content(edited_content.clone());
    assert!(updated, "Selected content should be updated");
    
    // Save the edited selection as a chunk
    let chunk_id = viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Verify the chunk was saved
    assert!(!chunk_id.is_empty(), "Should receive a valid chunk ID");
    
    // Check the chunks in storage
    let chunks = chunk_storage.get_chunks();
    assert!(!chunks.is_empty(), "Storage should contain at least one chunk");
    
    // Get the saved chunk and verify its content
    let saved_chunk = &chunks[0];
    
    // Verify that edited content is in the chunk
    let contains_edited = edited_content.iter().any(|line| saved_chunk.content.contains(line));
    assert!(contains_edited, "Chunk should contain the edited content");
    
    // Verify edited flag is set
    assert!(viewer.has_edited_content(), "Viewer should track that content was edited");
    assert!(saved_chunk.edited, "Chunk should be marked as edited");
    
    // Verify chunked ranges are tracked
    let chunked_ranges = viewer.chunked_ranges();
    assert!(!chunked_ranges.is_empty(), "Viewer should track chunked ranges");
    
    Ok(())
}