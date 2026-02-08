//! Authentication configuration for LLM providers.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Result, SaorsaAgentError};

/// A single authentication entry describing how to obtain an API key.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthEntry {
    /// A raw API key stored directly in the config.
    ApiKey {
        /// The API key value.
        key: String,
    },
    /// An environment variable containing the API key.
    EnvVar {
        /// The environment variable name.
        name: String,
    },
    /// A shell command whose stdout is the API key.
    Command {
        /// The shell command to execute.
        command: String,
    },
}

/// Authentication configuration mapping provider names to auth entries.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Provider name to authentication entry mapping.
    #[serde(flatten)]
    pub providers: HashMap<String, AuthEntry>,
}

/// Load authentication configuration from a JSON file.
///
/// Returns a default (empty) [`AuthConfig`] if the file does not exist.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::ConfigIo`] on I/O failures or
/// [`SaorsaAgentError::ConfigParse`] on JSON parse failures.
pub fn load(path: &Path) -> Result<AuthConfig> {
    if !path.exists() {
        return Ok(AuthConfig::default());
    }
    let data = std::fs::read_to_string(path).map_err(SaorsaAgentError::ConfigIo)?;
    let config: AuthConfig = serde_json::from_str(&data).map_err(SaorsaAgentError::ConfigParse)?;
    Ok(config)
}

/// Save authentication configuration to a JSON file.
///
/// Creates parent directories if they do not exist.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::ConfigIo`] on I/O failures or
/// [`SaorsaAgentError::ConfigParse`] on serialization failures.
pub fn save(config: &AuthConfig, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(SaorsaAgentError::ConfigIo)?;
    }
    let data = serde_json::to_string_pretty(config).map_err(SaorsaAgentError::ConfigParse)?;
    std::fs::write(path, data).map_err(SaorsaAgentError::ConfigIo)?;
    Ok(())
}

/// Resolve an [`AuthEntry`] to a concrete API key string.
///
/// - `ApiKey` returns the key directly.
/// - `EnvVar` reads the named environment variable.
/// - `Command` executes the shell command and returns trimmed stdout.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::EnvVarNotFound`] when an environment variable
/// is missing, or [`SaorsaAgentError::CommandFailed`] when a shell command
/// exits with a non-zero status or fails to execute.
pub fn resolve(entry: &AuthEntry) -> Result<String> {
    match entry {
        AuthEntry::ApiKey { key } => Ok(key.clone()),
        AuthEntry::EnvVar { name } => {
            std::env::var(name).map_err(|_| SaorsaAgentError::EnvVarNotFound { name: name.clone() })
        }
        AuthEntry::Command { command } => {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .map_err(|e| SaorsaAgentError::CommandFailed(e.to_string()))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(SaorsaAgentError::CommandFailed(format!(
                    "command exited with {}: {}",
                    output.status,
                    stderr.trim()
                )));
            }
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
    }
}

/// Look up and resolve the API key for a named provider.
///
/// # Errors
///
/// Returns [`SaorsaAgentError::EnvVarNotFound`] if the provider has no entry
/// in the config (with the provider name as `name`), or any resolution error
/// from [`resolve`].
pub fn get_key(config: &AuthConfig, provider: &str) -> Result<String> {
    let entry = config
        .providers
        .get(provider)
        .ok_or_else(|| SaorsaAgentError::EnvVarNotFound {
            name: provider.to_string(),
        })?;
    resolve(entry)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_auth_config() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("auth.json");

        let mut config = AuthConfig::default();
        config.providers.insert(
            "anthropic".into(),
            AuthEntry::ApiKey {
                key: "sk-test-123".into(),
            },
        );
        config.providers.insert(
            "openai".into(),
            AuthEntry::EnvVar {
                name: "OPENAI_API_KEY".into(),
            },
        );

        save(&config, &path).unwrap();
        let loaded = load(&path).unwrap();

        assert_eq!(loaded.providers.len(), 2);
        assert!(loaded.providers.contains_key("anthropic"));
        assert!(loaded.providers.contains_key("openai"));
    }

    #[test]
    fn load_missing_file_returns_default() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nonexistent.json");
        let config = load(&path).unwrap();
        assert!(config.providers.is_empty());
    }

    #[test]
    fn resolve_api_key() {
        let entry = AuthEntry::ApiKey {
            key: "sk-direct".into(),
        };
        let resolved = resolve(&entry).unwrap();
        assert_eq!(resolved, "sk-direct");
    }

    #[test]
    fn resolve_env_var() {
        // SAFETY: This test is single-threaded and the variable name is unique
        // to this test, so no other thread observes it.
        unsafe {
            std::env::set_var("SAORSA_TEST_AUTH_KEY", "sk-from-env");
        }
        let entry = AuthEntry::EnvVar {
            name: "SAORSA_TEST_AUTH_KEY".into(),
        };
        let resolved = resolve(&entry).unwrap();
        assert_eq!(resolved, "sk-from-env");
        // SAFETY: Same reasoning as above.
        unsafe {
            std::env::remove_var("SAORSA_TEST_AUTH_KEY");
        }
    }

    #[test]
    fn resolve_env_var_missing() {
        let entry = AuthEntry::EnvVar {
            name: "SAORSA_NONEXISTENT_VAR_12345".into(),
        };
        let err = resolve(&entry).unwrap_err();
        assert!(matches!(err, SaorsaAgentError::EnvVarNotFound { .. }));
    }

    #[test]
    fn resolve_command() {
        let entry = AuthEntry::Command {
            command: "echo sk-from-cmd".into(),
        };
        let resolved = resolve(&entry).unwrap();
        assert_eq!(resolved, "sk-from-cmd");
    }

    #[test]
    fn resolve_command_failure() {
        let entry = AuthEntry::Command {
            command: "exit 1".into(),
        };
        let err = resolve(&entry).unwrap_err();
        assert!(matches!(err, SaorsaAgentError::CommandFailed(_)));
    }

    #[test]
    fn get_key_found() {
        let mut config = AuthConfig::default();
        config.providers.insert(
            "test".into(),
            AuthEntry::ApiKey {
                key: "sk-test".into(),
            },
        );
        let key = get_key(&config, "test").unwrap();
        assert_eq!(key, "sk-test");
    }

    #[test]
    fn get_key_missing_provider() {
        let config = AuthConfig::default();
        let err = get_key(&config, "missing").unwrap_err();
        assert!(matches!(err, SaorsaAgentError::EnvVarNotFound { .. }));
    }

    #[test]
    fn save_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nested").join("deep").join("auth.json");
        let config = AuthConfig::default();
        save(&config, &path).unwrap();
        assert!(path.exists());
    }
}
