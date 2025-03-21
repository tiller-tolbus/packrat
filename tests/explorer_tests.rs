use anyhow::Result;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::Write;
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

// Helper to compare paths without being affected by symlinks, prefixes, etc.
fn is_same_path<P1: AsRef<std::path::Path>, P2: AsRef<std::path::Path>>(path1: P1, path2: P2) -> bool {
    let p1 = path1.as_ref();
    let p2 = path2.as_ref();
    
    if p1 == p2 {
        return true;
    }
    
    // Try canonicalization as a fallback
    if let (Ok(canon1), Ok(canon2)) = (p1.canonicalize(), p2.canonicalize()) {
        return canon1 == canon2;
    }
    
    false
}

#[test]
fn test_explorer_creation() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer with the test directory as root
    let explorer = Explorer::new(&root_path)?;
    
    // Check that it loaded entries correctly
    let entries = explorer.entries();
    assert!(!entries.is_empty(), "Explorer should have loaded entries");
    
    // Verify both directories and files are loaded
    let has_directories = entries.iter().any(|e| e.is_dir);
    let has_files = entries.iter().any(|e| !e.is_dir);
    
    assert!(has_directories, "Explorer should have loaded directories");
    assert!(has_files, "Explorer should have loaded files");
    
    Ok(())
}

#[test]
fn test_explorer_navigation() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer
    let mut explorer = Explorer::new(&root_path)?;
    
    // Initial state check
    assert_eq!(explorer.selected_index(), 0, "Initial selection should be 0");
    
    // Test basic navigation
    explorer.select_next();
    assert!(explorer.selected_index() > 0, "Selection should move after select_next()");
    
    explorer.select_previous();
    assert_eq!(explorer.selected_index(), 0, "Selection should move back after select_previous()");
    
    // Test navigation boundaries
    let entry_count = explorer.entries().len();
    
    // Go to last entry
    explorer.select_last();
    assert_eq!(explorer.selected_index(), entry_count - 1, "select_last() should go to the last entry");
    
    // Try to go beyond the end
    explorer.select_next();
    assert_eq!(explorer.selected_index(), entry_count - 1, "Cannot go beyond the last entry");
    
    // Go back to first
    explorer.select_first();
    assert_eq!(explorer.selected_index(), 0, "select_first() should go to the first entry");
    
    // Try to go before the start
    explorer.select_previous();
    assert_eq!(explorer.selected_index(), 0, "Cannot go before the first entry");
    
    // Test page navigation
    explorer.select_page_down(100); // Large number to ensure we reach the end
    assert_eq!(explorer.selected_index(), entry_count - 1, "select_page_down() should respect boundaries");
    
    explorer.select_page_up(100); // Large number to ensure we reach the start
    assert_eq!(explorer.selected_index(), 0, "select_page_up() should respect boundaries");
    
    Ok(())
}

#[test]
fn test_explorer_directory_navigation() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer
    let mut explorer = Explorer::new(&root_path)?;
    
    // Find a directory entry (dir1)
    let dir_index = explorer.entries().iter().position(|e| e.name == "dir1" && e.is_dir)
        .expect("Should find dir1 directory");
    
    // Navigate to the directory
    while explorer.selected_index() != dir_index {
        explorer.select_next();
    }
    
    // Open the directory
    explorer.open_selected()?;
    
    // Check that we're now in dir1
    let current_path = explorer.current_path();
    assert!(current_path.ends_with("dir1"), "Should have navigated to dir1");
    
    // Verify we can see file3.txt
    let file3_exists = explorer.entries().iter().any(|e| e.name == "file3.txt");
    assert!(file3_exists, "Should find file3.txt in dir1");
    
    // Now navigate back up
    explorer.go_to_parent()?;
    
    // We should be back in the root
    assert!(is_same_path(explorer.current_path(), &root_path), 
           "Should be back at root directory");
    
    Ok(())
}

#[test]
fn test_explorer_chroot_enforcement() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer with the test directory as root
    let mut explorer = Explorer::new(&root_path)?;
    
    // Try to navigate above root
    explorer.go_to_parent()?;
    
    // Check that we're still at the root
    assert!(is_same_path(explorer.current_path(), &root_path), 
           "Cannot navigate above root directory");
    
    // Navigate into a subdirectory
    let dir_index = explorer.entries().iter().position(|e| e.name == "dir1" && e.is_dir)
        .expect("Should find dir1 directory");
    
    // Navigate to the directory
    while explorer.selected_index() != dir_index {
        explorer.select_next();
    }
    
    // Open the directory
    explorer.open_selected()?;
    
    // Now navigate back up
    explorer.go_to_parent()?;
    
    // We should be back in the root
    assert!(is_same_path(explorer.current_path(), &root_path),
           "Should be back at root directory");
    
    // Try to go above root again
    explorer.go_to_parent()?;
    
    // Still should be at root
    assert!(is_same_path(explorer.current_path(), &root_path),
           "Cannot navigate above root directory");
    
    Ok(())
}

#[test]
fn test_explorer_file_selection() -> Result<()> {
    let (_temp_dir, root_path) = setup_test_directory()?;
    
    // Create a new explorer
    let mut explorer = Explorer::new(&root_path)?;
    
    // Find a file entry (file1.txt)
    let file_index = explorer.entries().iter().position(|e| e.name == "file1.txt" && !e.is_dir)
        .expect("Should find file1.txt");
    
    // Navigate to the file
    while explorer.selected_index() != file_index {
        explorer.select_next();
    }
    
    // Check that we've selected a file, not a directory
    let selected = &explorer.entries()[explorer.selected_index()];
    assert!(!selected.is_dir, "Selected entry should be a file");
    assert_eq!(selected.name, "file1.txt", "Selected entry should be file1.txt");
    
    Ok(())
}