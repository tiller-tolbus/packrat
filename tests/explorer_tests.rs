use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use std::env;
use tempfile::tempdir;

// Import the Explorer from the main crate
use packrat::explorer::Explorer;

// Helper function to create a test directory structure
fn setup_test_directory() -> Result<(tempfile::TempDir, PathBuf)> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    
    // Create subdirectories
    fs::create_dir(root_path.join("dir1"))?;
    fs::create_dir(root_path.join("dir2"))?;
    
    // Create files
    let mut file1 = File::create(root_path.join("file1.txt"))?;
    write!(file1, "This is file 1")?;
    
    let mut file2 = File::create(root_path.join("file2.txt"))?;
    write!(file2, "This is file 2")?;
    
    let mut file3 = File::create(root_path.join("dir1/file3.txt"))?;
    write!(file3, "This is file 3 in dir1")?;
    
    Ok((temp_dir, root_path))
}

#[test]
fn test_explorer_creation() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer with the test directory as root
    let explorer = Explorer::new(&root_path)?;
    
    // Check that it loaded entries correctly
    let entries = explorer.entries();
    assert!(!entries.is_empty(), "Explorer should have loaded entries");
    
    // There should be 4 entries: 2 directories and 2 files
    assert_eq!(entries.len(), 4, "Explorer should have loaded 4 entries");
    
    Ok(())
}

#[test]
fn test_explorer_navigation() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer
    let mut explorer = Explorer::new(&root_path)?;
    
    // Initial state check
    assert_eq!(explorer.selected_index(), 0, "Initial selection should be 0");
    
    // Test navigation down
    explorer.select_next();
    assert_eq!(explorer.selected_index(), 1, "Selection should move down");
    
    explorer.select_next();
    assert_eq!(explorer.selected_index(), 2, "Selection should move down");
    
    // Test navigation to the end
    explorer.select_next();
    explorer.select_next();
    explorer.select_next(); // Try to go past the end
    assert_eq!(explorer.selected_index(), 3, "Selection should not go past the end");
    
    // Test navigation up
    explorer.select_previous();
    assert_eq!(explorer.selected_index(), 2, "Selection should move up");
    
    explorer.select_previous();
    explorer.select_previous();
    explorer.select_previous();
    explorer.select_previous(); // Try to go past the beginning
    assert_eq!(explorer.selected_index(), 0, "Selection should not go past the beginning");
    
    // Test page navigation
    explorer.select_last(); // Go to the end
    assert_eq!(explorer.selected_index(), 3, "Selection should be at the end");
    
    explorer.select_page_up(2); // Go up 2 items
    assert_eq!(explorer.selected_index(), 1, "Selection should move up 2 items");
    
    explorer.select_page_down(3); // Go down 3 items (limited by list size)
    assert_eq!(explorer.selected_index(), 3, "Selection should be at the end again");
    
    // Test home/end navigation
    explorer.select_first();
    assert_eq!(explorer.selected_index(), 0, "Selection should be at the beginning");
    
    explorer.select_last();
    assert_eq!(explorer.selected_index(), 3, "Selection should be at the end");
    
    Ok(())
}

#[test]
fn test_explorer_directory_navigation() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer
    let mut explorer = Explorer::new(&root_path)?;
    
    // Find a directory entry (dir1)
    let dir_index = explorer.entries().iter().position(|e| e.name == "dir1")
        .expect("Should find dir1");
    
    // Navigate to the directory
    for _ in 0..dir_index {
        explorer.select_next();
    }
    
    // Open the directory
    explorer.open_selected()?;
    
    // Check that we're now in dir1
    let entries = explorer.entries();
    
    // There should be 2 entries: parent directory and file3.txt
    assert_eq!(entries.len(), 2, "Explorer should have loaded 2 entries in dir1");
    
    // First entry should be ".." for parent directory
    assert_eq!(entries[0].name, "..", "First entry should be parent directory");
    
    // Second entry should be file3.txt
    assert_eq!(entries[1].name, "file3.txt", "Second entry should be file3.txt");
    
    // Now navigate back up
    explorer.go_to_parent()?;
    
    // We should be back in the root with 4 entries
    assert_eq!(explorer.entries().len(), 4, "Explorer should be back in root with 4 entries");
    
    Ok(())
}

#[test]
fn test_explorer_chroot_enforcement() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer with the test directory as root
    let mut explorer = Explorer::new(&root_path)?;
    
    // Try to navigate above root
    explorer.go_to_parent()?;
    
    // Check that we're still at the root by checking entries
    let entries = explorer.entries();
    assert_eq!(entries.len(), 4, "Explorer should have 4 entries at root");
    
    // Check if directory entry names match what we set up
    let dir_names: Vec<String> = entries.iter()
        .filter(|e| e.is_dir)
        .map(|e| e.name.clone())
        .collect();
    
    assert!(dir_names.contains(&String::from("dir1")), "Should contain dir1");
    assert!(dir_names.contains(&String::from("dir2")), "Should contain dir2");
    
    Ok(())
}

#[test]
fn test_explorer_entry_sorting() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer
    let explorer = Explorer::new(&root_path)?;
    
    // Check that directories are listed before files
    let entries = explorer.entries();
    
    // The first entries should be directories
    assert!(entries[0].is_dir, "First entry should be a directory");
    assert!(entries[1].is_dir, "Second entry should be a directory");
    
    // The next entries should be files
    assert!(!entries[2].is_dir, "Third entry should be a file");
    assert!(!entries[3].is_dir, "Fourth entry should be a file");
    
    Ok(())
}