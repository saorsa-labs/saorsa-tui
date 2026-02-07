//! Bash tool for executing shell commands.

use std::path::PathBuf;
use std::time::Duration;

use tracing::debug;

use crate::error::{Result, SaorsaAgentError};
use crate::tool::Tool;

/// Default command timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Maximum output length in bytes before truncation.
const MAX_OUTPUT_BYTES: usize = 100_000;

/// Tool for executing bash commands.
pub struct BashTool {
    /// Working directory for commands.
    working_dir: PathBuf,
    /// Command timeout.
    timeout: Duration,
}

impl BashTool {
    /// Create a new bash tool with the given working directory.
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    /// Set the command timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Truncate output if it exceeds the maximum length.
    fn truncate_output(output: &str) -> String {
        if output.len() > MAX_OUTPUT_BYTES {
            let truncated = &output[..MAX_OUTPUT_BYTES];
            format!(
                "{truncated}\n\n... (output truncated, {} bytes total)",
                output.len()
            )
        } else {
            output.to_string()
        }
    }
}

#[async_trait::async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash command and return stdout and stderr"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<String> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SaorsaAgentError::Tool("missing 'command' field".into()))?;

        debug!(command = %command, dir = %self.working_dir.display(), "Executing bash command");

        let result = tokio::time::timeout(
            self.timeout,
            tokio::process::Command::new("bash")
                .arg("-c")
                .arg(command)
                .current_dir(&self.working_dir)
                .output(),
        )
        .await;

        let output = match result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(SaorsaAgentError::Tool(format!(
                    "failed to execute command: {e}"
                )));
            }
            Err(_) => {
                return Err(SaorsaAgentError::Tool(format!(
                    "command timed out after {} seconds",
                    self.timeout.as_secs()
                )));
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let exit_code = output.status.code().unwrap_or(-1);

        let mut result_text = String::new();

        if !stdout.is_empty() {
            result_text.push_str(&stdout);
        }

        if !stderr.is_empty() {
            if !result_text.is_empty() {
                result_text.push('\n');
            }
            result_text.push_str("STDERR:\n");
            result_text.push_str(&stderr);
        }

        if exit_code != 0 {
            if !result_text.is_empty() {
                result_text.push('\n');
            }
            result_text.push_str(&format!("Exit code: {exit_code}"));
        }

        if result_text.is_empty() {
            result_text = "(no output)".to_string();
        }

        Ok(Self::truncate_output(&result_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tool() -> BashTool {
        BashTool::new(std::env::temp_dir())
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn execute_echo() {
        let tool = test_tool();
        let result = tool
            .execute(serde_json::json!({"command": "echo hello"}))
            .await;
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert!(output.contains("hello"));
        }
    }

    #[tokio::test]
    async fn execute_missing_command_field() {
        let tool = test_tool();
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn execute_failing_command() {
        let tool = test_tool();
        let result = tool
            .execute(serde_json::json!({"command": "exit 42"}))
            .await;
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert!(output.contains("Exit code: 42"));
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn execute_stderr() {
        let tool = test_tool();
        let result = tool
            .execute(serde_json::json!({"command": "echo error >&2"}))
            .await;
        assert!(result.is_ok());
        if let Ok(output) = result {
            assert!(output.contains("STDERR:"));
            assert!(output.contains("error"));
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn execute_timeout() {
        let tool = BashTool::new(std::env::temp_dir()).timeout(Duration::from_millis(100));
        let result = tool
            .execute(serde_json::json!({"command": "sleep 10"}))
            .await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("timed out"));
        }
    }

    #[test]
    fn tool_metadata() {
        let tool = test_tool();
        assert_eq!(tool.name(), "bash");
        assert!(!tool.description().is_empty());
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
    }

    #[test]
    fn truncate_long_output() {
        let long = "x".repeat(MAX_OUTPUT_BYTES + 1000);
        let truncated = BashTool::truncate_output(&long);
        assert!(truncated.len() < long.len());
        assert!(truncated.contains("truncated"));
    }

    #[test]
    fn truncate_short_output() {
        let short = "hello";
        let result = BashTool::truncate_output(short);
        assert_eq!(result, "hello");
    }
}
