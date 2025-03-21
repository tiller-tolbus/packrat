use anyhow::Result;
use std::path::PathBuf;
use std::fs::{self, File};
use std::io::{Read, Write};
use tempfile::tempdir;

use packrat::viewer::Viewer;
use packrat::storage::ChunkStorage;

fn setup_test_environment() -> Result<(tempfile::TempDir, PathBuf, ChunkStorage)> {
    let temp_dir = tempdir()?;
    let root_path = temp_dir.path().to_path_buf();
    
    let csv_path = root_path.join("chunks.csv");
    let chunk_storage = ChunkStorage::new(&csv_path)?;
    
    let test_file_path = root_path.join("test_file.txt");
    let mut test_file = File::create(&test_file_path)?;
    
    for i in 1..=20 {
        writeln!(test_file, "Line {}: This is test content for line {}.", i, i)?;
    }
    
    Ok((temp_dir, root_path, chunk_storage))
}



#[test]
fn test_chunking_with_edited_content() -> Result<()> {
    let (_temp_dir, root_path, mut chunk_storage) = setup_test_environment()?;
    let test_file_path = root_path.join("test_file.txt");
    
    let mut viewer = Viewer::new();
    viewer.open_file(&test_file_path)?;
    
    viewer.toggle_selection_mode();
    for _ in 0..4 {
        viewer.cursor_down();
    }
    viewer.cursor_down();
    viewer.cursor_down();
    
    assert!(viewer.selection_range().is_some());
    
    let edited_content = vec![
        "Line 5: EDITED content for testing.".to_string(),
        "Line 6: EDITED content for testing.".to_string(),
        "Line 7: EDITED content for testing.".to_string(),
    ];
    
    assert!(viewer.update_selected_content(edited_content.clone()));
    
    let _chunk_id = viewer.save_selection_as_chunk(&mut chunk_storage, &root_path)?;
    
    let chunks = chunk_storage.get_chunks();
    assert_eq!(chunks.len(), 1);
    
    let chunk = &chunks[0];
    assert!(chunk.content.contains("Line 5: EDITED content for testing."));
    assert!(chunk.content.contains("Line 6: EDITED content for testing."));
    assert!(chunk.content.contains("Line 7: EDITED content for testing."));
    
    assert!(viewer.has_edited_content());
    
    assert!(chunk.edited);
    
    Ok(())
}


