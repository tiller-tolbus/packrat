use std::sync::OnceLock;
use tiktoken_rs::{cl100k_base, CoreBPE};

static TOKENIZER: OnceLock<CoreBPE> = OnceLock::new();

fn get_tokenizer() -> &'static CoreBPE {
    TOKENIZER.get_or_init(|| {
        cl100k_base().expect("Failed to initialize Claude tokenizer")
    })
}

pub fn count_tokens(text: &str) -> usize {
    let tokenizer = get_tokenizer();
    tokenizer.encode_ordinary(text).len()
}

pub fn count_tokens_in_lines(lines: &[String]) -> usize {
    let text = lines.join("\n");
    count_tokens(&text)
}

#[allow(dead_code)]
pub fn format_token_count(count: usize) -> String {
    match count {
        0 => "0 tokens".to_string(),
        1 => "1 token".to_string(),
        _ => format!("{} tokens", count),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_counting() {
        assert_eq!(count_tokens(""), 0);
        
        let simple = "Hello, world!";
        let simple_count = count_tokens(simple);
        assert!(simple_count > 0);
        
        let lines = vec![
            "This is line 1".to_string(),
            "This is line 2".to_string(),
            "This is line 3".to_string(),
        ];
        
        let total_count = count_tokens_in_lines(&lines);
        assert!(total_count > 0);
    }
    
    #[test]
    fn test_formatting() {
        assert_eq!(format_token_count(0), "0 tokens");
        assert_eq!(format_token_count(1), "1 token");
        assert_eq!(format_token_count(2), "2 tokens");
    }
}