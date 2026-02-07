//! Error types for fae-core.

use std::io;

/// Error type for fae-core operations.
#[derive(Debug, thiserror::Error)]
pub enum FaeCoreError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Terminal operation failed.
    #[error("terminal error: {0}")]
    Terminal(String),

    /// Layout calculation failed.
    #[error("layout error: {0}")]
    Layout(String),

    /// CSS parsing or styling error.
    #[error("style error: {0}")]
    Style(String),

    /// Rendering failed.
    #[error("render error: {0}")]
    Render(String),

    /// Widget error.
    #[error("widget error: {0}")]
    Widget(String),

    /// Unicode handling error.
    #[error("unicode error: {0}")]
    Unicode(String),

    /// Reactive system error.
    #[error("reactive error: {0}")]
    Reactive(String),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Result type alias for fae-core operations.
pub type Result<T> = std::result::Result<T, FaeCoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = FaeCoreError::Terminal("no tty".into());
        assert_eq!(err.to_string(), "terminal error: no tty");
    }

    #[test]
    fn io_error_converts() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "missing");
        let err: FaeCoreError = io_err.into();
        assert!(matches!(err, FaeCoreError::Io(_)));
    }
}
