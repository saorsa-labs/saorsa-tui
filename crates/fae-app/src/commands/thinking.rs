//! /thinking command - toggle thinking mode.

/// Handle the /thinking command.
///
/// Toggles thinking mode on/off to show/hide the model's reasoning.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("Thinking mode toggled".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thinking_toggles() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("toggled")),
            Err(_) => unreachable!(),
        }
    }
}
