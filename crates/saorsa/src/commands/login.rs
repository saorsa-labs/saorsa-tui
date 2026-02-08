//! `/login` command — configure API keys.

/// Show instructions for configuring API keys.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("\
To configure API keys, edit ~/.saorsa/auth.json:

  {
    \"anthropic\": \"sk-ant-...\",
    \"openai\": \"sk-...\",
    \"gemini\": \"AI...\"
  }

Supported providers: anthropic, openai, gemini, ollama, openrouter, lmstudio, vllm"
        .to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn login_shows_config_path() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("auth.json"));
    }

    #[test]
    fn login_lists_providers() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("anthropic"));
        assert!(text.contains("openai"));
    }
}
