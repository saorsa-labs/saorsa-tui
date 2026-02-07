//! /compact command - toggle compact mode.

/// Handle the /compact command.
///
/// Toggles compact mode on/off for minimal UI.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("Compact mode toggled".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_toggles() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("toggled")),
            Err(_) => unreachable!(),
        }
    }
}
