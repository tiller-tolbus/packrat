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