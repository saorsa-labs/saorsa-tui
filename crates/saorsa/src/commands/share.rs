//! `/share` command — share conversation.

/// Show sharing status.
///
/// Sharing is not yet implemented.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("Conversation sharing is not yet available.\n\
        Session export coming in a future release."
        .to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn share_says_not_available() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("not yet available"));
    }
}
