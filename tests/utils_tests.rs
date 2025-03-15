use std::path::Path;
use packrat::utils;

#[test]
fn test_generate_chunk_filename_basic() {
    // Simple file in root directory
    let file_path = Path::new("/root/test.py");
    let root_path = Path::new("/root");
    let start = 2; // 0-indexed (line 3)
    let end = 9;   // 0-indexed (line 10)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "test_py_3-10.txt");
}

#[test]
fn test_generate_chunk_filename_nested_directory() {
    // File in nested directory
    let file_path = Path::new("/projects/example/data.csv");
    let root_path = Path::new("/projects");
    let start = 14; // 0-indexed (line 15)
    let end = 19;   // 0-indexed (line 20)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "example_data_csv_15-20.txt");
    
    // More deeply nested directory
    let file_path = Path::new("/projects/module1/submodule/component/utils.js");
    let root_path = Path::new("/projects");
    let start = 7;  // 0-indexed (line 8)
    let end = 15;   // 0-indexed (line 16)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "module1_submodule_component_utils_js_8-16.txt");
}

#[test]
fn test_generate_chunk_filename_special_characters() {
    // File with special characters in name
    let file_path = Path::new("/root/data-set.v1.2:test.log");
    let root_path = Path::new("/root");
    let start = 0; // 0-indexed (line 1)
    let end = 4;   // 0-indexed (line 5)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "data_set_v1_2_test_log_1-5.txt");
    
    // File with spaces and other special chars
    let file_path = Path::new("/root/My Document (1).md");
    let root_path = Path::new("/root");
    let start = 5;  // 0-indexed (line 6)
    let end = 10;   // 0-indexed (line 11)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    // Parentheses should be preserved but spaces converted to underscores
    assert!(result.contains("My_Document"), "Spaces should be converted to underscores: {}", result);
    assert!(result.ends_with("_6-11.txt"), "Line numbers should be 1-indexed: {}", result);
}

#[test]
fn test_generate_chunk_filename_path_not_in_root() {
    // Path not within root (should use absolute path)
    let file_path = Path::new("/other/path/file.txt");
    let root_path = Path::new("/root");
    let start = 9;   // 0-indexed (line 10)
    let end = 19;    // 0-indexed (line 20)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "other_path_file_txt_10-20.txt");
}

#[test]
fn test_generate_chunk_filename_line_number_conversion() {
    // Test line number conversion (0-indexed to 1-indexed)
    let file_path = Path::new("/test/file.txt");
    let root_path = Path::new("/test");
    
    // First line
    let result = utils::generate_chunk_filename(file_path, root_path, 0, 0);
    assert_eq!(result, "file_txt_1-1.txt", "First line should be converted to 1");
    
    // Single line selection in middle of file
    let result = utils::generate_chunk_filename(file_path, root_path, 99, 99);
    assert_eq!(result, "file_txt_100-100.txt", "Line 100 selection");
    
    // Large range
    let result = utils::generate_chunk_filename(file_path, root_path, 9, 999);
    assert_eq!(result, "file_txt_10-1000.txt", "Large range selection");
}

#[test]
#[cfg(windows)]
fn test_generate_chunk_filename_windows_paths() {
    // Windows-style paths with backslashes - Windows specific test
    let file_path = Path::new("C:\\Users\\test\\Document.docx");
    let root_path = Path::new("C:\\Users");
    let start = 0;   // 0-indexed (line 1)
    let end = 9;     // 0-indexed (line 10)
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "test_Document_docx_1-10.txt");
    
    // Windows path with mixed separators
    let file_path = Path::new("C:\\Projects/src\\main/index.ts");
    let root_path = Path::new("C:\\Projects");
    let start = 5;
    let end = 15;
    
    let result = utils::generate_chunk_filename(file_path, root_path, start, end);
    assert_eq!(result, "src_main_index_ts_6-16.txt");
}

#[test]
fn test_truncate_string() {
    // Test the truncate_string function
    
    // String shorter than max length
    let s = "Hello";
    let result = utils::truncate_string(s, 10);
    assert_eq!(result, "Hello");
    
    // String equal to max length
    let result = utils::truncate_string(s, 5);
    assert_eq!(result, "Hello");
    
    // String longer than max length
    let s = "Hello, world!";
    let result = utils::truncate_string(s, 8);
    assert_eq!(result, "Hello...");
    
    // Very short max length (less than ellipsis)
    let result = utils::truncate_string(s, 2);
    assert_eq!(result, "He");
}