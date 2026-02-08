//! General agent settings.

use std::fmt;
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{Result, SaorsaAgentError};

/// Extended-thinking / chain-of-thought level.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingLevel {
    /// Thinking disabled.
    #[default]
    Off,
    /// Minimal thinking budget.
    Low,
    /// Moderate thinking budget.
    Medium,
    /// Maximum thinking budget.
    High,
}

impl fmt::Display for ThinkingLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Off => "off",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        };
        f.write_str(s)
    }
}

/// Error returned when parsing an invalid thinking level string.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid thinking level: '{0}' (expected off, low, medium, high)")]
pub struct ParseThinkingLevelError(String);

impl FromStr for ThinkingLevel {
    type Err = ParseThinkingLevelError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "off" | "none" | "0" => Ok(Self::Off),
            "low" | "1" => Ok(Self::Low),
            "medium" | "med" | "2" => Ok(Self::Medium),
            "high" | "3" => Ok(Self::High),
            other => Err(ParseThinkingLevelError(other.to_string())),
        }
    }
}

/// General agent settings that apply across all sessions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Default provider name (e.g. `"anthropic"`, `"openai"`).
    #[serde(default)]
    pub default_provider: Option<String>,
    /// Default model identifier.
    #[serde(default)]
    pub default_model: Option<String>,
    /// Extended-thinking level.
    #[serde(default)]
    pub thinking_level: ThinkingLevel,
    /// List of model identifiers the user has enabled for selection.
    #[serde(default)]
    pub enabled_models: Vec<String>,
    /// Maximum number of agent turns per run.
    #[serde(default)]
    pub max_turns: Option<u32>,
    /// Maximum tokens per LLM response.
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

/// Load settings from a JSON file.
///
/// Returns [`Settings::default()`] if the file does not exist.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::ConfigIo`] on I/O failures or
/// [`SaorsaAgentError::ConfigParse`] on JSON parse failures.
pub fn load(path: &Path) -> Result<Settings> {
    if !path.exists() {
        return Ok(Settings::default());
    }
    let data = std::fs::read_to_string(path).map_err(SaorsaAgentError::ConfigIo)?;
    let settings: Settings = serde_json::from_str(&data).map_err(SaorsaAgentError::ConfigParse)?;
    Ok(settings)
}

/// Save settings to a JSON file.
///
/// Creates parent directories if they do not exist.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::ConfigIo`] on I/O failures or
/// [`SaorsaAgentError::ConfigParse`] on serialization failures.
pub fn save(settings: &Settings, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(SaorsaAgentError::ConfigIo)?;
    }
    let data = serde_json::to_string_pretty(settings).map_err(SaorsaAgentError::ConfigParse)?;
    std::fs::write(path, data).map_err(SaorsaAgentError::ConfigIo)?;
    Ok(())
}

/// Merge an overlay settings into base settings.
///
/// Fields in `overlay` that are `Some` or non-default override the
/// corresponding fields in `base`.
pub fn merge(base: &Settings, overlay: &Settings) -> Settings {
    Settings {
        default_provider: overlay
            .default_provider
            .clone()
            .or_else(|| base.default_provider.clone()),
        default_model: overlay
            .default_model
            .clone()
            .or_else(|| base.default_model.clone()),
        thinking_level: if overlay.thinking_level != ThinkingLevel::Off {
            overlay.thinking_level.clone()
        } else {
            base.thinking_level.clone()
        },
        enabled_models: if overlay.enabled_models.is_empty() {
            base.enabled_models.clone()
        } else {
            overlay.enabled_models.clone()
        },
        max_turns: overlay.max_turns.or(base.max_turns),
        max_tokens: overlay.max_tokens.or(base.max_tokens),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_settings() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("settings.json");

        let settings = Settings {
            default_provider: Some("anthropic".into()),
            default_model: Some("claude-sonnet-4-5-20250929".into()),
            thinking_level: ThinkingLevel::High,
            enabled_models: vec!["claude-sonnet-4-5-20250929".into(), "gpt-4".into()],
            max_turns: Some(20),
            max_tokens: Some(8192),
        };

        save(&settings, &path).unwrap();
        let loaded = load(&path).unwrap();

        assert_eq!(loaded.default_provider.as_deref(), Some("anthropic"));
        assert_eq!(
            loaded.default_model.as_deref(),
            Some("claude-sonnet-4-5-20250929")
        );
        assert_eq!(loaded.thinking_level, ThinkingLevel::High);
        assert_eq!(loaded.enabled_models.len(), 2);
        assert_eq!(loaded.max_turns, Some(20));
        assert_eq!(loaded.max_tokens, Some(8192));
    }

    #[test]
    fn load_missing_file_returns_default() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nonexistent.json");
        let settings = load(&path).unwrap();
        assert!(settings.default_provider.is_none());
        assert!(settings.default_model.is_none());
        assert_eq!(settings.thinking_level, ThinkingLevel::Off);
        assert!(settings.enabled_models.is_empty());
        assert!(settings.max_turns.is_none());
        assert!(settings.max_tokens.is_none());
    }

    #[test]
    fn merge_overlay_wins() {
        let base = Settings {
            default_provider: Some("anthropic".into()),
            default_model: Some("old-model".into()),
            thinking_level: ThinkingLevel::Low,
            enabled_models: vec!["a".into()],
            max_turns: Some(10),
            max_tokens: Some(4096),
        };
        let overlay = Settings {
            default_provider: Some("openai".into()),
            default_model: None,
            thinking_level: ThinkingLevel::High,
            enabled_models: vec!["b".into(), "c".into()],
            max_turns: None,
            max_tokens: Some(8192),
        };

        let merged = merge(&base, &overlay);
        assert_eq!(merged.default_provider.as_deref(), Some("openai"));
        // overlay.default_model is None, so base wins.
        assert_eq!(merged.default_model.as_deref(), Some("old-model"));
        assert_eq!(merged.thinking_level, ThinkingLevel::High);
        assert_eq!(merged.enabled_models, vec!["b", "c"]);
        // overlay.max_turns is None, so base wins.
        assert_eq!(merged.max_turns, Some(10));
        assert_eq!(merged.max_tokens, Some(8192));
    }

    #[test]
    fn merge_base_preserved_when_overlay_empty() {
        let base = Settings {
            default_provider: Some("anthropic".into()),
            default_model: Some("model".into()),
            thinking_level: ThinkingLevel::Medium,
            enabled_models: vec!["x".into()],
            max_turns: Some(5),
            max_tokens: Some(2048),
        };
        let overlay = Settings::default();

        let merged = merge(&base, &overlay);
        assert_eq!(merged.default_provider.as_deref(), Some("anthropic"));
        assert_eq!(merged.default_model.as_deref(), Some("model"));
        // ThinkingLevel::Off in overlay means use base.
        assert_eq!(merged.thinking_level, ThinkingLevel::Medium);
        assert_eq!(merged.enabled_models, vec!["x"]);
        assert_eq!(merged.max_turns, Some(5));
        assert_eq!(merged.max_tokens, Some(2048));
    }

    #[test]
    fn save_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("a").join("b").join("settings.json");
        let settings = Settings::default();
        save(&settings, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn thinking_level_serde_roundtrip() {
        let json = serde_json::to_string(&ThinkingLevel::High).unwrap();
        assert_eq!(json, "\"high\"");
        let deserialized: ThinkingLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ThinkingLevel::High);
    }

    #[test]
    fn thinking_level_default_is_off() {
        assert_eq!(ThinkingLevel::default(), ThinkingLevel::Off);
    }

    #[test]
    fn thinking_level_display() {
        assert_eq!(ThinkingLevel::Off.to_string(), "off");
        assert_eq!(ThinkingLevel::Low.to_string(), "low");
        assert_eq!(ThinkingLevel::Medium.to_string(), "medium");
        assert_eq!(ThinkingLevel::High.to_string(), "high");
    }

    #[test]
    fn thinking_level_from_str() {
        assert_eq!("off".parse::<ThinkingLevel>().unwrap(), ThinkingLevel::Off);
        assert_eq!("low".parse::<ThinkingLevel>().unwrap(), ThinkingLevel::Low);
        assert_eq!(
            "medium".parse::<ThinkingLevel>().unwrap(),
            ThinkingLevel::Medium
        );
        assert_eq!(
            "high".parse::<ThinkingLevel>().unwrap(),
            ThinkingLevel::High
        );
        // Case insensitive.
        assert_eq!(
            "HIGH".parse::<ThinkingLevel>().unwrap(),
            ThinkingLevel::High
        );
        // Numeric aliases.
        assert_eq!("0".parse::<ThinkingLevel>().unwrap(), ThinkingLevel::Off);
        assert_eq!("3".parse::<ThinkingLevel>().unwrap(), ThinkingLevel::High);
    }

    #[test]
    fn thinking_level_from_str_invalid() {
        let err = "extreme".parse::<ThinkingLevel>().unwrap_err();
        assert!(err.to_string().contains("extreme"));
    }
}
