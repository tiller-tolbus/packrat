use anyhow::Result;
use packrat::editor::Editor;
use packrat::viewer::Viewer;
use packrat::utils::generate_chunk_filename;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tempfile::tempdir;

/// Setup a test environment for chunk editing tests
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
fn test_edited_content_in_chunk() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, chunks_dir) = setup_test_environment()?;
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
    let chunk_path = viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Verify the chunk file exists and has the expected name
    assert!(chunk_path.exists(), "Chunk file should exist");
    
    // Expected filename is test_file_txt_1-5.txt (1-indexed in filename)
    let expected_filename = generate_chunk_filename(&test_file_path, &root_path, 0, 4);
    assert_eq!(chunk_path.file_name().unwrap().to_string_lossy(), expected_filename);
    
    // Read and verify the chunk content
    let mut chunk_content = String::new();
    File::open(&chunk_path)?.read_to_string(&mut chunk_content)?;
    
    // Check that the chunk contains the edited content
    assert!(chunk_content.contains("EDITED: "), "Chunk should contain the edited content");
    
    // Check that the viewer's has_edited_content flag is set
    assert!(viewer.has_edited_content(), "Viewer should mark content as edited");
    
    Ok(())
}

#[test]
fn test_editor_to_chunk_workflow() -> Result<()> {
    // Setup test environment
    let (_temp_dir, root_path, chunks_dir) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    // Create a viewer and open the test file
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    // Select lines 10-15 (indexes 9-14)
    viewer.cursor_position = 9; // Set to line 10
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
    let chunk_path = viewer.save_selection_as_chunk(&chunks_dir, &root_path)?;
    
    // Verify the chunk file exists
    assert!(chunk_path.exists(), "Chunk file should exist");
    
    // Read and verify the chunk content
    let mut chunk_content = String::new();
    File::open(&chunk_path)?.read_to_string(&mut chunk_content)?;
    
    // Check that the chunk contains the new content
    assert!(chunk_content.contains("This is completely new line 1"), "Chunk should contain the new content");
    assert!(chunk_content.contains("This is completely new line 2"), "Chunk should contain the new content");
    assert!(chunk_content.contains("This is completely new line 3"), "Chunk should contain the new content");
    
    // Final check - load content in a new viewer to ensure the chunk can be properly loaded
    let mut new_viewer = Viewer::new();
    new_viewer.open_file(&test_file_path)?;
    new_viewer.load_chunked_ranges(&chunks_dir, &root_path)?;
    
    // Verify the chunked ranges are loaded correctly
    let chunked_ranges = new_viewer.chunked_ranges();
    assert_eq!(chunked_ranges.len(), 1, "Should have one chunked range");
    
    // Check that the line range matches the original selection
    let has_range = chunked_ranges.contains(&(9, 14)) || chunked_ranges.contains(&(14, 9));
    assert!(has_range, "Should have chunked range for lines 10-15");
    
    Ok(())
}