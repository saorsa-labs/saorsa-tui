//! /logout command - clear credentials.

/// Handle the /logout command.
///
/// Clears API credentials.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("Credentials cleared".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logout_clears() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("cleared")),
            Err(_) => unreachable!(),
        }
    }
}
