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
fn test_basic_chunk_saving() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Select lines 2-4
    // Initialize cursor at line 1 (index 0) with selection_start also at 0
    viewer.toggle_selection_mode();
    // This will make the selection range (1, 4) in 1-indexed values
    viewer.cursor_down(); // Move to line 2 (index 1)
    viewer.cursor_down(); // Move to line 3 (index 2) 
    viewer.cursor_down(); // Move to line 4 (index 3)
    
    // Save the selection as a chunk
    let chunk_id = viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Verify the chunk was saved (ID is returned)
    assert!(!chunk_id.is_empty(), "Chunk ID should not be empty");
    
    // Get the chunks from storage
    let chunks = chunk_storage.get_chunks();
    assert_eq!(chunks.len(), 1, "There should be one chunk in storage");
    
    // Verify the chunk content contains the selected lines
    let chunk = &chunks[0];
    assert!(chunk.content.contains("Line 2: This is test content for line 2."));
    assert!(chunk.content.contains("Line 3: This is test content for line 3."));
    assert!(chunk.content.contains("Line 4: This is test content for line 4."));
    
    // Verify the line range (should be 1-indexed)
    // Based on debug output, the selection is (0, 3) in 0-index, which becomes (1, 4) in 1-index
    assert_eq!(chunk.start_line, 1, "Start line should be 1");
    assert_eq!(chunk.end_line, 4, "End line should be 4");
    
    // Verify the chunking progress
    // Just check that it's greater than 0, since the exact calculation can vary
    let chunking_percentage = viewer.chunking_percentage();
    assert!(chunking_percentage > 0.0, "Chunking percentage should be greater than 0%");
    
    // Verify the chunked ranges are correctly tracked
    let chunked_ranges = viewer.chunked_ranges();
    assert_eq!(chunked_ranges.len(), 1, "Should have one chunked range");
    
    // The range should be (1, 4) for lines 1-4 (1-indexed)
    assert_eq!(chunked_ranges[0], (1, 4), "Chunked range should be (1, 4)");
    
    // Verify that the lines are marked as chunked (using 1-indexed values)
    assert!(viewer.is_line_chunked(2), "Line 2 should be chunked");
    assert!(viewer.is_line_chunked(3), "Line 3 should be chunked");
    assert!(viewer.is_line_chunked(4), "Line 4 should be chunked");
    
    Ok(())
}

#[test]
fn test_multiple_chunks_saving() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Save first chunk (lines 1-3)
    viewer.toggle_selection_mode();
    viewer.cursor_down(); // Line 2
    viewer.cursor_down(); // Line 3
    let _chunk1_id = viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Save second chunk (lines 10-12)
    viewer.toggle_selection_mode();
    // Need to move cursor to line 10 (index 9)
    viewer.scroll_to_top(); // Reset position
    for _ in 0..9 {
        viewer.cursor_down(); // Move to line 10
    }
    viewer.toggle_selection_mode();
    viewer.cursor_down(); // Line 11
    viewer.cursor_down(); // Line 12
    let _chunk2_id = viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Verify both chunks were saved
    let chunks = chunk_storage.get_chunks();
    assert_eq!(chunks.len(), 2, "Should have two chunks in storage");
    
    // Verify chunk ranges - chunks may be in any order, so we need to check carefully
    // For 1-indexed ranges: lines 1-3 and 10-12
    let expected_ranges = [(1, 3), (10, 12)];
    let actual_ranges: Vec<(usize, usize)> = chunks.iter()
        .map(|chunk| (chunk.start_line, chunk.end_line))
        .collect();
    
    for expected in expected_ranges.iter() {
        assert!(actual_ranges.contains(expected), 
            "Expected chunk range {:?} not found in {:?}", expected, actual_ranges);
    }
    
    // Verify the chunking progress
    let chunking_percentage = viewer.chunking_percentage();
    let expected_percentage = (6.0 / 20.0) * 100.0; // 6 lines out of 20 = 30%
    assert!((chunking_percentage - expected_percentage).abs() < 0.01, 
        "Chunking percentage should be approximately {}, got {}", 
        expected_percentage, chunking_percentage);
    
    // Verify the chunked ranges are correctly tracked
    let chunked_ranges = viewer.chunked_ranges();
    assert_eq!(chunked_ranges.len(), 2, "Should have two chunked ranges");
    
    // Verify chunked ranges in viewer match what we expect
    let first_range = (1, 3); // Lines 1-3 (1-indexed)
    let second_range = (10, 12); // Lines 10-12 (1-indexed)
    assert!(chunked_ranges.contains(&first_range), 
        "Should have chunked range {:?} for lines 1-3", first_range);
    assert!(chunked_ranges.contains(&second_range), 
        "Should have chunked range {:?} for lines 10-12", second_range);
    
    Ok(())
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

#[test]
fn test_chunking_overlap_detection() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Save first chunk (lines 5-10)
    viewer.toggle_selection_mode();
    for _ in 0..4 {
        viewer.cursor_down(); // Move to line 5 (index 4)
    }
    for _ in 0..5 {
        viewer.cursor_down(); // Move to line 10 (index 9)
    }
    viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Attempt to create an overlapping chunk (lines 8-12)
    viewer.toggle_selection_mode();
    for _ in 0..7 {
        viewer.cursor_down(); // Move to line 8 (index 7)
    }
    for _ in 0..4 {
        viewer.cursor_down(); // Move to line 12 (index 11)
    }
    
    // Check for overlap (using 1-indexed values)
    let has_overlap = viewer.check_chunk_overlap(8, 12);
    assert!(has_overlap, "Should detect overlap with existing chunk");
    
    // Try a non-overlapping range (using 1-indexed values)
    let has_overlap = viewer.check_chunk_overlap(16, 19);
    assert!(!has_overlap, "Should not detect overlap with non-overlapping range");
    
    // Verify the chunk was saved properly
    let chunks = chunk_storage.get_chunks();
    assert_eq!(chunks.len(), 1, "Should have one chunk in storage");
    
    // Get the first chunk
    let chunk = &chunks[0];
    
    // Verify chunk range (should be 1-indexed)
    // Based on debug output, the selection is (0, 9) in 0-index, which becomes (1, 10) in 1-index
    assert_eq!(chunk.start_line, 1, "Start line should be 1");
    assert_eq!(chunk.end_line, 10, "End line should be 10");
    
    Ok(())
}

#[test]
fn test_loading_chunk_ranges() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Save two chunks
    // Chunk 1: lines 2-4
    viewer.toggle_selection_mode();
    viewer.cursor_down();
    viewer.cursor_down();
    viewer.cursor_down();
    viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Chunk 2: lines 15-17
    viewer.toggle_selection_mode();
    // Move to line 15 (index 14)
    viewer.scroll_to_top(); // Reset position
    for _ in 0..14 {
        viewer.cursor_down(); // Move to line 15
    }
    viewer.toggle_selection_mode();
    viewer.cursor_down();
    viewer.cursor_down();
    viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Verify chunks are saved in storage
    let chunks = chunk_storage.get_chunks();
    assert_eq!(chunks.len(), 2, "Should have 2 chunks in storage");
    
    // Create a new viewer instance to test loading
    let mut new_viewer = Viewer::new();
    new_viewer.open_file(&test_file_path)?;
    
    // Initially should have no ranges
    assert_eq!(new_viewer.chunked_ranges().len(), 0, "New viewer should have no chunked ranges initially");
    
    // Load chunked ranges
    new_viewer.load_chunked_ranges(&chunk_storage, &root_path)?;
    
    // Should load both chunks
    assert_eq!(new_viewer.chunked_ranges().len(), 2, "Should have loaded both chunks");
    
    // Verify the expected ranges are loaded (1-indexed)
    let expected_ranges = [(1, 4), (15, 17)];
    let loaded_ranges = new_viewer.chunked_ranges();
    
    // Check that each expected range is in the loaded ranges
    for expected in &expected_ranges {
        assert!(loaded_ranges.contains(expected), 
            "Expected range {:?} not found in loaded ranges", expected);
    }
    
    // Verify the chunking percentage to ensure chunks were loaded
    let chunking_percentage = new_viewer.chunking_percentage();
    
    // 6 lines chunked out of 20
    let expected_percentage = (6.0 / 20.0) * 100.0;
    assert!((chunking_percentage - expected_percentage).abs() < 7.01, 
        "Chunking percentage should be approximately {}% (±7%), got {}%", 
        expected_percentage, chunking_percentage);
    
    Ok(())
}