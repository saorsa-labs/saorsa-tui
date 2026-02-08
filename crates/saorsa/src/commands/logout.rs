//! `/logout` command — remove API keys.

/// Show instructions for removing API keys.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("\
To remove API keys, edit ~/.saorsa/auth.json and delete the provider entry.
Config location: ~/.saorsa/auth.json"
        .to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn logout_shows_config_path() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("auth.json"));
    }

    #[test]
    fn logout_mentions_delete() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("delete"));
    }
}
