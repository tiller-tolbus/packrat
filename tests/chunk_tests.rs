use anyhow::Result;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Read, Write};
use tempfile::tempdir;

// Import types from the main crate
use packrat::viewer::Viewer;
use packrat::explorer::Explorer;

// Helper function to create test files and directory structure
fn setup_test_environment() -> Result<(tempfile::TempDir, PathBuf, PathBuf)> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    
    // Create chunks directory
    let chunks_dir = root_path.join("chunks");
    fs::create_dir_all(&chunks_dir)?;
    
    // Create a test file
    let test_file_path = root_path.join("test_file.txt");
    let mut test_file = File::create(&test_file_path)?;
    
    // Write 20 lines to the test file
    for i in 1..=20 {
        writeln!(test_file, "Line {}: This is test content for line {}.", i, i)?;
    }
    
    Ok((temp_dir, root_path, chunks_dir))
}

#[test]
fn test_basic_chunk_saving() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, chunks_dir) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Select lines 2-4 (indexes 1-3)
    viewer.toggle_selection_mode();
    viewer.cursor_down(); // Move to line 2 (index 1)
    viewer.cursor_down(); // Move to line 3 (index 2)
    viewer.cursor_down(); // Move to line 4 (index 3)
    
    // Save the selection as a chunk
    let chunk_path = viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Verify the chunk file exists
    assert!(chunk_path.exists(), "Chunk file should exist");
    
    // Verify the chunk filename follows the correct pattern (test_file_txt_2-4.txt)
    assert_eq!(chunk_path.file_name().unwrap().to_string_lossy(), "test_file_txt_2-4.txt");
    
    // Read the chunk content
    let mut chunk_content = String::new();
    File::open(&chunk_path)?.read_to_string(&mut chunk_content)?;
    
    // Verify the chunk content matches the selected lines
    let expected_content = "Line 2: This is test content for line 2.\nLine 3: This is test content for line 3.\nLine 4: This is test content for line 4.\n";
    assert_eq!(chunk_content, expected_content);
    
    // Verify the chunking progress
    let chunking_percentage = viewer.chunking_percentage();
    let expected_percentage = (3.0 / 20.0) * 100.0; // 3 lines out of 20 = 15%
    assert!((chunking_percentage - expected_percentage).abs() < 0.01, 
        "Chunking percentage should be approximately {}, got {}", 
        expected_percentage, chunking_percentage);
    
    // Verify the chunked ranges are correctly tracked
    let chunked_ranges = viewer.chunked_ranges();
    assert_eq!(chunked_ranges.len(), 1, "Should have one chunked range");
    assert_eq!(chunked_ranges[0], (1, 3), "Chunked range should be (1, 3)");
    
    // Verify the is_line_chunked method works correctly
    assert!(!viewer.is_line_chunked(0), "Line 1 should not be chunked");
    assert!(viewer.is_line_chunked(1), "Line 2 should be chunked");
    assert!(viewer.is_line_chunked(2), "Line 3 should be chunked");
    assert!(viewer.is_line_chunked(3), "Line 4 should be chunked");
    assert!(!viewer.is_line_chunked(4), "Line 5 should not be chunked");
    
    Ok(())
}

#[test]
fn test_multiple_chunks_saving() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, chunks_dir) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Save first chunk (lines 1-3)
    viewer.toggle_selection_mode();
    viewer.cursor_down(); // Line 2
    viewer.cursor_down(); // Line 3
    let chunk1_path = viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Save second chunk (lines 10-12)
    viewer.toggle_selection_mode();
    viewer.cursor_position = 9; // Set cursor to line 10
    viewer.toggle_selection_mode();
    viewer.cursor_down(); // Line 11
    viewer.cursor_down(); // Line 12
    let chunk2_path = viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Verify both chunk files exist
    assert!(chunk1_path.exists(), "First chunk file should exist");
    assert!(chunk2_path.exists(), "Second chunk file should exist");
    
    // Verify the chunk filenames
    assert_eq!(chunk1_path.file_name().unwrap().to_string_lossy(), "test_file_txt_1-3.txt");
    assert_eq!(chunk2_path.file_name().unwrap().to_string_lossy(), "test_file_txt_10-12.txt");
    
    // Verify the chunking progress
    let chunking_percentage = viewer.chunking_percentage();
    let expected_percentage = (6.0 / 20.0) * 100.0; // 6 lines out of 20 = 30%
    assert!((chunking_percentage - expected_percentage).abs() < 0.01, 
        "Chunking percentage should be approximately {}, got {}", 
        expected_percentage, chunking_percentage);
    
    // Verify the chunked ranges are correctly tracked
    let chunked_ranges = viewer.chunked_ranges();
    assert_eq!(chunked_ranges.len(), 2, "Should have two chunked ranges");
    
    // Ranges might not be in insertion order, so check both possibilities
    let has_first_range = chunked_ranges.contains(&(0, 2)) || chunked_ranges.contains(&(2, 0));
    let has_second_range = chunked_ranges.contains(&(9, 11)) || chunked_ranges.contains(&(11, 9));
    assert!(has_first_range, "Should have chunked range for lines 1-3");
    assert!(has_second_range, "Should have chunked range for lines 10-12");
    
    Ok(())
}

#[test]
fn test_chunking_with_edited_content() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, chunks_dir) = setup_test_environment()?;
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
    
    // Get the selection range
    let selection_range = viewer.selection_range().unwrap();
    
    // Prepare edited content
    let edited_content = vec![
        "Line 5: EDITED content for testing.".to_string(),
        "Line 6: EDITED content for testing.".to_string(),
        "Line 7: EDITED content for testing.".to_string(),
    ];
    
    // Update the selected content with edited version
    assert!(viewer.update_selected_content(edited_content.clone()));
    
    // Save the chunk
    let chunk_path = viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Read the chunk content
    let mut chunk_content = String::new();
    File::open(&chunk_path)?.read_to_string(&mut chunk_content)?;
    
    // Verify the chunk content matches the edited content, not the original
    let expected_content = "Line 5: EDITED content for testing.\nLine 6: EDITED content for testing.\nLine 7: EDITED content for testing.\n";
    assert_eq!(chunk_content, expected_content);
    
    // Verify has_edited_content flag
    assert!(viewer.has_edited_content(), "has_edited_content should be true");
    
    Ok(())
}

#[test]
fn test_chunking_overlap_detection() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, chunks_dir) = setup_test_environment()?;
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
    viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Attempt to create an overlapping chunk (lines 8-12)
    viewer.toggle_selection_mode();
    for _ in 0..7 {
        viewer.cursor_down(); // Move to line 8 (index 7)
    }
    for _ in 0..4 {
        viewer.cursor_down(); // Move to line 12 (index 11)
    }
    
    // Check for overlap
    let has_overlap = viewer.check_chunk_overlap(7, 11);
    assert!(has_overlap, "Should detect overlap with existing chunk");
    
    // Try a non-overlapping range
    let has_overlap = viewer.check_chunk_overlap(15, 18);
    assert!(!has_overlap, "Should not detect overlap with non-overlapping range");
    
    Ok(())
}

#[test]
fn test_loading_chunk_ranges() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, chunks_dir) = setup_test_environment()?;
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
    viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Chunk 2: lines 15-17
    viewer.toggle_selection_mode();
    viewer.cursor_position = 14; // Set to line 15
    viewer.toggle_selection_mode();
    viewer.cursor_down();
    viewer.cursor_down();
    viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Create a new viewer instance to test loading
    let mut new_viewer = Viewer::new();
    new_viewer.open_file(&test_file_path)?;
    
    // Initially should have no ranges
    assert_eq!(new_viewer.chunked_ranges().len(), 0, "New viewer should have no chunked ranges initially");
    
    // Load chunked ranges
    new_viewer.load_chunked_ranges(&chunks_dir, &root_path)?;
    
    // Should have loaded both ranges
    assert_eq!(new_viewer.chunked_ranges().len(), 2, "Should have loaded two chunked ranges");
    
    // Verify the loaded ranges
    let loaded_ranges = new_viewer.chunked_ranges();
    let has_first_range = loaded_ranges.contains(&(1, 3)) || loaded_ranges.contains(&(3, 1));
    let has_second_range = loaded_ranges.contains(&(14, 16)) || loaded_ranges.contains(&(16, 14));
    assert!(has_first_range, "Should have loaded range for lines 2-4");
    assert!(has_second_range, "Should have loaded range for lines 15-17");
    
    // Verify chunking percentage
    let chunking_percentage = new_viewer.chunking_percentage();
    let expected_percentage = (6.0 / 20.0) * 100.0; // 6 lines out of 20 = 30%
    assert!((chunking_percentage - expected_percentage).abs() < 0.01, 
        "Chunking percentage should be approximately {}, got {}", 
        expected_percentage, chunking_percentage);
    
    Ok(())
}

#[test]
fn test_explorer_chunking_progress() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, chunks_dir) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and save a chunk
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Select lines 1-5 (25% of the file)
    viewer.toggle_selection_mode();
    for _ in 0..4 {
        viewer.cursor_down();
    }
    
    // Save the chunk
    viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Create an explorer
    let mut explorer = Explorer::new(&root_path)?;
    
    // Initialize chunking progress
    explorer.init_chunking_progress(&chunks_dir)?;
    
    // Find the test file in the entries
    let test_file_entry = explorer.entries().iter()
        .find(|entry| entry.path == test_file_path)
        .expect("Test file should be in explorer entries");
    
    // Verify the chunking progress
    let expected_percentage = (5.0 / 20.0) * 100.0; // 5 lines out of 20 = 25%
    assert!((test_file_entry.chunking_progress - expected_percentage).abs() < 0.01, 
        "Explorer chunking progress should be approximately {}, got {}", 
        expected_percentage, test_file_entry.chunking_progress);
    
    Ok(())
}