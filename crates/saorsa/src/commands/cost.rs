//! `/cost` command — show session cost breakdown.

use saorsa_agent::CostTracker;

/// Display cost breakdown for the current session.
pub fn execute(_args: &str, tracker: &CostTracker) -> anyhow::Result<String> {
    if tracker.entries.is_empty() {
        return Ok("No interactions yet — session cost: $0.00".into());
    }

    let mut text = format!("Session cost: {}\n", tracker.format_session_cost());
    text.push_str(&format!("Interactions: {}\n", tracker.entries.len()));

    let total_input: u32 = tracker.entries.iter().map(|e| e.input_tokens).sum();
    let total_output: u32 = tracker.entries.iter().map(|e| e.output_tokens).sum();
    text.push_str(&format!(
        "Total tokens: {} in / {} out",
        total_input, total_output
    ));

    // Show last few entries.
    let recent = tracker.entries.len().min(5);
    if recent > 0 {
        text.push_str("\n\nRecent:");
        for entry in tracker.entries.iter().rev().take(recent) {
            let cost_str = if entry.cost_usd < 0.01 {
                format!("${:.4}", entry.cost_usd)
            } else {
                format!("${:.2}", entry.cost_usd)
            };
            text.push_str(&format!(
                "\n  {} — {} in / {} out — {}",
                entry.model, entry.input_tokens, entry.output_tokens, cost_str,
            ));
        }
    }

    Ok(text)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn empty_tracker_shows_zero() {
        let tracker = CostTracker::new();
        let text = execute("", &tracker).expect("should succeed");
        assert!(text.contains("$0.00"));
        assert!(text.contains("No interactions"));
    }

    #[test]
    fn populated_tracker_shows_breakdown() {
        let mut tracker = CostTracker::new();
        let usage = saorsa_ai::Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };
        tracker.track("claude-sonnet-4", &usage);

        let text = execute("", &tracker).expect("should succeed");
        assert!(text.contains("Session cost:"));
        assert!(text.contains("Interactions: 1"));
        assert!(text.contains("1000 in / 500 out"));
        assert!(text.contains("Recent:"));
        assert!(text.contains("claude-sonnet-4"));
    }
}
