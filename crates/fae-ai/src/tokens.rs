//! Token counting and context window management.

use crate::message::{ContentBlock, Message};

/// Approximate token count for a string.
///
/// Uses the rough heuristic of ~4 characters per token for English text.
/// This is a fast approximation; actual token counts depend on the
/// specific tokenizer used by each model.
pub fn estimate_tokens(text: &str) -> u32 {
    let chars = text.len() as u32;
    // Rough approximation: 1 token ≈ 4 characters
    chars.div_ceil(4)
}

/// Estimate the token count for a message.
pub fn estimate_message_tokens(message: &Message) -> u32 {
    // 3 tokens overhead per message (role, formatting)
    let overhead = 3u32;
    let content: u32 = message
        .content
        .iter()
        .map(|block| match block {
            ContentBlock::Text { text } => estimate_tokens(text),
            ContentBlock::ToolUse { name, input, .. } => {
                estimate_tokens(name) + estimate_tokens(&input.to_string())
            }
            ContentBlock::ToolResult { content, .. } => estimate_tokens(content),
        })
        .sum();
    overhead + content
}

/// Estimate the total token count for a conversation.
pub fn estimate_conversation_tokens(messages: &[Message], system: Option<&str>) -> u32 {
    let system_tokens = system.map_or(0, estimate_tokens);
    let message_tokens: u32 = messages.iter().map(estimate_message_tokens).sum();
    system_tokens + message_tokens
}

/// Context window sizes for known models.
///
/// Delegates to the model registry in [`crate::models`]. Returns `None`
/// for unknown models.
pub fn context_window(model: &str) -> Option<u32> {
    crate::models::get_context_window(model)
}

/// Check if a conversation fits within the model's context window.
pub fn fits_in_context(
    messages: &[Message],
    system: Option<&str>,
    model: &str,
    max_output_tokens: u32,
) -> bool {
    let window = match context_window(model) {
        Some(w) => w,
        None => return true, // Unknown model, assume it fits
    };
    let estimated = estimate_conversation_tokens(messages, system);
    estimated + max_output_tokens <= window
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Message;

    #[test]
    fn estimate_tokens_basic() {
        // "hello" = 5 chars → ~1-2 tokens
        let t = estimate_tokens("hello");
        assert!((1..=3).contains(&t));
    }

    #[test]
    fn estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn estimate_tokens_long_text() {
        let text = "a".repeat(1000);
        let t = estimate_tokens(&text);
        // Should be around 250 (1000/4)
        assert!((200..=300).contains(&t));
    }

    #[test]
    fn estimate_message_tokens_text() {
        let msg = Message::user("Hello, how are you?");
        let t = estimate_message_tokens(&msg);
        // 3 overhead + ~5 content tokens
        assert!(t >= 5);
    }

    #[test]
    fn estimate_conversation_basic() {
        let messages = vec![Message::user("Hello"), Message::assistant("Hi there!")];
        let t = estimate_conversation_tokens(&messages, Some("Be helpful"));
        assert!(t > 0);
    }

    #[test]
    fn context_window_known_models() {
        assert_eq!(context_window("claude-sonnet-4"), Some(200_000));
        assert_eq!(context_window("claude-opus-4"), Some(200_000));
        assert_eq!(context_window("gpt-4o"), Some(128_000));
        assert_eq!(context_window("unknown-model"), None);
    }

    #[test]
    fn fits_in_context_check() {
        let messages = vec![Message::user("Hello")];
        assert!(fits_in_context(
            &messages,
            None,
            "claude-sonnet-4",
            4096
        ));
    }

    #[test]
    fn fits_in_context_unknown_model() {
        let messages = vec![Message::user("Hello")];
        assert!(fits_in_context(&messages, None, "unknown-model", 4096));
    }
}
