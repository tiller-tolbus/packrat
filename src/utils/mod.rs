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

