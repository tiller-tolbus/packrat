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
    
    let content = viewer.content();
    assert_eq!(content.len(), 3);
    assert!(content[0].contains("Line 1"));
    assert!(content[1].contains("Line 2"));
    assert!(content[2].contains("Line 3"));
    
    assert_eq!(viewer.scroll_position(), 0);
    
    Ok(())
}

#[test]
fn test_viewer_scrolling() -> Result<()> {
    let (_temp_dir, _, _, large_file_path) = setup_test_files()?;
    
    let mut viewer = Viewer::new();
    
    viewer.open_file(&large_file_path)?;
    
    assert_eq!(viewer.scroll_position(), 0);
    
    viewer.scroll_down();
    assert_eq!(viewer.scroll_position(), 1);
    
    for _ in 0..5 {
        viewer.scroll_down();
    }
    assert_eq!(viewer.scroll_position(), 6);
    
    viewer.scroll_up();
    assert_eq!(viewer.scroll_position(), 5);
    
    viewer.scroll_page_down(10);
    assert_eq!(viewer.scroll_position(), 15);
    
    viewer.scroll_page_up(10);
    assert_eq!(viewer.scroll_position(), 5);
    
    viewer.scroll_to_bottom();
    assert_eq!(viewer.scroll_position(), 99);
    
    viewer.scroll_to_top();
    assert_eq!(viewer.scroll_position(), 0);
    
    Ok(())
}

#[test]
fn test_viewer_boundary_conditions() -> Result<()> {
    let (_temp_dir, small_file_path, _, _) = setup_test_files()?;
    
    let mut viewer = Viewer::new();
    
    viewer.open_file(&small_file_path)?;
    
    for _ in 0..10 {
        viewer.scroll_down();
    }
    assert_eq!(viewer.scroll_position(), 2);
    
    viewer.scroll_to_top();
    for _ in 0..10 {
        viewer.scroll_up();
    }
    assert_eq!(viewer.scroll_position(), 0);
    
    viewer.scroll_page_down(100);
    assert_eq!(viewer.scroll_position(), 2);
    
    viewer.scroll_page_up(100);
    assert_eq!(viewer.scroll_position(), 0);
    
    Ok(())
}


#[test]
fn test_viewer_state_persistence() -> Result<()> {
    let (_temp_dir, _, _, large_file_path) = setup_test_files()?;
    
    let mut viewer = Viewer::new();
    
    viewer.open_file(&large_file_path)?;
    
    viewer.scroll_to_position(50);
    assert_eq!(viewer.scroll_position(), 50);
    
    let (_temp_dir2, small_file_path, _, _) = setup_test_files()?;
    viewer.open_file(&small_file_path)?;
    
    assert_eq!(viewer.scroll_position(), 0);
    
    viewer.open_file(&large_file_path)?;
    
    assert_eq!(viewer.scroll_position(), 0);
    
    Ok(())
}

