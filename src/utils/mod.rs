pub mod tokenizer;
pub use tokenizer::*;

#[allow(dead_code)]
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        if max_len < 3 {
            return s.chars().take(max_len).collect();
        }
        
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

pub fn generate_chunk_filename(file_path: &std::path::Path, root_path: &std::path::Path, start_line: usize, end_line: usize) -> String {
    let relative_path = if file_path.starts_with(root_path) {
        match file_path.strip_prefix(root_path) {
            Ok(rel_path) => rel_path,
            Err(_) => file_path,
        }
    } else {
        file_path
    };
    
    let path_str = relative_path.to_string_lossy();
    let replaced = path_str
        .replace(['/', '\\'], "_")
        .replace(['.', ' ', '-', ':', '+'], "_");
    let sanitized_path = replaced.trim_start_matches('_');
    
    let sanitized_path = if sanitized_path.is_empty() {
        "unnamed_file"
    } else {
        sanitized_path
    };
    
    format!("{}_{}-{}.txt", sanitized_path, start_line + 1, end_line + 1)
}