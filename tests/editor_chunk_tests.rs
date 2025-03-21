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

#[test]
fn test_edited_content_in_chunk() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Select lines 3-5 (indexes 2-4)
    viewer.toggle_selection_mode();
    viewer.cursor_down(); // Line 2
    viewer.cursor_down(); // Line 3
    viewer.cursor_down(); // Line 4
    viewer.cursor_down(); // Line 5
    
    // Get the selection range
    let selection_range = viewer.selection_range().unwrap();
    assert_eq!(selection_range, (0, 4), "Selection range should be lines 1-5");
    
    // Get the selected lines
    let content = viewer.content();
    let selected_lines = content[selection_range.0..=selection_range.1].to_vec();
    
    // Create an editor and set the selected content
    let mut editor = Editor::new();
    editor.set_content(selected_lines);
    
    // Make edits to the content in the editor
    // Enter insert mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
    
    // Add prefix to first line - add "EDITED: " at the beginning
    for c in "EDITED: ".chars() {
        editor.handle_key_event(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
    }
    
    // Return to normal mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    
    // Verify editor content shows the changes
    let edited_content = editor.content();
    assert!(edited_content[0].starts_with("EDITED: "), "First line should be edited");
    assert!(editor.is_modified(), "Editor should mark content as modified");
    
    // Update the viewer with the edited content
    assert!(viewer.update_selected_content(edited_content.clone()));
    
    // Verify viewer content has been updated
    let updated_viewer_content = viewer.content();
    assert!(updated_viewer_content[0].starts_with("EDITED: "), "Viewer content should be updated with edited text");
    
    // Save the chunk with edited content
    let chunk_id = viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Verify the chunk was saved (ID is returned)
    assert!(!chunk_id.is_empty(), "Chunk ID should not be empty");
    
    // Get the saved chunk and verify its content
    let chunks = chunk_storage.get_chunks();
    assert_eq!(chunks.len(), 1, "Should have one chunk in storage");
    
    // Get the chunk and check its content
    let chunk = &chunks[0];
    assert!(chunk.content.contains("EDITED: "), "Chunk should contain the edited content");
    
    // Check that the viewer's has_edited_content flag is set
    assert!(viewer.has_edited_content(), "Viewer should mark content as edited");
    
    // Check that the chunk's edited flag is set
    assert!(chunk.edited, "Chunk edited flag should be true");
    
    Ok(())
}

#[test]
fn test_editor_to_chunk_workflow() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Select lines 10-15 (indexes 9-14)
    // Move to line 10 (index 9)
    viewer.scroll_to_top(); // Reset position
    for _ in 0..9 {
        viewer.cursor_down(); // Move to line 10
    }
    viewer.toggle_selection_mode();
    for _ in 0..5 {
        viewer.cursor_down(); // Move to line 15
    }
    
    // Get the selection range
    let selection_range = viewer.selection_range().unwrap();
    assert_eq!(selection_range, (9, 14), "Selection range should be lines 10-15");
    
    // Get original content for comparison
    let original_content = viewer.content()[selection_range.0..=selection_range.1].to_vec();
    
    // Create an editor and set the selected content
    let mut editor = Editor::new();
    editor.set_content(original_content.clone());
    
    // Make significant edits to the content in the editor
    // 1. Replace lines 10-12 (first 3 lines of selection) with completely new content
    
    // Go to normal mode and position cursor at start
    editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    
    // Enter command mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Char(':'), KeyModifiers::empty()));
    
    // Type command to delete 3 lines (":1,3d")
    for c in "1,3d".chars() {
        editor.handle_key_event(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
    }
    editor.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    
    // Now add new content at the beginning
    // Enter insert mode at first line
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('O'), KeyModifiers::empty()));
    
    // Add three new lines
    for line in &[
        "This is completely new line 1",
        "This is completely new line 2",
        "This is completely new line 3"
    ] {
        // Type the line
        for c in line.chars() {
            editor.handle_key_event(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
        }
        // Add newline
        editor.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    }
    
    // Return to normal mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    
    // Verify editor content has been modified
    let edited_content = editor.content();
    assert_ne!(edited_content, original_content, "Content should be different after editing");
    assert!(editor.is_modified(), "Editor should mark content as modified");
    
    // Update the viewer with the edited content
    assert!(viewer.update_selected_content(edited_content.clone()));
    
    // Save the chunk with edited content
    let _chunk_id = viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    // Get the saved chunk
    let chunks = chunk_storage.get_chunks();
    assert_eq!(chunks.len(), 1, "Should have one chunk in storage");
    
    // Verify the chunk content
    let chunk = &chunks[0];
    
    // Check that the chunk contains the new content
    assert!(chunk.content.contains("This is completely new line 1"), "Chunk should contain the new content");
    assert!(chunk.content.contains("This is completely new line 2"), "Chunk should contain the new content");
    assert!(chunk.content.contains("This is completely new line 3"), "Chunk should contain the new content");
    
    // Final check - load content in a new viewer to ensure the chunk can be properly loaded
    let mut new_viewer = Viewer::new();
    new_viewer.open_file(&test_file_path)?;
    new_viewer.load_chunked_ranges(&chunk_storage, &root_path)?;
    
    // Verify the chunked ranges are loaded correctly
    let chunked_ranges = new_viewer.chunked_ranges();
    assert_eq!(chunked_ranges.len(), 1, "Should have one chunked range");
    
    // Check that the line range matches the original selection (using 0-indexed values)
    assert_eq!(chunked_ranges[0], (9, 14), "Should have chunked range for lines 10-15");
    
    Ok(())
}