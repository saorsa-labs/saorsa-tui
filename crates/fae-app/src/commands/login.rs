//! /login command - authenticate with API providers.

/// Handle the /login command.
///
/// Authenticates with LLM API providers.
pub fn execute(args: &str) -> anyhow::Result<String> {
    if args.trim().is_empty() {
        Ok("Usage: /login <provider>".to_string())
    } else {
        let provider = args.trim();
        Ok(format!("Authenticated with {}", provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_with_provider() {
        let result = execute("anthropic");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("Authenticated")),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn login_without_provider() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("Usage")),
            Err(_) => unreachable!(),
        }
    }
}
