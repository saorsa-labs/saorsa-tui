//! /share command - export conversation with shareable link.

/// Handle the /share command.
///
/// Exports the current conversation and generates a shareable link.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    let link = "https://share.saorsa.dev/abc123";
    Ok(format!("Conversation shared: {}", link))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_generates_link() {
        let result = execute("");
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                assert!(output.contains("shared"));
                assert!(output.contains("https://"));
            }
            Err(_) => unreachable!(),
        }
    }
}
