//! Error types for saorsa-agent.

use saorsa_ai::SaorsaAiError;

/// Error type for agent runtime operations.
#[derive(Debug, thiserror::Error)]
pub enum SaorsaAgentError {
    /// Tool execution failed.
    #[error("tool error: {0}")]
    Tool(String),

    /// Session management error.
    #[error("session error: {0}")]
    Session(String),

    /// Context engineering error.
    #[error("context error: {0}")]
    Context(String),

    /// LLM provider error.
    #[error("provider error: {0}")]
    Provider(#[from] SaorsaAiError),

    /// Agent run was cancelled.
    #[error("cancelled: {0}")]
    Cancelled(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Extension error.
    #[error("extension error: {0}")]
    Extension(String),

    /// Home directory could not be determined.
    #[error("could not determine home directory")]
    HomeDirectory,

    /// Configuration file I/O error.
    #[error("config I/O error: {0}")]
    ConfigIo(std::io::Error),

    /// Configuration file parse error.
    #[error("config parse error: {0}")]
    ConfigParse(serde_json::Error),

    /// Environment variable not found.
    #[error("environment variable not found: {name}")]
    EnvVarNotFound {
        /// The name of the missing environment variable.
        name: String,
    },

    /// Shell command execution failed.
    #[error("command execution failed: {0}")]
    CommandFailed(String),
}

/// Result type alias for saorsa-agent operations.
pub type Result<T> = std::result::Result<T, SaorsaAgentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_error_display() {
        let err = SaorsaAgentError::Tool("command failed".into());
        assert_eq!(err.to_string(), "tool error: command failed");
    }

    #[test]
    fn provider_error_converts() {
        let ai_err = SaorsaAiError::Auth("bad key".into());
        let err: SaorsaAgentError = ai_err.into();
        assert!(matches!(err, SaorsaAgentError::Provider(_)));
    }
}
