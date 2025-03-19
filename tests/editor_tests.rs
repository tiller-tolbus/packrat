use packrat::editor::Editor;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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