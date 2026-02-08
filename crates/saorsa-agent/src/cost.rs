//! Cost tracking for LLM interactions.
//!
//! Tracks per-interaction and cumulative session costs based on token usage
//! and model pricing from the [`saorsa_ai`] model registry.

use saorsa_ai::{Usage, lookup_model, lookup_model_by_prefix};

/// A single cost entry for one LLM interaction.
#[derive(Clone, Debug)]
pub struct CostEntry {
    /// Model that was used.
    pub model: String,
    /// Number of input tokens.
    pub input_tokens: u32,
    /// Number of output tokens.
    pub output_tokens: u32,
    /// Estimated cost in USD.
    pub cost_usd: f64,
}

/// Tracks cumulative costs across a session.
#[derive(Clone, Debug, Default)]
pub struct CostTracker {
    /// Individual cost entries.
    pub entries: Vec<CostEntry>,
    /// Running total cost in USD.
    pub session_total: f64,
}

impl CostTracker {
    /// Create a new empty cost tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Track a new interaction and return the cost entry.
    ///
    /// Looks up the model in the known models registry to find pricing.
    /// If the model is unknown or has no pricing information, cost is `0.0`.
    pub fn track(&mut self, model: &str, usage: &Usage) -> CostEntry {
        let model_info = lookup_model(model).or_else(|| lookup_model_by_prefix(model));

        let cost_usd = model_info
            .and_then(|info| {
                let input_cost = info.cost_per_million_input?;
                let output_cost = info.cost_per_million_output?;
                let input_usd = f64::from(usage.input_tokens) * input_cost / 1_000_000.0;
                let output_usd = f64::from(usage.output_tokens) * output_cost / 1_000_000.0;
                Some(input_usd + output_usd)
            })
            .unwrap_or(0.0);

        let entry = CostEntry {
            model: model.to_string(),
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cost_usd,
        };

        self.session_total += cost_usd;
        self.entries.push(entry.clone());

        entry
    }

    /// Format the session cost as a display string.
    ///
    /// Uses 4 decimal places for small amounts (< $0.01) and 2 decimal
    /// places otherwise.
    pub fn format_session_cost(&self) -> String {
        if self.session_total < 0.01 {
            format!("${:.4}", self.session_total)
        } else {
            format!("${:.2}", self.session_total)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn new_tracker_empty() {
        let tracker = CostTracker::new();
        assert!(tracker.entries.is_empty());
        assert!((tracker.session_total - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn track_known_model() {
        let mut tracker = CostTracker::new();
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };

        let entry = tracker.track("claude-sonnet-4", &usage);
        assert!(entry.cost_usd > 0.0);
        assert_eq!(entry.input_tokens, 1000);
        assert_eq!(entry.output_tokens, 500);
        assert_eq!(entry.model, "claude-sonnet-4");

        // claude-sonnet-4: input $3/M, output $15/M
        // 1000 input = $0.003, 500 output = $0.0075
        let expected = 1000.0 * 3.0 / 1_000_000.0 + 500.0 * 15.0 / 1_000_000.0;
        assert!((entry.cost_usd - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn track_unknown_model() {
        let mut tracker = CostTracker::new();
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };

        let entry = tracker.track("totally-unknown-model", &usage);
        assert!((entry.cost_usd - 0.0).abs() < f64::EPSILON);
        assert_eq!(tracker.entries.len(), 1);
    }

    #[test]
    fn format_cost_small() {
        let mut tracker = CostTracker::new();
        tracker.session_total = 0.0035;
        assert_eq!(tracker.format_session_cost(), "$0.0035");
    }

    #[test]
    fn format_cost_large() {
        let mut tracker = CostTracker::new();
        tracker.session_total = 1.2345;
        assert_eq!(tracker.format_session_cost(), "$1.23");
    }

    #[test]
    fn session_total_accumulates() {
        let mut tracker = CostTracker::new();
        let usage = Usage {
            input_tokens: 1_000_000,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };

        // claude-sonnet-4: input $3/M
        tracker.track("claude-sonnet-4", &usage);
        let first_total = tracker.session_total;
        assert!((first_total - 3.0).abs() < f64::EPSILON);

        // Track again
        tracker.track("claude-sonnet-4", &usage);
        assert!((tracker.session_total - 6.0).abs() < f64::EPSILON);
        assert_eq!(tracker.entries.len(), 2);
    }

    #[test]
    fn track_prefix_matched_model() {
        let mut tracker = CostTracker::new();
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };

        // Versioned model name should match via prefix
        let entry = tracker.track("claude-sonnet-4-5-20250929", &usage);
        assert!(entry.cost_usd > 0.0);
    }

    #[test]
    fn track_model_without_pricing() {
        let mut tracker = CostTracker::new();
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        };

        // Ollama models have no pricing
        let entry = tracker.track("llama3", &usage);
        assert!((entry.cost_usd - 0.0).abs() < f64::EPSILON);
    }
}
