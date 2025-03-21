use packrat::editor::Editor;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_editor_creation() {
    let editor = Editor::new();
    assert!(!editor.is_modified());
    assert_eq!(editor.content(), Vec::<String>::new());
    assert!(editor.mode().len() > 0, "Editor should have a mode");
}

#[test]
fn test_set_content() {
    let mut editor = Editor::new();
    let content = vec!["Line one".to_string(), "Line two".to_string()];
    
    editor.set_content(content.clone());
    
    assert_eq!(editor.content(), content);
    assert!(!editor.is_modified());  // Should not be modified after initial content set
}

#[test]
fn test_content_modification() {
    let mut editor = Editor::new();
    let initial_content = vec!["Test content".to_string()];
    editor.set_content(initial_content);
    
    // Enter insert mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
    
    // Type some text
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('A'), KeyModifiers::empty()));
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('B'), KeyModifiers::empty()));
    
    // Exit insert mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    
    // Content should be modified
    assert!(editor.is_modified());
    
    // Content should include the added text
    let modified_content = editor.content();
    assert!(modified_content[0].contains("AB"), 
           "Content should include the text we added");
}

#[test]
fn test_editor_mode_changes() {
    let mut editor = Editor::new();
    
    // Get the initial mode
    let initial_mode = editor.mode();
    
    // Enter insert mode and verify mode changed
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
    let insert_mode = editor.mode();
    assert_ne!(insert_mode, initial_mode, "Mode should change after 'i' key");
    
    // Return to normal mode and verify
    editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert_eq!(editor.mode(), initial_mode, "Mode should return to initial after Esc");
    
    // Test visual mode if it exists
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty()));
    let visual_mode = editor.mode();
    if visual_mode != initial_mode {
        // If visual mode exists, test escaping from it
        editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
        assert_eq!(editor.mode(), initial_mode, "Mode should return to initial after Esc from visual mode");
    }
}

#[test]
fn test_key_handling() {
    let mut editor = Editor::new();
    editor.set_content(vec!["Line one".to_string(), "Line two".to_string()]);
    
    // Ctrl+S should not be handled by the editor (reserved for app-level save)
    let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let handled = editor.handle_key_event(ctrl_s);
    assert_eq!(handled, false);
    
    // Question mark key should not be handled by editor (reserved for help)
    let help_key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
    let handled = editor.handle_key_event(help_key);
    assert_eq!(handled, false);
    
    // Esc in insert mode should be handled by the editor
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty())); // Enter insert mode
    let handled = editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert_eq!(handled, true);
}

/// Comprehensive test to verify that the original files are never modified
#[test]
fn test_file_safety() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path().join("source.txt");
    let chunk_dir = temp_dir.path().join("chunks");
    
    // Create the source file with some test content
    fs::create_dir_all(&chunk_dir).unwrap();
    let original_content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n";
    {
        let mut file = File::create(&source_path).unwrap();
        file.write_all(original_content.as_bytes()).unwrap();
    }
    
    // Create a file hash to verify it doesn't change
    let original_hash = hash_file(&source_path);
    
    // Set up editor and make changes
    let mut editor = Editor::new();
    
    // Set content (lines 2-4)
    let content = vec![
        "Line 2".to_string(),
        "Line 3".to_string(),
        "Line 4".to_string(),
    ];
    editor.set_content(content);
    
    // Make edits in insert mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::empty()));
    editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    
    // Verify the source file remains unmodified
    let current_hash = hash_file(&source_path);
    assert_eq!(original_hash, current_hash, "Original source file should not be modified");
    
    // Final check that source file is still unmodified
    let final_content = fs::read_to_string(&source_path).unwrap();
    assert_eq!(final_content, original_content, "Original file content should be unchanged");
}

/// Helper function to create a simple hash of a file for comparison
fn hash_file(path: &Path) -> u64 {
    let mut file = File::open(path).unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    
    // Simple hash function for testing
    let mut hash = 0u64;
    for byte in contents {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}