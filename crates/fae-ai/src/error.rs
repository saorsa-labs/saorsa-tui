//! Error types for fae-ai.

/// Error type for LLM provider operations.
#[derive(Debug, thiserror::Error)]
pub enum FaeAiError {
    /// LLM provider error.
    #[error("provider error ({provider}): {message}")]
    Provider {
        /// Provider name.
        provider: String,
        /// Error detail.
        message: String,
    },

    /// Authentication failed.
    #[error("authentication error: {0}")]
    Auth(String),

    /// Network/HTTP error.
    #[error("network error: {0}")]
    Network(String),

    /// Rate limit exceeded.
    #[error("rate limit exceeded: {0}")]
    RateLimit(String),

    /// Invalid request.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Streaming error.
    #[error("streaming error: {0}")]
    Streaming(String),

    /// Token limit exceeded.
    #[error("token limit exceeded: {0}")]
    TokenLimit(String),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result type alias for fae-ai operations.
pub type Result<T> = std::result::Result<T, FaeAiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_error_display() {
        let err = FaeAiError::Provider {
            provider: "anthropic".into(),
            message: "invalid key".into(),
        };
        assert_eq!(err.to_string(), "provider error (anthropic): invalid key");
    }

    #[test]
    fn io_error_converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let err: FaeAiError = io_err.into();
        assert!(matches!(err, FaeAiError::Io(_)));
    }
}
