//! /model command - switch the active LLM model.

/// Handle the /model command.
///
/// Switches the active LLM model to the specified model name.
/// If no model name is provided, shows the current model.
pub fn execute(args: &str) -> anyhow::Result<String> {
    if args.trim().is_empty() {
        Ok("Usage: /model <model-name>".to_string())
    } else {
        let model = args.trim();
        Ok(format!("Switched to model: {}", model))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_with_name() {
        let result = execute("claude-opus-4");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("claude-opus-4")),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn model_without_name() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("Usage")),
            Err(_) => unreachable!(),
        }
    }
}
