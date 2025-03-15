use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

// Import the Viewer from the main crate
use packrat::viewer::Viewer;

// Helper function to create test files
fn setup_test_files() -> Result<(tempfile::TempDir, PathBuf, PathBuf, PathBuf)> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    
    // Create a small text file
    let small_file_path = root_path.join("small_file.txt");
    let mut small_file = File::create(&small_file_path)?;
    writeln!(small_file, "Line 1: This is a small text file.")?;
    writeln!(small_file, "Line 2: It has only a few lines.")?;
    writeln!(small_file, "Line 3: Perfect for basic tests.")?;
    
    // Create a file with varying line lengths
    let varied_file_path = root_path.join("varied_lines.txt");
    let mut varied_file = File::create(&varied_file_path)?;
    writeln!(varied_file, "Short line")?;
    writeln!(varied_file, "This is a medium length line that has a bit more content.")?;
    writeln!(varied_file, "This is a much longer line that contains significantly more text which should test how the viewer handles longer content that might need to be wrapped or displayed differently depending on the terminal width.")?;
    writeln!(varied_file, "Another short line")?;
    writeln!(varied_file, "Yet another line with medium length content.")?;
    
    // Create a "large" file (for test purposes, 100 lines is sufficient)
    let large_file_path = root_path.join("large_file.txt");
    let mut large_file = File::create(&large_file_path)?;
    for i in 1..=100 {
        writeln!(large_file, "Line {}: This is line {} of the large test file.", i, i)?;
    }
    
    Ok((temp_dir, small_file_path, varied_file_path, large_file_path))
}

#[test]
fn test_viewer_open_file() -> Result<()> {
    let (_temp_dir, small_file_path, _, _) = setup_test_files()?;
    
    // Create a new viewer
    let mut viewer = Viewer::new();
    
    // Open the small file
    viewer.open_file(&small_file_path)?;
    
    // Check that file was opened correctly
    assert_eq!(viewer.file_path(), Some(small_file_path.as_path()), 
        "Viewer should have the correct file path");
    
    // Check content
    let content = viewer.content();
    assert_eq!(content.len(), 3, "Small file should have 3 lines");
    assert!(content[0].contains("Line 1"), "First line should contain 'Line 1'");
    assert!(content[1].contains("Line 2"), "Second line should contain 'Line 2'");
    assert!(content[2].contains("Line 3"), "Third line should contain 'Line 3'");
    
    // Check initial scroll position
    assert_eq!(viewer.scroll_position(), 0, "Initial scroll position should be 0");
    
    Ok(())
}

#[test]
fn test_viewer_scrolling() -> Result<()> {
    let (_temp_dir, _, _, large_file_path) = setup_test_files()?;
    
    // Create a new viewer
    let mut viewer = Viewer::new();
    
    // Open the large file
    viewer.open_file(&large_file_path)?;
    
    // Check initial scroll position
    assert_eq!(viewer.scroll_position(), 0, "Initial scroll position should be 0");
    
    // Test scrolling down
    viewer.scroll_down();
    assert_eq!(viewer.scroll_position(), 1, "Scroll position should be 1 after scrolling down");
    
    // Test scrolling down multiple times
    for _ in 0..5 {
        viewer.scroll_down();
    }
    assert_eq!(viewer.scroll_position(), 6, "Scroll position should be 6 after scrolling down 5 more times");
    
    // Test scrolling up
    viewer.scroll_up();
    assert_eq!(viewer.scroll_position(), 5, "Scroll position should be 5 after scrolling up");
    
    // Test page down (assuming page size of 10)
    viewer.scroll_page_down(10);
    assert_eq!(viewer.scroll_position(), 15, "Scroll position should be 15 after page down");
    
    // Test page up
    viewer.scroll_page_up(10);
    assert_eq!(viewer.scroll_position(), 5, "Scroll position should be 5 after page up");
    
    // Test jump to bottom
    viewer.scroll_to_bottom();
    assert_eq!(viewer.scroll_position(), 99, "Scroll position should be 99 (last line) after jumping to bottom");
    
    // Test jump to top
    viewer.scroll_to_top();
    assert_eq!(viewer.scroll_position(), 0, "Scroll position should be 0 after jumping to top");
    
    Ok(())
}

#[test]
fn test_viewer_boundary_conditions() -> Result<()> {
    let (_temp_dir, small_file_path, _, _) = setup_test_files()?;
    
    // Create a new viewer
    let mut viewer = Viewer::new();
    
    // Open the small file (3 lines)
    viewer.open_file(&small_file_path)?;
    
    // Test scrolling past the bottom
    for _ in 0..10 {
        viewer.scroll_down();
    }
    assert_eq!(viewer.scroll_position(), 2, "Scroll position should not go beyond the last line (2)");
    
    // Test scrolling past the top
    viewer.scroll_to_top();
    for _ in 0..10 {
        viewer.scroll_up();
    }
    assert_eq!(viewer.scroll_position(), 0, "Scroll position should not go below 0");
    
    // Test page down beyond end
    viewer.scroll_page_down(100);
    assert_eq!(viewer.scroll_position(), 2, "Scroll position should not go beyond the last line");
    
    // Test page up beyond beginning
    viewer.scroll_page_up(100);
    assert_eq!(viewer.scroll_position(), 0, "Scroll position should not go below 0");
    
    Ok(())
}

#[test]
fn test_viewer_visible_content() -> Result<()> {
    let (_temp_dir, _, _, large_file_path) = setup_test_files()?;
    
    // Create a new viewer
    let mut viewer = Viewer::new();
    
    // Open the large file
    viewer.open_file(&large_file_path)?;
    
    // Test visible content at beginning
    let visible = viewer.visible_content(5);
    assert_eq!(visible.len(), 5, "Should return 5 lines of visible content");
    assert!(visible[0].contains("Line 1"), "First visible line should be line 1");
    assert!(visible[4].contains("Line 5"), "Last visible line should be line 5");
    
    // Scroll down and test visible content
    viewer.scroll_to_position(10);
    let visible = viewer.visible_content(5);
    assert_eq!(visible.len(), 5, "Should return 5 lines of visible content");
    assert!(visible[0].contains("Line 11"), "First visible line should be line 11");
    assert!(visible[4].contains("Line 15"), "Last visible line should be line 15");
    
    // Test near the end
    viewer.scroll_to_position(96);
    let visible = viewer.visible_content(10);
    assert_eq!(visible.len(), 4, "Should return only the remaining lines (4)");
    assert!(visible[0].contains("Line 97"), "First visible line should be line 97");
    assert!(visible[3].contains("Line 100"), "Last visible line should be line 100");
    
    Ok(())
}

#[test]
fn test_viewer_state_persistence() -> Result<()> {
    let (_temp_dir, _, _, large_file_path) = setup_test_files()?;
    
    // Create a new viewer
    let mut viewer = Viewer::new();
    
    // Open the large file
    viewer.open_file(&large_file_path)?;
    
    // Scroll to position 50
    viewer.scroll_to_position(50);
    assert_eq!(viewer.scroll_position(), 50, "Scroll position should be 50");
    
    // Open another file
    let (_temp_dir2, small_file_path, _, _) = setup_test_files()?;
    viewer.open_file(&small_file_path)?;
    
    // Scroll position should reset
    assert_eq!(viewer.scroll_position(), 0, "Scroll position should reset to 0 for new file");
    
    // Reopen the first file
    viewer.open_file(&large_file_path)?;
    
    // Scroll position should also reset (this is expected behavior, we're not caching state)
    assert_eq!(viewer.scroll_position(), 0, "Scroll position should reset to 0 when reopening a file");
    
    Ok(())
}

#[test]
fn test_viewer_selection() -> Result<()> {
    let (_temp_dir, small_file_path, _, _) = setup_test_files()?;
    
    // Create a new viewer
    let mut viewer = Viewer::new();
    
    // Open the small file (3 lines)
    viewer.open_file(&small_file_path)?;
    
    // Initially no selection
    assert_eq!(viewer.selection_range(), None, "Should have no selection initially");
    assert_eq!(viewer.is_selection_mode(), false, "Selection mode should be off initially");
    
    // Turn on selection mode
    viewer.toggle_selection_mode();
    assert_eq!(viewer.is_selection_mode(), true, "Selection mode should be on after toggle");
    
    // Initial selection range should be cursor position only
    assert_eq!(viewer.selection_range(), Some((0, 0)), "Initial selection should be cursor position only");
    
    // Move cursor to select more lines
    viewer.cursor_down();
    assert_eq!(viewer.cursor_position(), 1, "Cursor should be at position 1");
    assert_eq!(viewer.selection_range(), Some((0, 1)), "Selection range should be from 0 to 1");
    
    // Move cursor down again
    viewer.cursor_down();
    assert_eq!(viewer.cursor_position(), 2, "Cursor should be at position 2");
    assert_eq!(viewer.selection_range(), Some((0, 2)), "Selection range should be from 0 to 2");
    
    // Exit selection mode
    viewer.toggle_selection_mode();
    assert_eq!(viewer.is_selection_mode(), false, "Selection mode should be off after second toggle");
    
    // Selection range should be cleared when exiting selection mode
    assert_eq!(viewer.selection_range(), None, "Selection range should be cleared after exiting selection mode");
    
    // Start new selection from middle
    viewer.cursor_up();
    assert_eq!(viewer.cursor_position(), 1, "Cursor should be at position 1");
    
    // Start a new selection
    viewer.toggle_selection_mode();
    assert_eq!(viewer.selection_range(), Some((1, 1)), "New selection should start at current cursor position");
    
    // Select upward
    viewer.cursor_up();
    assert_eq!(viewer.selection_range(), Some((0, 1)), "Selection range should be from 0 to 1, updated for upward selection");
    
    // Test clear selection
    viewer.clear_selection();
    assert_eq!(viewer.is_selection_mode(), false, "Selection mode should be off after clear");
    assert_eq!(viewer.selection_range(), None, "Selection range should be cleared");
    
    // Test selection reset when opening a new file
    viewer.toggle_selection_mode();
    viewer.cursor_down();
    assert_eq!(viewer.selection_range(), Some((0, 1)), "Selection should be active again");
    
    // Open the file again - should reset selection
    viewer.open_file(&small_file_path)?;
    assert_eq!(viewer.is_selection_mode(), false, "Selection mode should reset after opening a file");
    assert_eq!(viewer.selection_range(), None, "Selection range should reset after opening a file");
    
    Ok(())
}