//! Custom model and provider configuration.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Result, SaorsaAgentError};

/// Cost structure for a model (per million tokens).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModelCost {
    /// Cost per million input tokens.
    #[serde(default)]
    pub input: f64,
    /// Cost per million output tokens.
    #[serde(default)]
    pub output: f64,
    /// Cost per million cache-read tokens.
    #[serde(default)]
    pub cache_read: f64,
    /// Cost per million cache-write tokens.
    #[serde(default)]
    pub cache_write: f64,
}

/// A custom model definition within a provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomModel {
    /// The model identifier sent to the API.
    pub id: String,
    /// A human-readable display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Maximum context window size in tokens.
    #[serde(default)]
    pub context_window: Option<u64>,
    /// Maximum output tokens per request.
    #[serde(default)]
    pub max_tokens: Option<u64>,
    /// Whether the model supports extended thinking / chain-of-thought.
    #[serde(default)]
    pub reasoning: bool,
    /// Whether the model accepts image/file inputs.
    #[serde(default)]
    pub input: Option<String>,
    /// Token pricing information.
    #[serde(default)]
    pub cost: Option<ModelCost>,
}

/// A custom provider configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomProvider {
    /// The base URL for the provider API.
    pub base_url: String,
    /// The API type (e.g. `"openai"`, `"anthropic"`).
    #[serde(default)]
    pub api: Option<String>,
    /// The API key (if static; prefer auth config for dynamic keys).
    #[serde(default)]
    pub api_key: Option<String>,
    /// The authorization header name (defaults to `"Authorization"`).
    #[serde(default)]
    pub auth_header: Option<String>,
    /// Additional headers to send with every request.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Models available from this provider.
    #[serde(default)]
    pub models: Vec<CustomModel>,
}

/// Top-level models configuration mapping provider names to their configs.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModelsConfig {
    /// Provider name to configuration mapping.
    #[serde(flatten)]
    pub providers: HashMap<String, CustomProvider>,
}

/// Load models configuration from a JSON file.
///
/// Returns a default (empty) [`ModelsConfig`] if the file does not exist.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::ConfigIo`] on I/O failures or
/// [`SaorsaAgentError::ConfigParse`] on JSON parse failures.
pub fn load(path: &Path) -> Result<ModelsConfig> {
    if !path.exists() {
        return Ok(ModelsConfig::default());
    }
    let data = std::fs::read_to_string(path).map_err(SaorsaAgentError::ConfigIo)?;
    let config: ModelsConfig =
        serde_json::from_str(&data).map_err(SaorsaAgentError::ConfigParse)?;
    Ok(config)
}

/// Save models configuration to a JSON file.
///
/// Creates parent directories if they do not exist.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::ConfigIo`] on I/O failures or
/// [`SaorsaAgentError::ConfigParse`] on serialization failures.
pub fn save(config: &ModelsConfig, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(SaorsaAgentError::ConfigIo)?;
    }
    let data = serde_json::to_string_pretty(config).map_err(SaorsaAgentError::ConfigParse)?;
    std::fs::write(path, data).map_err(SaorsaAgentError::ConfigIo)?;
    Ok(())
}

/// Merge an overlay configuration into a base configuration.
///
/// Providers in `overlay` take precedence; within a provider, models from
/// the overlay are appended after the base models (no deduplication).
pub fn merge(base: &ModelsConfig, overlay: &ModelsConfig) -> ModelsConfig {
    let mut merged = base.clone();
    for (name, overlay_provider) in &overlay.providers {
        if let Some(existing) = merged.providers.get_mut(name) {
            // Overlay scalar fields if present.
            if overlay_provider.api.is_some() {
                existing.api.clone_from(&overlay_provider.api);
            }
            if overlay_provider.api_key.is_some() {
                existing.api_key.clone_from(&overlay_provider.api_key);
            }
            if overlay_provider.auth_header.is_some() {
                existing
                    .auth_header
                    .clone_from(&overlay_provider.auth_header);
            }
            existing.base_url.clone_from(&overlay_provider.base_url);
            for (k, v) in &overlay_provider.headers {
                existing.headers.insert(k.clone(), v.clone());
            }
            existing
                .models
                .extend(overlay_provider.models.iter().cloned());
        } else {
            merged
                .providers
                .insert(name.clone(), overlay_provider.clone());
        }
    }
    merged
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_provider() -> CustomProvider {
        CustomProvider {
            base_url: "https://api.example.com".into(),
            api: Some("openai".into()),
            api_key: None,
            auth_header: None,
            headers: HashMap::new(),
            models: vec![CustomModel {
                id: "model-1".into(),
                name: Some("Model One".into()),
                context_window: Some(128_000),
                max_tokens: Some(4096),
                reasoning: false,
                input: None,
                cost: Some(ModelCost {
                    input: 3.0,
                    output: 15.0,
                    cache_read: 0.0,
                    cache_write: 0.0,
                }),
            }],
        }
    }

    #[test]
    fn roundtrip_models_config() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("models.json");

        let mut config = ModelsConfig::default();
        config.providers.insert("custom".into(), sample_provider());

        save(&config, &path).unwrap();
        let loaded = load(&path).unwrap();

        assert_eq!(loaded.providers.len(), 1);
        let provider = loaded.providers.get("custom").unwrap();
        assert_eq!(provider.base_url, "https://api.example.com");
        assert_eq!(provider.models.len(), 1);
        assert_eq!(provider.models[0].id, "model-1");
    }

    #[test]
    fn load_missing_file_returns_default() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nonexistent.json");
        let config = load(&path).unwrap();
        assert!(config.providers.is_empty());
    }

    #[test]
    fn merge_adds_new_provider() {
        let base = ModelsConfig::default();
        let mut overlay = ModelsConfig::default();
        overlay.providers.insert("new".into(), sample_provider());

        let merged = merge(&base, &overlay);
        assert_eq!(merged.providers.len(), 1);
        assert!(merged.providers.contains_key("new"));
    }

    #[test]
    fn merge_appends_models() {
        let mut base = ModelsConfig::default();
        base.providers.insert("p".into(), sample_provider());

        let mut overlay = ModelsConfig::default();
        let mut overlay_provider = sample_provider();
        overlay_provider.models[0].id = "model-2".into();
        overlay.providers.insert("p".into(), overlay_provider);

        let merged = merge(&base, &overlay);
        let provider = merged.providers.get("p").unwrap();
        assert_eq!(provider.models.len(), 2);
        assert_eq!(provider.models[0].id, "model-1");
        assert_eq!(provider.models[1].id, "model-2");
    }

    #[test]
    fn merge_overlay_overrides_scalars() {
        let mut base = ModelsConfig::default();
        base.providers.insert("p".into(), sample_provider());

        let mut overlay = ModelsConfig::default();
        let mut overlay_provider = sample_provider();
        overlay_provider.base_url = "https://new.example.com".into();
        overlay_provider.api = Some("anthropic".into());
        overlay_provider.models.clear();
        overlay.providers.insert("p".into(), overlay_provider);

        let merged = merge(&base, &overlay);
        let provider = merged.providers.get("p").unwrap();
        assert_eq!(provider.base_url, "https://new.example.com");
        assert_eq!(provider.api.as_deref(), Some("anthropic"));
        // Models from base are preserved.
        assert_eq!(provider.models.len(), 1);
    }

    #[test]
    fn save_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("a").join("b").join("models.json");
        let config = ModelsConfig::default();
        save(&config, &path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn model_cost_defaults_to_zero() {
        let cost = ModelCost::default();
        assert!((cost.input - 0.0).abs() < f64::EPSILON);
        assert!((cost.output - 0.0).abs() < f64::EPSILON);
        assert!((cost.cache_read - 0.0).abs() < f64::EPSILON);
        assert!((cost.cache_write - 0.0).abs() < f64::EPSILON);
    }
}
