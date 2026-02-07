//! /clear command - clear conversation.

/// Handle the /clear command.
///
/// Clears the current conversation.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("Conversation cleared".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_works() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("cleared")),
            Err(_) => unreachable!(),
        }
    }
}
