//! Operating modes for different use cases.

/// Operating mode for the application.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum OperatingMode {
    /// Full interactive TUI (default).
    #[default]
    Interactive,
    /// Plain text streaming to stdout.
    Print,
    /// JSON Lines output.
    Json,
    /// JSON-RPC 2.0 over stdio.
    Rpc,
}

impl std::fmt::Display for OperatingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interactive => write!(f, "interactive"),
            Self::Print => write!(f, "print"),
            Self::Json => write!(f, "json"),
            Self::Rpc => write!(f, "rpc"),
        }
    }
}

impl std::str::FromStr for OperatingMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "interactive" => Ok(Self::Interactive),
            "print" => Ok(Self::Print),
            "json" => Ok(Self::Json),
            "rpc" => Ok(Self::Rpc),
            _ => Err(format!("Invalid operating mode: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn default_mode() {
        assert_eq!(OperatingMode::default(), OperatingMode::Interactive);
    }

    #[test]
    fn display_modes() {
        assert_eq!(OperatingMode::Interactive.to_string(), "interactive");
        assert_eq!(OperatingMode::Print.to_string(), "print");
        assert_eq!(OperatingMode::Json.to_string(), "json");
        assert_eq!(OperatingMode::Rpc.to_string(), "rpc");
    }

    #[test]
    fn parse_modes() {
        assert_eq!(
            OperatingMode::from_str("interactive"),
            Ok(OperatingMode::Interactive)
        );
        assert_eq!(OperatingMode::from_str("print"), Ok(OperatingMode::Print));
        assert_eq!(OperatingMode::from_str("json"), Ok(OperatingMode::Json));
        assert_eq!(OperatingMode::from_str("rpc"), Ok(OperatingMode::Rpc));
    }

    #[test]
    fn parse_invalid() {
        assert!(OperatingMode::from_str("invalid").is_err());
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(
            OperatingMode::from_str("INTERACTIVE"),
            Ok(OperatingMode::Interactive)
        );
    }
}
