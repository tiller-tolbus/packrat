use std::sync::OnceLock;
use tiktoken_rs::{cl100k_base, CoreBPE};

// Global tokenizer for Claude (cl100k_base)
static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();

/// Get the global Claude tokenizer instance (cl100k_base)
fn get_tokenizer() -> &'static CoreBPE {
    TOKENIZER.get_or_init(|| {
        cl100k_base().expect("Failed to initialize Claude tokenizer")
    })
}

/// Count the number of tokens in a text using Claude's tokenizer
pub fn count_tokens(text: &str) -> usize {
    let tokenizer = get_tokenizer();
    tokenizer.encode_ordinary(text).len()
}

/// Count tokens in multiple lines of text
pub fn count_tokens_in_lines(lines: &[String]) -> usize {
    let text = lines.join("\n");
    count_tokens(&text)
}

/// Count tokens in each line separately and return the total
pub fn count_tokens_per_line(lines: &[String]) -> Vec<usize> {
    let tokenizer = get_tokenizer();
    
    lines.iter()
        .map(|line| tokenizer.encode_ordinary(line).len())
        .collect()
}

/// Format a token count in a human-readable way
pub fn format_token_count(count: usize) -> String {
    match count {
        0 => "0 tokens".to_string(),
        1 => "1 token".to_string(),
        _ => format!("{} tokens", count),
    }
}

/// Calculate token usage percentage relative to a maximum
pub fn token_usage_percentage(count: usize, max: usize) -> f64 {
    if max == 0 {
        return 0.0;
    }
    (count as f64 / max as f64) * 100.0
}

/// Get a color-coded token usage description
pub fn token_usage_description(count: usize, max: usize) -> (&'static str, &'static str) {
    let percentage = token_usage_percentage(count, max);
    
    match percentage {
        p if p >= 100.0 => ("OVER LIMIT", "red"),
        p if p >= 90.0 => ("VERY HIGH", "red"),
        p if p >= 75.0 => ("HIGH", "yellow"),
        p if p >= 50.0 => ("MEDIUM", "green"),
        p if p >= 25.0 => ("LOW", "blue"),
        _ => ("VERY LOW", "cyan"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_counting() {
        // Test with an empty string
        assert_eq!(count_tokens(""), 0);
        
        // Test with a simple string
        let simple = "Hello, world!";
        let simple_count = count_tokens(simple);
        assert!(simple_count > 0, "Should count at least one token");
        
        // Test with multiple lines
        let lines = vec![
            "This is line 1".to_string(),
            "This is line 2".to_string(),
            "This is line 3".to_string(),
        ];
        
        let total_count = count_tokens_in_lines(&lines);
        let per_line_counts = count_tokens_per_line(&lines);
        
        assert_eq!(per_line_counts.len(), 3, "Should return a count for each line");
        
        // The total count should be slightly less than the sum of individual lines
        // because joining with newlines is more efficient for tokenization
        let sum_of_lines: usize = per_line_counts.iter().sum();
        assert!(total_count <= sum_of_lines, "Total count should be at most the sum of individual lines");
    }
    
    #[test]
    fn test_formatting() {
        assert_eq!(format_token_count(0), "0 tokens");
        assert_eq!(format_token_count(1), "1 token");
        assert_eq!(format_token_count(2), "2 tokens");
    }
    
    #[test]
    fn test_usage_percentage() {
        assert_eq!(token_usage_percentage(0, 100), 0.0);
        assert_eq!(token_usage_percentage(50, 100), 50.0);
        assert_eq!(token_usage_percentage(100, 100), 100.0);
        assert_eq!(token_usage_percentage(150, 100), 150.0);
        
        // Handle division by zero
        assert_eq!(token_usage_percentage(100, 0), 0.0);
    }
    
    #[test]
    fn test_usage_description() {
        assert_eq!(token_usage_description(0, 100), ("VERY LOW", "cyan"));
        assert_eq!(token_usage_description(24, 100), ("VERY LOW", "cyan"));
        assert_eq!(token_usage_description(25, 100), ("LOW", "blue"));
        assert_eq!(token_usage_description(49, 100), ("LOW", "blue"));
        assert_eq!(token_usage_description(50, 100), ("MEDIUM", "green"));
        assert_eq!(token_usage_description(74, 100), ("MEDIUM", "green"));
        assert_eq!(token_usage_description(75, 100), ("HIGH", "yellow"));
        assert_eq!(token_usage_description(89, 100), ("HIGH", "yellow"));
        assert_eq!(token_usage_description(90, 100), ("VERY HIGH", "red"));
        assert_eq!(token_usage_description(99, 100), ("VERY HIGH", "red"));
        assert_eq!(token_usage_description(100, 100), ("OVER LIMIT", "red"));
        assert_eq!(token_usage_description(101, 100), ("OVER LIMIT", "red"));
    }
}