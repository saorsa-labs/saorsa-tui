//! TCSS parsing error types.

use std::fmt;

/// Errors from TCSS parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TcssError {
    /// General parse error.
    Parse(String),
    /// Unknown property name.
    UnknownProperty(String),
    /// Invalid value for a property.
    InvalidValue {
        /// The property name.
        property: String,
        /// The invalid value.
        value: String,
    },
    /// Selector parse error.
    SelectorError(String),
}

impl fmt::Display for TcssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
            Self::UnknownProperty(name) => write!(f, "unknown property: {name}"),
            Self::InvalidValue { property, value } => {
                write!(f, "invalid value '{value}' for property '{property}'")
            }
            Self::SelectorError(msg) => write!(f, "selector error: {msg}"),
        }
    }
}

impl std::error::Error for TcssError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_parse() {
        let err = TcssError::Parse("unexpected token".into());
        assert_eq!(err.to_string(), "parse error: unexpected token");
    }

    #[test]
    fn display_unknown_property() {
        let err = TcssError::UnknownProperty("foobaz".into());
        assert_eq!(err.to_string(), "unknown property: foobaz");
    }

    #[test]
    fn display_invalid_value() {
        let err = TcssError::InvalidValue {
            property: "color".into(),
            value: "notacolor".into(),
        };
        assert_eq!(
            err.to_string(),
            "invalid value 'notacolor' for property 'color'"
        );
    }

    #[test]
    fn display_selector_error() {
        let err = TcssError::SelectorError("expected ident".into());
        assert_eq!(err.to_string(), "selector error: expected ident");
    }

    #[test]
    fn error_trait() {
        let err = TcssError::Parse("test".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn clone_and_eq() {
        let err = TcssError::Parse("test".into());
        let err2 = err.clone();
        assert_eq!(err, err2);
    }
}
