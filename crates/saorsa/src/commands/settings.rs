//! /settings command - open settings screen.

/// Handle the /settings command.
///
/// Opens the settings screen.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("Opening settings...".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_opens() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => assert!(output.contains("settings")),
            Err(_) => unreachable!(),
        }
    }
}
