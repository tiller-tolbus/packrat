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

