//! Session export functionality (HTML, plain text).

use crate::FaeAgentError;
use crate::session::{Message, SessionId, SessionMetadata, SessionStorage};
use std::fs;
use std::path::Path;

/// Export a session to HTML format.
pub fn export_to_html(
    storage: &SessionStorage,
    session_id: &SessionId,
    output_path: &Path,
) -> Result<(), FaeAgentError> {
    let metadata = storage.load_manifest(session_id)?;
    let messages = storage.load_messages(session_id)?;

    let html = generate_html(&metadata, &messages)?;

    fs::write(output_path, html)
        .map_err(|e| FaeAgentError::Session(format!("Failed to write HTML file: {}", e)))?;

    Ok(())
}

/// Generate HTML from session data.
fn generate_html(
    metadata: &SessionMetadata,
    messages: &[Message],
) -> Result<String, FaeAgentError> {
    let mut html = String::new();

    // HTML header
    html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<title>");
    html.push_str(&html_escape(
        metadata.title.as_deref().unwrap_or("Fae Session Export"),
    ));
    html.push_str("</title>\n");

    // Embedded CSS
    html.push_str("<style>\n");
    html.push_str("body { font-family: -apple-system, system-ui, sans-serif; max-width: 800px; margin: 40px auto; padding: 20px; }\n");
    html.push_str(
        ".header { border-bottom: 2px solid #333; padding-bottom: 20px; margin-bottom: 30px; }\n",
    );
    html.push_str(".message { margin: 20px 0; padding: 15px; border-radius: 8px; }\n");
    html.push_str(".user { background: #e3f2fd; }\n");
    html.push_str(".assistant { background: #f5f5f5; }\n");
    html.push_str(".tool { background: #fff3e0; font-family: monospace; }\n");
    html.push_str(".role { font-weight: bold; margin-bottom: 5px; }\n");
    html.push_str(".timestamp { color: #666; font-size: 0.9em; }\n");
    html.push_str("pre { white-space: pre-wrap; }\n");
    html.push_str("</style>\n</head>\n<body>\n");

    // Header
    html.push_str("<div class=\"header\">\n");
    html.push_str("<h1>");
    html.push_str(&html_escape(
        metadata.title.as_deref().unwrap_or("Untitled Session"),
    ));
    html.push_str("</h1>\n");
    html.push_str("<p class=\"timestamp\">Created: ");
    html.push_str(&metadata.created.format("%Y-%m-%d %H:%M:%S").to_string());
    html.push_str("</p>\n");
    html.push_str("</div>\n");

    // Messages
    for msg in messages {
        match msg {
            Message::User { content, timestamp } => {
                html.push_str("<div class=\"message user\">\n");
                html.push_str("<div class=\"role\">User</div>\n");
                html.push_str("<div class=\"timestamp\">");
                html.push_str(&timestamp.format("%H:%M:%S").to_string());
                html.push_str("</div>\n");
                html.push_str("<pre>");
                html.push_str(&html_escape(content));
                html.push_str("</pre>\n</div>\n");
            }
            Message::Assistant { content, timestamp } => {
                html.push_str("<div class=\"message assistant\">\n");
                html.push_str("<div class=\"role\">Assistant</div>\n");
                html.push_str("<div class=\"timestamp\">");
                html.push_str(&timestamp.format("%H:%M:%S").to_string());
                html.push_str("</div>\n");
                html.push_str("<pre>");
                html.push_str(&html_escape(content));
                html.push_str("</pre>\n</div>\n");
            }
            Message::ToolCall {
                tool_name,
                tool_input,
                timestamp,
            } => {
                html.push_str("<div class=\"message tool\">\n");
                html.push_str("<div class=\"role\">Tool Call: ");
                html.push_str(&html_escape(tool_name));
                html.push_str("</div>\n");
                html.push_str("<div class=\"timestamp\">");
                html.push_str(&timestamp.format("%H:%M:%S").to_string());
                html.push_str("</div>\n");
                html.push_str("<pre>");
                html.push_str(&html_escape(&tool_input.to_string()));
                html.push_str("</pre>\n</div>\n");
            }
            Message::ToolResult {
                tool_name,
                result,
                timestamp,
            } => {
                html.push_str("<div class=\"message tool\">\n");
                html.push_str("<div class=\"role\">Tool Result: ");
                html.push_str(&html_escape(tool_name));
                html.push_str("</div>\n");
                html.push_str("<div class=\"timestamp\">");
                html.push_str(&timestamp.format("%H:%M:%S").to_string());
                html.push_str("</div>\n");
                html.push_str("<pre>");
                html.push_str(&html_escape(&result.to_string()));
                html.push_str("</pre>\n</div>\n");
            }
        }
    }

    html.push_str("</body>\n</html>\n");
    Ok(html)
}

/// HTML escape special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_html_escape() {
        assert!(html_escape("<script>") == "&lt;script&gt;");
        assert!(html_escape("A & B") == "A &amp; B");
    }

    #[test]
    fn test_export_to_html() {
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(_) => panic!("Failed to create temp dir"),
        };

        let storage = SessionStorage::with_base_path(temp_dir.path().to_path_buf());
        let session_id = SessionId::new();
        let mut metadata = SessionMetadata::new();
        metadata.title = Some("Test".to_string());

        assert!(storage.save_manifest(&session_id, &metadata).is_ok());
        assert!(
            storage
                .save_message(&session_id, 0, &Message::user("Hello".to_string()))
                .is_ok()
        );

        let output = temp_dir.path().join("export.html");
        let result = export_to_html(&storage, &session_id, &output);
        assert!(result.is_ok());
        assert!(output.exists());
    }
}
