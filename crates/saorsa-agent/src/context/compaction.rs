//! Context compaction strategies for managing conversation token limits.

use saorsa_ai::message::Message;
use saorsa_ai::tokens::{estimate_conversation_tokens, estimate_message_tokens};

/// Strategy for compacting conversation history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// Remove oldest messages first, preserving recent and system messages.
    TruncateOldest,
    /// Summarize blocks of messages (not yet implemented).
    SummarizeBlocks,
    /// Hybrid approach: truncate old, summarize middle blocks.
    Hybrid,
}

/// Configuration for context compaction.
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// Maximum tokens to keep after compaction.
    pub max_tokens: u32,
    /// Number of most recent messages to always preserve.
    pub preserve_recent_count: usize,
    /// Compaction strategy to use.
    pub strategy: CompactionStrategy,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,
            preserve_recent_count: 5,
            strategy: CompactionStrategy::TruncateOldest,
        }
    }
}

/// Statistics from a compaction operation.
#[derive(Debug, Clone)]
pub struct CompactionStats {
    /// Original token count before compaction.
    pub original_tokens: u32,
    /// Token count after compaction.
    pub compacted_tokens: u32,
    /// Number of messages removed.
    pub messages_removed: usize,
}

/// Compact a conversation history according to the given configuration.
///
/// System messages and the most recent N messages are always preserved.
/// Returns the compacted message list and statistics.
pub fn compact(
    messages: &[Message],
    system: Option<&str>,
    config: &CompactionConfig,
) -> (Vec<Message>, CompactionStats) {
    let original_tokens = estimate_conversation_tokens(messages, system);

    // If we're already under the limit, no compaction needed
    if original_tokens <= config.max_tokens {
        return (
            messages.to_vec(),
            CompactionStats {
                original_tokens,
                compacted_tokens: original_tokens,
                messages_removed: 0,
            },
        );
    }

    match config.strategy {
        CompactionStrategy::TruncateOldest => {
            truncate_oldest(messages, system, config, original_tokens)
        }
        CompactionStrategy::SummarizeBlocks | CompactionStrategy::Hybrid => {
            // For now, fall back to truncate (summarization not implemented)
            truncate_oldest(messages, system, config, original_tokens)
        }
    }
}

/// Truncate oldest messages first, preserving recent messages.
///
/// Note: saorsa-ai Message doesn't have a "system" role - system prompts are separate.
/// This function preserves recent messages and fits as many older messages as possible.
fn truncate_oldest(
    messages: &[Message],
    system: Option<&str>,
    config: &CompactionConfig,
    original_tokens: u32,
) -> (Vec<Message>, CompactionStats) {
    let system_tokens = system.map_or(0, saorsa_ai::tokens::estimate_tokens);

    // All messages are either User or Assistant (no system role in Message)
    let non_system = messages;

    let recent_start = non_system
        .len()
        .saturating_sub(config.preserve_recent_count);
    let old_messages = &non_system[..recent_start];
    let recent_messages = &non_system[recent_start..];

    // Calculate tokens for recent messages
    let recent_tokens: u32 = recent_messages.iter().map(estimate_message_tokens).sum();

    // Available tokens for old messages
    let available_for_old = config
        .max_tokens
        .saturating_sub(system_tokens)
        .saturating_sub(recent_tokens);

    // Keep as many old messages as fit
    let mut kept_old = Vec::new();
    let mut current_tokens = 0u32;

    for msg in old_messages.iter().rev() {
        let msg_tokens = estimate_message_tokens(msg);
        if current_tokens + msg_tokens <= available_for_old {
            kept_old.push((*msg).clone());
            current_tokens += msg_tokens;
        } else {
            break;
        }
    }
    kept_old.reverse();

    // Reconstruct message list: kept_old + recent
    let mut result = Vec::new();
    result.extend(kept_old);
    result.extend(recent_messages.iter().map(|m| (*m).clone()));

    let compacted_tokens = estimate_conversation_tokens(&result, system);
    let messages_removed = messages.len() - result.len();

    (
        result,
        CompactionStats {
            original_tokens,
            compacted_tokens,
            messages_removed,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_ai::message::{Message, Role};

    fn make_message(role: &str, text: &str) -> Message {
        match role {
            "user" => Message::user(text),
            "assistant" => Message::assistant(text),
            _ => unreachable!("Invalid role"),
        }
    }

    #[test]
    fn test_no_compaction_when_under_limit() {
        let messages = vec![
            make_message("user", "Hello"),
            make_message("assistant", "Hi"),
        ];
        let config = CompactionConfig {
            max_tokens: 100_000,
            ..Default::default()
        };

        let (compacted, stats) = compact(&messages, None, &config);

        assert_eq!(compacted.len(), messages.len());
        assert_eq!(stats.messages_removed, 0);
        assert_eq!(stats.original_tokens, stats.compacted_tokens);
    }

    #[test]
    fn test_truncate_oldest_removes_old_messages() {
        let large_text = "x".repeat(1000);
        let messages = vec![
            make_message("user", &large_text),
            make_message("assistant", &large_text),
            make_message("user", &large_text),
            make_message("assistant", &large_text),
            make_message("user", "Recent message"),
            make_message("assistant", "Recent response"),
        ];
        let config = CompactionConfig {
            max_tokens: 100, // Low limit to force removal of large old messages
            preserve_recent_count: 2,
            strategy: CompactionStrategy::TruncateOldest,
        };

        let (compacted, stats) = compact(&messages, None, &config);

        // Should preserve at least the recent 2
        assert!(compacted.len() >= 2);
        assert!(stats.messages_removed > 0);
        assert!(stats.compacted_tokens <= config.max_tokens);
    }

    #[test]
    fn test_recent_messages_always_preserved() {
        let large_text = "a".repeat(1000);
        let messages = vec![
            make_message("user", &large_text), // Large old message
            make_message("assistant", "Old response"),
            make_message("user", "Recent 1"),
            make_message("assistant", "Recent 2"),
        ];
        let config = CompactionConfig {
            max_tokens: 100,
            preserve_recent_count: 2,
            strategy: CompactionStrategy::TruncateOldest,
        };

        let (compacted, _stats) = compact(&messages, None, &config);

        // Last 2 messages should always be there
        assert!(compacted.len() >= 2);
        let last_two = &compacted[compacted.len() - 2..];
        assert_eq!(last_two[0].role, Role::User);
        assert_eq!(last_two[1].role, Role::Assistant);
    }

    #[test]
    fn test_compaction_with_system_prompt() {
        let large_text = "a".repeat(1000);
        let messages = vec![
            make_message("user", &large_text),
            make_message("assistant", "Response"),
        ];
        let system = Some("System prompt here");
        let config = CompactionConfig {
            max_tokens: 100,
            preserve_recent_count: 1,
            strategy: CompactionStrategy::TruncateOldest,
        };

        let (_compacted, stats) = compact(&messages, system, &config);

        // Should compact while accounting for system prompt tokens
        assert!(stats.compacted_tokens <= config.max_tokens);
    }

    #[test]
    fn test_compaction_achieves_target() {
        let a_text = "a".repeat(1000);
        let b_text = "b".repeat(1000);
        let c_text = "c".repeat(1000);
        let d_text = "d".repeat(1000);

        let messages = vec![
            make_message("user", &a_text),
            make_message("assistant", &b_text),
            make_message("user", &c_text),
            make_message("assistant", &d_text),
            make_message("user", "Recent"),
        ];
        let config = CompactionConfig {
            max_tokens: 100,
            preserve_recent_count: 1,
            strategy: CompactionStrategy::TruncateOldest,
        };

        let (compacted, stats) = compact(&messages, None, &config);

        // Should be significantly reduced
        assert!(stats.compacted_tokens <= config.max_tokens);
        assert!(stats.messages_removed > 0);
        assert!(compacted.len() < messages.len());
    }

    #[test]
    fn test_statistics_tracked_correctly() {
        let messages = vec![
            make_message("user", "Message 1"),
            make_message("assistant", "Response 1"),
            make_message("user", "Message 2"),
        ];
        let config = CompactionConfig {
            max_tokens: 20,
            preserve_recent_count: 1,
            strategy: CompactionStrategy::TruncateOldest,
        };

        let (compacted, stats) = compact(&messages, None, &config);

        assert_eq!(stats.messages_removed, messages.len() - compacted.len());
        assert!(stats.original_tokens > 0);
        assert!(stats.compacted_tokens > 0);
        assert!(stats.compacted_tokens <= stats.original_tokens);
    }

    #[test]
    fn test_default_config() {
        let config = CompactionConfig::default();
        assert_eq!(config.max_tokens, 100_000);
        assert_eq!(config.preserve_recent_count, 5);
        assert_eq!(config.strategy, CompactionStrategy::TruncateOldest);
    }
}
