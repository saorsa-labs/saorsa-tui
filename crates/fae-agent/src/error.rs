//! Error types for fae-agent.

use fae_ai::FaeAiError;

/// Error type for agent runtime operations.
#[derive(Debug, thiserror::Error)]
pub enum FaeAgentError {
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
    Provider(#[from] FaeAiError),

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
}

/// Result type alias for fae-agent operations.
pub type Result<T> = std::result::Result<T, FaeAgentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_error_display() {
        let err = FaeAgentError::Tool("command failed".into());
        assert_eq!(err.to_string(), "tool error: command failed");
    }

    #[test]
    fn provider_error_converts() {
        let ai_err = FaeAiError::Auth("bad key".into());
        let err: FaeAgentError = ai_err.into();
        assert!(matches!(err, FaeAgentError::Provider(_)));
    }
}
