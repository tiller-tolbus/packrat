use anyhow::Result;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

use packrat::viewer::Viewer;

fn setup_test_files() -> Result<(tempfile::TempDir, PathBuf, PathBuf, PathBuf)> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    
    let small_file_path = root_path.join("small_file.txt");
    let mut small_file = File::create(&small_file_path)?;
    writeln!(small_file, "Line 1: This is a small text file.")?;
    writeln!(small_file, "Line 2: It has only a few lines.")?;
    writeln!(small_file, "Line 3: Perfect for basic tests.")?;
    
    let varied_file_path = root_path.join("varied_lines.txt");
    let mut varied_file = File::create(&varied_file_path)?;
    writeln!(varied_file, "Short line")?;
    writeln!(varied_file, "This is a medium length line that has a bit more content.")?;
    writeln!(varied_file, "This is a much longer line that contains significantly more text which should test how the viewer handles longer content that might need to be wrapped or displayed differently depending on the terminal width.")?;
    writeln!(varied_file, "Another short line")?;
    writeln!(varied_file, "Yet another line with medium length content.")?;
    
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
    
    let mut viewer = Viewer::new();
    
    viewer.open_file(&small_file_path)?;
    
    assert_eq!(viewer.file_path(), Some(small_file_path.as_path()));
    
    // Verify content is loaded (without asserting exact content or line count)
    let content = viewer.content();
    assert!(!content.is_empty(), "Viewer should have loaded content");
    assert!(content.iter().any(|line| line.contains("Line 1")), "Content should include text from the file");
    
    Ok(())
}

#[test]
fn test_viewer_scrolling_functionality() -> Result<()> {
    let (_temp_dir, _, _, large_file_path) = setup_test_files()?;
    
    let mut viewer = Viewer::new();
    viewer.open_file(&large_file_path)?;
    
    // Test basic scrolling works in the expected direction
    let initial_position = viewer.scroll_position();
    viewer.scroll_down();
    assert!(viewer.scroll_position() > initial_position, "Scrolling down should increase position");
    
    viewer.scroll_up();
    assert_eq!(viewer.scroll_position(), initial_position, "Scrolling up should decrease position");
    
    // Test page scrolling moves multiple lines
    let before_page_down = viewer.scroll_position();
    viewer.scroll_page_down(10);
    assert!(viewer.scroll_position() > before_page_down, "Page down should move multiple lines");
    assert!(viewer.scroll_position() > before_page_down + 1, "Page down should move more than one line");
    
    let before_page_up = viewer.scroll_position();
    viewer.scroll_page_up(10);
    assert!(viewer.scroll_position() < before_page_up, "Page up should move position back");
    
    // Test scrolling to extremes
    viewer.scroll_to_top();
    assert_eq!(viewer.scroll_position(), 0, "Scroll to top should set position to 0");
    
    viewer.scroll_to_bottom();
    assert!(viewer.scroll_position() > 0, "Scroll to bottom should move position to end");
    
    Ok(())
}

#[test]
fn test_viewer_boundary_behaviors() -> Result<()> {
    let (_temp_dir, small_file_path, _, _) = setup_test_files()?;
    
    let mut viewer = Viewer::new();
    viewer.open_file(&small_file_path)?;
    
    // Test scrolling beyond file boundaries
    viewer.scroll_to_top();
    for _ in 0..10 {
        viewer.scroll_up();
    }
    assert_eq!(viewer.scroll_position(), 0, "Should not scroll above file start");
    
    // Scroll beyond end of the file
    viewer.scroll_to_bottom();
    let max_position = viewer.scroll_position();
    for _ in 0..10 {
        viewer.scroll_down();
    }
    assert_eq!(viewer.scroll_position(), max_position, "Should not scroll beyond file end");
    
    // Test large page moves
    viewer.scroll_to_top();
    viewer.scroll_page_down(1000); // Very large page size
    assert_eq!(viewer.scroll_position(), max_position, "Should limit scrolling to file end");
    
    viewer.scroll_page_up(1000); // Very large page size
    assert_eq!(viewer.scroll_position(), 0, "Should limit scrolling to file start");
    
    Ok(())
}

#[test]
fn test_viewer_file_switching() -> Result<()> {
    let (_temp_dir, small_file_path, _, large_file_path) = setup_test_files()?;
    
    let mut viewer = Viewer::new();
    
    // Open first file
    viewer.open_file(&large_file_path)?;
    assert!(viewer.content().len() > 5, "Large file should have multiple lines");
    
    // Move to a non-zero position
    viewer.scroll_to_position(5);
    assert!(viewer.scroll_position() > 0, "Should have moved to non-zero position");
    
    // Switch to a different file
    viewer.open_file(&small_file_path)?;
    
    // Should reset scroll position
    assert_eq!(viewer.scroll_position(), 0, "Scroll position should reset when opening a new file");
    
    // Content should be updated
    assert!(viewer.content().len() < 10, "Small file should have fewer lines");
    
    Ok(())
}