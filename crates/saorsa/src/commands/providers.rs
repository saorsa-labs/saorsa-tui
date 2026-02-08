//! `/providers` command — list configured providers with auth status.

use saorsa_ai::ProviderKind;

/// All provider kinds to list.
const ALL_PROVIDERS: &[ProviderKind] = &[
    ProviderKind::Anthropic,
    ProviderKind::OpenAi,
    ProviderKind::Gemini,
    ProviderKind::Ollama,
    ProviderKind::OpenAiCompatible,
    ProviderKind::LmStudio,
    ProviderKind::Vllm,
    ProviderKind::OpenRouter,
];

/// Show available providers and whether an API key is detected.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    let mut text = "Providers:".to_string();

    for &kind in ALL_PROVIDERS {
        let env_var = kind.env_var_name();
        let has_key = std::env::var(env_var).is_ok();
        let status = if has_key { "configured" } else { "not set" };
        text.push_str(&format!(
            "\n  {:<20} {} ({})",
            kind.display_name(),
            status,
            env_var,
        ));
    }

    text.push_str("\n\nUse /login for instructions on configuring API keys.");
    Ok(text)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn lists_all_providers() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("Anthropic"));
        assert!(text.contains("OpenAI"));
        assert!(text.contains("Gemini"));
        assert!(text.contains("Ollama"));
        assert!(text.contains("OpenRouter"));
    }

    #[test]
    fn shows_env_var_names() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("ANTHROPIC_API_KEY"));
        assert!(text.contains("OPENAI_API_KEY"));
    }

    #[test]
    fn shows_login_hint() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("/login"));
    }
}
