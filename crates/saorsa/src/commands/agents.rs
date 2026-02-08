//! `/agents` command — list available agent tools.

/// Display the list of built-in agent tools.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    Ok("\
Agent tools:
  bash         Execute shell commands
  read         Read file contents (optional line range)
  write        Write content to files
  edit         Surgical text replacement in files
  grep         Search file contents with regex
  find         Find files by glob pattern
  ls           List directory contents
  web_search   Search the web (DuckDuckGo)

All tools operate relative to the current working directory."
        .to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn lists_all_tools() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("bash"));
        assert!(text.contains("read"));
        assert!(text.contains("write"));
        assert!(text.contains("edit"));
        assert!(text.contains("grep"));
        assert!(text.contains("find"));
        assert!(text.contains("ls"));
        assert!(text.contains("web_search"));
    }

    #[test]
    fn mentions_working_directory() {
        let text = execute("").expect("should succeed");
        assert!(text.contains("working directory"));
    }
}
