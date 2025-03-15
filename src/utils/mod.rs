// Utility functions for the application
// Will be expanded as needed

/// Truncate a string to a maximum length, adding ellipsis if truncated
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Make sure we have room for the ellipsis
        if max_len < 3 {
            return s[0..max_len].to_string();
        }
        
        format!("{}...", &s[0..(max_len - 3)])
    }
}

/// Generate a chunk filename from a file path, start line, and end line
/// 
/// The filename follows the format: path_from_root_converted_to_underscores_START-END.txt
/// 
/// For example:
/// - /foo/bar.py with lines 3-10 becomes foo_bar_py_3-10.txt
/// - /projects/example/data.csv with lines 15-20 becomes projects_example_data_csv_15-20.txt
/// 
/// All paths are relative to the root directory provided to the function.
/// 
/// # Arguments
/// * `file_path` - Path to the file (absolute or relative)
/// * `root_path` - Root directory path for the application
/// * `start_line` - Starting line number (0-indexed internally, converted to 1-indexed for filename)
/// * `end_line` - Ending line number (0-indexed internally, converted to 1-indexed for filename)
/// 
/// # Returns
/// The generated chunk filename as a String
pub fn generate_chunk_filename(file_path: &std::path::Path, root_path: &std::path::Path, start_line: usize, end_line: usize) -> String {
    // Convert file_path to be relative to root_path
    let relative_path = if file_path.starts_with(root_path) {
        file_path.strip_prefix(root_path).unwrap_or(file_path)
    } else {
        file_path
    };
    
    // Convert path separators and special characters to underscores
    let path_str = relative_path.to_string_lossy();
    let sanitized_path = path_str
        .replace(['/', '\\'], "_") // Replace path separators with underscores
        .replace(['.', ' ', '-', ':', '+'], "_"); // Replace other special characters
    
    // Remove leading underscore if present (from absolute paths)
    let sanitized_path = sanitized_path.trim_start_matches('_');
    
    // Add line range (converting from 0-indexed to 1-indexed for user-facing numbers)
    format!("{}_{}-{}.txt", sanitized_path, start_line + 1, end_line + 1)
}