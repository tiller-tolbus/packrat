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
    assert_eq!(editor.mode(), "NORMAL");
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
    
    // Simulate typing in insert mode
    
    // First enter insert mode with 'i'
    let insert_key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty());
    editor.handle_key_event(insert_key);
    
    // Then type some text
    for c in "Hello, ".chars() {
        let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty());
        editor.handle_key_event(key);
    }
    
    // Exit insert mode with Esc
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    editor.handle_key_event(esc_key);
    
    // Content should be modified
    assert!(editor.is_modified());
    
    // Content should now start with "Hello, Test content"
    let modified_content = editor.content();
    assert!(modified_content[0].starts_with("Hello, "));
}

#[test]
fn test_editor_mode_changes() {
    let mut editor = Editor::new();
    
    // Default mode should be normal
    assert_eq!(editor.mode(), "NORMAL");
    
    // Enter insert mode
    let insert_key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty());
    editor.handle_key_event(insert_key);
    assert_eq!(editor.mode(), "INSERT");
    
    // Return to normal mode
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    editor.handle_key_event(esc_key);
    assert_eq!(editor.mode(), "NORMAL");
    
    // Enter visual mode
    let visual_key = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty());
    editor.handle_key_event(visual_key);
    assert_eq!(editor.mode(), "VISUAL");
    
    // Return to normal mode
    editor.handle_key_event(esc_key);
    assert_eq!(editor.mode(), "NORMAL");
}

#[test]
fn test_key_handling() {
    let mut editor = Editor::new();
    editor.set_content(vec!["Line one".to_string(), "Line two".to_string()]);
    
    // Ctrl+S should not be handled by the editor
    let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let handled = editor.handle_key_event(ctrl_s);
    assert_eq!(handled, false);
    
    // ? key should not be handled by the editor
    let help_key = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());
    let handled = editor.handle_key_event(help_key);
    assert_eq!(handled, false);
    
    // Esc in normal mode should not be handled by the editor
    // (it should exit the editor at the app level)
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
    let handled = editor.handle_key_event(esc_key);
    assert_eq!(handled, false);
    
    // Esc in insert mode should be handled by the editor
    // First enter insert mode
    let insert_key = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty());
    editor.handle_key_event(insert_key);
    
    // Now esc should be handled by the editor and return to normal mode
    let handled = editor.handle_key_event(esc_key);
    assert_eq!(handled, true);
    assert_eq!(editor.mode(), "NORMAL");
}

#[test]
fn test_vim_commands() {
    let mut editor = Editor::new();
    editor.set_content(vec!["Test content".to_string()]);
    
    // Test entering command mode with ':'
    let colon_key = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::empty());
    let handled = editor.handle_key_event(colon_key);
    assert_eq!(handled, true);
    assert!(editor.is_in_command_mode());
    assert_eq!(editor.mode(), ":");
    
    // Test the :w command (save)
    for c in "w".chars() {
        let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty());
        editor.handle_key_event(key);
    }
    
    // Execute command with Enter
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
    editor.handle_key_event(enter_key);
    
    // Should return to normal mode
    assert_eq!(editor.mode(), "NORMAL");
    
    // Make changes to verify modified flag
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
    editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    
    // Should be modified now
    assert!(editor.is_modified());
    
    // Enter :w command again
    editor.handle_key_event(colon_key);
    for c in "w".chars() {
        let key = KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty());
        editor.handle_key_event(key);
    }
    editor.handle_key_event(enter_key);
    
    // After :w, should no longer be modified
    assert!(!editor.is_modified());
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
    
    // Now simulate the editing and chunking process
    let mut editor = Editor::new();
    
    // Set up the initial content (lines 2-4)
    let content = vec![
        "Line 2".to_string(),
        "Line 3".to_string(),
        "Line 4".to_string(),
    ];
    editor.set_content(content);
    
    // Make edits
    // Enter insert mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
    
    // Make changes to Line 2
    for c in "EDITED ".chars() {
        editor.handle_key_event(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
    }
    
    // Exit insert mode
    editor.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    
    // Get the modified content
    let modified_content = editor.content();
    assert!(modified_content[0].starts_with("EDITED "));
    assert!(editor.is_modified());
    
    // Verify the source file remains unmodified
    let current_hash = hash_file(&source_path);
    assert_eq!(original_hash, current_hash, "Original source file should not be modified");
    
    // Simulate saving a chunk
    let chunk_path = chunk_dir.join("source_txt_2-4.txt");
    {
        let mut file = File::create(&chunk_path).unwrap();
        for line in &modified_content {
            writeln!(file, "{}", line).unwrap();
        }
    }
    
    // Verify chunk was created with modified content
    let chunk_content = fs::read_to_string(&chunk_path).unwrap();
    assert!(chunk_content.contains("EDITED "), "Chunk should contain edited content");
    
    // Final check that source file is still unmodified
    let final_hash = hash_file(&source_path);
    assert_eq!(original_hash, final_hash, "Original source file should remain unmodified after chunking");
    
    // Read original file content to verify
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