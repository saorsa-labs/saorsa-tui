//! Request, response, and streaming types for LLM APIs.

use serde::{Deserialize, Serialize};

use crate::message::{ContentBlock, Message, ToolDefinition};

/// A completion request to send to an LLM provider.
#[derive(Clone, Debug, Serialize)]
pub struct CompletionRequest {
    /// The model identifier.
    pub model: String,
    /// The conversation messages.
    pub messages: Vec<Message>,
    /// Maximum tokens to generate.
    pub max_tokens: u32,
    /// Optional system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Available tools.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub stream: bool,
    /// Sampling temperature (0.0-1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Stop sequences.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
}

impl CompletionRequest {
    /// Create a new request with required fields.
    pub fn new(model: impl Into<String>, messages: Vec<Message>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            messages,
            max_tokens,
            system: None,
            tools: Vec::new(),
            stream: false,
            temperature: None,
            stop_sequences: Vec::new(),
        }
    }

    /// Set the system prompt.
    #[must_use]
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set streaming mode.
    #[must_use]
    pub fn stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Set the temperature.
    #[must_use]
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Add tools.
    #[must_use]
    pub fn tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }
}

/// A completion response from an LLM provider.
#[derive(Clone, Debug, Deserialize)]
pub struct CompletionResponse {
    /// Unique response ID.
    pub id: String,
    /// The response content blocks.
    pub content: Vec<ContentBlock>,
    /// The model that generated this response.
    pub model: String,
    /// Why the model stopped generating.
    pub stop_reason: Option<StopReason>,
    /// Token usage information.
    pub usage: Usage,
}

/// Why the model stopped generating.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The model finished its response naturally.
    EndTurn,
    /// The response hit the max_tokens limit.
    MaxTokens,
    /// A stop sequence was encountered.
    StopSequence,
    /// The model wants to use a tool.
    ToolUse,
}

/// Token usage information.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Number of input tokens.
    #[serde(default)]
    pub input_tokens: u32,
    /// Number of output tokens.
    #[serde(default)]
    pub output_tokens: u32,
}

impl Usage {
    /// Total tokens (input + output).
    pub fn total(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// A streaming event from the LLM provider.
#[derive(Clone, Debug)]
pub enum StreamEvent {
    /// The message has started.
    MessageStart {
        /// Response ID.
        id: String,
        /// Model name.
        model: String,
        /// Usage so far.
        usage: Usage,
    },
    /// A content block has started.
    ContentBlockStart {
        /// Index of the content block.
        index: u32,
        /// The initial content block (may be partial).
        content_block: ContentBlock,
    },
    /// A delta (incremental update) to a content block.
    ContentBlockDelta {
        /// Index of the content block.
        index: u32,
        /// The delta content.
        delta: ContentDelta,
    },
    /// A content block has finished.
    ContentBlockStop {
        /// Index of the content block.
        index: u32,
    },
    /// Final message metadata.
    MessageDelta {
        /// Why the model stopped.
        stop_reason: Option<StopReason>,
        /// Final usage info.
        usage: Usage,
    },
    /// The message is complete.
    MessageStop,
    /// Keepalive ping.
    Ping,
    /// An error occurred.
    Error {
        /// Error message.
        message: String,
    },
}

/// Delta content for streaming updates.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentDelta {
    /// A text delta.
    TextDelta {
        /// The incremental text.
        text: String,
    },
    /// A tool input delta (partial JSON).
    InputJsonDelta {
        /// Partial JSON string.
        partial_json: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Message;

    #[test]
    fn request_builder() {
        let req = CompletionRequest::new("claude-sonnet-4-5-20250929", vec![Message::user("hi")], 1024)
            .system("You are helpful")
            .temperature(0.7)
            .stream(true);
        assert_eq!(req.model, "claude-sonnet-4-5-20250929");
        assert_eq!(req.max_tokens, 1024);
        assert!(req.stream);
        assert_eq!(req.temperature, Some(0.7));
        assert_eq!(req.system, Some("You are helpful".into()));
    }

    #[test]
    fn request_serialization() {
        let req = CompletionRequest::new("claude-sonnet-4-5-20250929", vec![Message::user("hi")], 1024);
        let json = serde_json::to_string(&req);
        assert!(json.is_ok());
        let json_str = json.as_deref().unwrap_or("");
        assert!(json_str.contains("claude-sonnet-4-5-20250929"));
        assert!(json_str.contains("1024"));
        // stream=false should not be serialized
        assert!(!json_str.contains("stream"));
    }

    #[test]
    fn response_parsing() {
        let json = r#"{
            "id": "msg_123",
            "content": [{"type": "text", "text": "Hello!"}],
            "model": "claude-sonnet-4-5-20250929",
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;
        let resp: std::result::Result<CompletionResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());
        if let Ok(resp) = resp {
            assert_eq!(resp.id, "msg_123");
            assert_eq!(resp.usage.total(), 15);
        }
    }

    #[test]
    fn stop_reason_parsing() {
        let json = r#""end_turn""#;
        let reason: Result<StopReason, _> = serde_json::from_str(json);
        assert_eq!(reason.ok(), Some(StopReason::EndTurn));

        let json = r#""tool_use""#;
        let reason: Result<StopReason, _> = serde_json::from_str(json);
        assert_eq!(reason.ok(), Some(StopReason::ToolUse));
    }

    #[test]
    fn usage_total() {
        let u = Usage {
            input_tokens: 100,
            output_tokens: 50,
        };
        assert_eq!(u.total(), 150);
    }

    #[test]
    fn content_delta_serialization() {
        let delta = ContentDelta::TextDelta {
            text: "hello".into(),
        };
        let json = serde_json::to_string(&delta);
        assert!(json.is_ok());
        assert!(json.as_deref().unwrap_or("").contains("text_delta"));
    }
}
