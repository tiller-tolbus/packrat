use std::path::Path;
use packrat::utils;

#[test]
fn test_generate_chunk_filename() {
    // Test case 1: Simple file in root directory
    let file_path = Path::new("/root/test.py");
    let root_path = Path::new("/root");
    let start = 2; // 0-indexed (line 3)
    let end = 9;   // 0-indexed (line 10)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "test_py_3-10.txt");
    
    // Test case 2: File in nested directory
    let file_path = Path::new("/projects/example/data.csv");
    let root_path = Path::new("/projects");
    let start = 14; // 0-indexed (line 15)
    let end = 19;   // 0-indexed (line 20)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "example_data_csv_15-20.txt");
    
    // Test case 3: File with special characters
    let file_path = Path::new("/root/data-set.v1.2:test.log");
    let root_path = Path::new("/root");
    let start = 0; // 0-indexed (line 1)
    let end = 4;   // 0-indexed (line 5)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "data_set_v1_2_test_log_1-5.txt");
    
    // Test case 4: Path not within root (should use absolute path)
    let file_path = Path::new("/other/path/file.txt");
    let root_path = Path::new("/root");
    let start = 9;   // 0-indexed (line 10)
    let end = 19;    // 0-indexed (line 20)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "other_path_file_txt_10-20.txt");
    
    // Test case 5: Windows-style paths with backslashes
    let file_path = Path::new("C:\\Users\\test\\Document.docx");
    let root_path = Path::new("C:\\Users");
    let start = 0;   // 0-indexed (line 1)
    let end = 9;     // 0-indexed (line 10)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    // Note: backslashes are handled differently on different platforms,
    // so we verify the essential parts instead
    assert!(result.contains("test_Document_docx"));
    assert!(result.ends_with("_1-10.txt"));
}