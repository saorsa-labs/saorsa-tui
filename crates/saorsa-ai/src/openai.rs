//! OpenAI Chat Completions API provider.

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::{Result, SaorsaAiError};
use crate::message::{ContentBlock, Message, Role, ToolDefinition};
use crate::provider::{Provider, ProviderConfig, StreamingProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, Usage,
};

/// OpenAI Chat Completions API provider.
pub struct OpenAiProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider with the given configuration.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| SaorsaAiError::Network(e.to_string()))?;
        Ok(Self { config, client })
    }

    /// Build the request headers.
    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth_value = format!("Bearer {}", self.config.api_key);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| SaorsaAiError::Auth(format!("invalid API key: {e}")))?,
        );
        Ok(headers)
    }

    /// Build the API URL for chat completions.
    fn url(&self) -> String {
        format!("{}/v1/chat/completions", self.config.base_url)
    }

    /// Convert an internal `CompletionRequest` to OpenAI's JSON format.
    fn build_oai_request(request: &CompletionRequest) -> OaiRequest {
        let mut oai_messages = Vec::new();

        // Prepend system message if present.
        if let Some(system) = &request.system {
            oai_messages.push(OaiMessage {
                role: "system".to_string(),
                content: Some(OaiContent::String(system.clone())),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Convert each message.
        for msg in &request.messages {
            let converted = Self::convert_message(msg);
            oai_messages.extend(converted);
        }

        // Convert tools.
        let tools = if request.tools.is_empty() {
            None
        } else {
            Some(
                request
                    .tools
                    .iter()
                    .map(Self::convert_tool_definition)
                    .collect(),
            )
        };

        // Convert stop sequences.
        let stop = if request.stop_sequences.is_empty() {
            None
        } else {
            Some(request.stop_sequences.clone())
        };

        OaiRequest {
            model: request.model.clone(),
            messages: oai_messages,
            max_tokens: Some(request.max_tokens),
            temperature: request.temperature,
            tools,
            stream: false,
            stop,
        }
    }

    /// Convert an internal `Message` into one or more OpenAI messages.
    ///
    /// A single internal message may produce multiple OpenAI messages when
    /// it contains both text and tool-result content blocks.
    fn convert_message(msg: &Message) -> Vec<OaiMessage> {
        let role_str = match msg.role {
            Role::User => "user",
            Role::Assistant => "assistant",
        };

        // Check if this message contains tool results — these become
        // separate "tool" role messages in OpenAI format.
        let has_tool_results = msg
            .content
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolResult { .. }));

        if has_tool_results {
            // Each tool result becomes its own message.
            return msg
                .content
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::ToolResult {
                        tool_use_id,
                        content,
                    } => Some(OaiMessage {
                        role: "tool".to_string(),
                        content: Some(OaiContent::String(content.clone())),
                        tool_calls: None,
                        tool_call_id: Some(tool_use_id.clone()),
                    }),
                    _ => None,
                })
                .collect();
        }

        // Check if this is an assistant message with tool calls.
        let has_tool_use = msg
            .content
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolUse { .. }));

        if has_tool_use {
            // Collect text content (if any).
            let text_content: Option<String> = {
                let texts: Vec<&str> = msg
                    .content
                    .iter()
                    .filter_map(|b| match b {
                        ContentBlock::Text { text } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect();
                if texts.is_empty() {
                    None
                } else {
                    Some(texts.join(""))
                }
            };

            // Collect tool calls.
            let tool_calls: Vec<OaiToolCall> = msg
                .content
                .iter()
                .filter_map(|b| match b {
                    ContentBlock::ToolUse { id, name, input } => Some(OaiToolCall {
                        id: id.clone(),
                        call_type: "function".to_string(),
                        function: OaiFunctionCall {
                            name: name.clone(),
                            arguments: input.to_string(),
                        },
                    }),
                    _ => None,
                })
                .collect();

            return vec![OaiMessage {
                role: role_str.to_string(),
                content: text_content.map(OaiContent::String),
                tool_calls: Some(tool_calls),
                tool_call_id: None,
            }];
        }

        // Standard text message.
        let content_parts: Vec<&str> = msg
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();

        let content_str = content_parts.join("");

        vec![OaiMessage {
            role: role_str.to_string(),
            content: Some(OaiContent::String(content_str)),
            tool_calls: None,
            tool_call_id: None,
        }]
    }

    /// Convert a `ToolDefinition` to OpenAI tool format.
    fn convert_tool_definition(tool: &ToolDefinition) -> OaiTool {
        OaiTool {
            tool_type: "function".to_string(),
            function: OaiFunction {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.input_schema.clone(),
            },
        }
    }

    /// Parse an OpenAI response into an internal `CompletionResponse`.
    fn parse_oai_response(oai: OaiResponse) -> Result<CompletionResponse> {
        let choice = oai
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| SaorsaAiError::Provider {
                provider: "openai".into(),
                message: "response contained no choices".into(),
            })?;

        let mut content = Vec::new();

        // Add text content if present.
        if let Some(text) = choice.message.content
            && !text.is_empty()
        {
            content.push(ContentBlock::Text { text });
        }

        // Add tool calls if present.
        if let Some(tool_calls) = choice.message.tool_calls {
            for tc in tool_calls {
                let input: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or_else(|_| serde_json::Value::String(tc.function.arguments.clone()));
                content.push(ContentBlock::ToolUse {
                    id: tc.id,
                    name: tc.function.name,
                    input,
                });
            }
        }

        let stop_reason = choice.finish_reason.map(|r| match r.as_str() {
            "stop" => StopReason::EndTurn,
            "length" => StopReason::MaxTokens,
            "tool_calls" => StopReason::ToolUse,
            _ => StopReason::EndTurn,
        });

        let usage = Usage {
            input_tokens: oai.usage.prompt_tokens,
            output_tokens: oai.usage.completion_tokens,
        };

        Ok(CompletionResponse {
            id: oai.id,
            content,
            model: oai.model,
            stop_reason,
            usage,
        })
    }

    /// Parse an SSE event from an OpenAI streaming response.
    ///
    /// Returns `None` for the `[DONE]` sentinel or unrecognised data.
    pub fn parse_sse_event(data: &str) -> Option<StreamEvent> {
        if data == "[DONE]" {
            return Some(StreamEvent::MessageStop);
        }

        let chunk: std::result::Result<OaiStreamChunk, _> = serde_json::from_str(data);
        let chunk = chunk.ok()?;

        let choice = chunk.choices.into_iter().next()?;

        // Text delta.
        if let Some(content) = choice.delta.content
            && !content.is_empty()
        {
            return Some(StreamEvent::ContentBlockDelta {
                index: 0,
                delta: ContentDelta::TextDelta { text: content },
            });
        }

        // Tool call delta.
        if let Some(tool_calls) = choice.delta.tool_calls {
            for tc in tool_calls {
                if let Some(function) = tc.function
                    && let Some(args) = function.arguments
                {
                    return Some(StreamEvent::ContentBlockDelta {
                        index: tc.index.unwrap_or(0),
                        delta: ContentDelta::InputJsonDelta { partial_json: args },
                    });
                }
            }
        }

        // Finish reason.
        if let Some(reason) = choice.finish_reason {
            let stop_reason = match reason.as_str() {
                "stop" => StopReason::EndTurn,
                "length" => StopReason::MaxTokens,
                "tool_calls" => StopReason::ToolUse,
                _ => StopReason::EndTurn,
            };
            return Some(StreamEvent::MessageDelta {
                stop_reason: Some(stop_reason),
                usage: chunk.usage.map_or_else(Usage::default, |u| Usage {
                    input_tokens: u.prompt_tokens.unwrap_or(0),
                    output_tokens: u.completion_tokens.unwrap_or(0),
                }),
            });
        }

        None
    }
}

#[async_trait::async_trait]
impl Provider for OpenAiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let headers = self.headers()?;
        let url = self.url();
        let oai_request = Self::build_oai_request(&request);

        debug!(model = %request.model, "Sending OpenAI completion request");

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&oai_request)
            .send()
            .await
            .map_err(|e| SaorsaAiError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".into());
            return match status.as_u16() {
                401 => Err(SaorsaAiError::Auth(body)),
                429 => Err(SaorsaAiError::RateLimit(body)),
                _ => Err(SaorsaAiError::Provider {
                    provider: "openai".into(),
                    message: format!("HTTP {status}: {body}"),
                }),
            };
        }

        let oai_response: OaiResponse =
            response.json().await.map_err(|e| SaorsaAiError::Provider {
                provider: "openai".into(),
                message: format!("response parse error: {e}"),
            })?;

        Self::parse_oai_response(oai_response)
    }
}

#[async_trait::async_trait]
impl StreamingProvider for OpenAiProvider {
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamEvent>>> {
        let headers = self.headers()?;
        let url = self.url();
        let mut oai_request = Self::build_oai_request(&request);
        oai_request.stream = true;

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&oai_request)
            .send()
            .await
            .map_err(|e| SaorsaAiError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".into());
            return match status.as_u16() {
                401 => Err(SaorsaAiError::Auth(body)),
                429 => Err(SaorsaAiError::RateLimit(body)),
                _ => Err(SaorsaAiError::Provider {
                    provider: "openai".into(),
                    message: format!("HTTP {status}: {body}"),
                }),
            };
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(Err(SaorsaAiError::Streaming(e.to_string()))).await;
                        break;
                    }
                };

                let text = String::from_utf8_lossy(&chunk);
                buffer.push_str(&text);

                // Parse SSE lines: "data: ..." separated by double newlines.
                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_string();
                    buffer = buffer[pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ")
                        && let Some(event) = Self::parse_sse_event(data)
                    {
                        let is_done = matches!(event, StreamEvent::MessageStop);
                        if tx.send(Ok(event)).await.is_err() {
                            return;
                        }
                        if is_done {
                            return;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}

// ── Internal OpenAI-specific types ──────────────────────────────────────────

/// OpenAI request body.
#[derive(Serialize)]
struct OaiRequest {
    model: String,
    messages: Vec<OaiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OaiTool>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

/// OpenAI message content — either a plain string or structured parts.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OaiContent {
    /// Plain string content.
    String(String),
}

/// An OpenAI chat message.
#[derive(Serialize)]
struct OaiMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<OaiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OaiToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

/// An OpenAI tool call within an assistant message.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OaiToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OaiFunctionCall,
}

/// Function name + arguments inside a tool call.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OaiFunctionCall {
    name: String,
    arguments: String,
}

/// An OpenAI tool definition.
#[derive(Serialize)]
struct OaiTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OaiFunction,
}

/// The function details within a tool definition.
#[derive(Serialize)]
struct OaiFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// Top-level OpenAI completion response.
#[derive(Deserialize)]
struct OaiResponse {
    id: String,
    model: String,
    choices: Vec<OaiChoice>,
    usage: OaiUsage,
}

/// A choice within an OpenAI response.
#[derive(Deserialize)]
struct OaiChoice {
    message: OaiResponseMessage,
    finish_reason: Option<String>,
}

/// The message body within a response choice.
#[derive(Deserialize)]
struct OaiResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OaiToolCall>>,
}

/// Token usage in an OpenAI response.
#[derive(Deserialize)]
struct OaiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

// ── Streaming types ─────────────────────────────────────────────────────────

/// A streaming chunk from the OpenAI API.
#[derive(Deserialize)]
struct OaiStreamChunk {
    choices: Vec<OaiStreamChoice>,
    usage: Option<OaiStreamUsage>,
}

/// A choice within a streaming chunk.
#[derive(Deserialize)]
struct OaiStreamChoice {
    delta: OaiStreamDelta,
    finish_reason: Option<String>,
}

/// Delta content within a streaming choice.
#[derive(Deserialize)]
struct OaiStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OaiStreamToolCall>>,
}

/// A tool call delta in a streaming chunk.
#[derive(Deserialize)]
struct OaiStreamToolCall {
    index: Option<u32>,
    function: Option<OaiStreamFunctionDelta>,
}

/// Function delta within a streaming tool call.
#[derive(Deserialize)]
struct OaiStreamFunctionDelta {
    arguments: Option<String>,
}

/// Streaming usage info (may have optional fields).
#[derive(Deserialize)]
struct OaiStreamUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ProviderKind;

    #[test]
    fn provider_creation() {
        let config = ProviderConfig::new(ProviderKind::OpenAi, "sk-test", "gpt-4o");
        let provider = OpenAiProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn request_serialization_basic() {
        let request = CompletionRequest::new("gpt-4o", vec![Message::user("Hello")], 1024);
        let oai = OpenAiProvider::build_oai_request(&request);

        let json = serde_json::to_value(&oai);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["model"], "gpt-4o");
            assert_eq!(v["max_tokens"], 1024);
            assert_eq!(v["messages"][0]["role"], "user");
            assert_eq!(v["messages"][0]["content"], "Hello");
        }
    }

    #[test]
    fn request_serialization_with_system() {
        let request =
            CompletionRequest::new("gpt-4o", vec![Message::user("Hi")], 512).system("Be helpful");
        let oai = OpenAiProvider::build_oai_request(&request);

        let json = serde_json::to_value(&oai);
        assert!(json.is_ok());
        if let Ok(v) = json {
            // System message should be first.
            assert_eq!(v["messages"][0]["role"], "system");
            assert_eq!(v["messages"][0]["content"], "Be helpful");
            assert_eq!(v["messages"][1]["role"], "user");
        }
    }

    #[test]
    fn request_serialization_with_tools() {
        let tool = ToolDefinition::new(
            "bash",
            "Run a command",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                }
            }),
        );
        let request =
            CompletionRequest::new("gpt-4o", vec![Message::user("ls")], 1024).tools(vec![tool]);
        let oai = OpenAiProvider::build_oai_request(&request);

        let json = serde_json::to_value(&oai);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["tools"][0]["type"], "function");
            assert_eq!(v["tools"][0]["function"]["name"], "bash");
            assert_eq!(v["tools"][0]["function"]["description"], "Run a command");
        }
    }

    #[test]
    fn request_serialization_tool_use_message() {
        let msg = Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolUse {
                id: "call_123".into(),
                name: "bash".into(),
                input: serde_json::json!({"command": "ls"}),
            }],
        };
        let request = CompletionRequest::new("gpt-4o", vec![msg], 1024);
        let oai = OpenAiProvider::build_oai_request(&request);

        let json = serde_json::to_value(&oai);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["messages"][0]["role"], "assistant");
            assert_eq!(v["messages"][0]["tool_calls"][0]["id"], "call_123");
            assert_eq!(v["messages"][0]["tool_calls"][0]["type"], "function");
            assert_eq!(
                v["messages"][0]["tool_calls"][0]["function"]["name"],
                "bash"
            );
        }
    }

    #[test]
    fn request_serialization_tool_result_message() {
        let msg = Message::tool_result("call_123", "file.txt");
        let request = CompletionRequest::new("gpt-4o", vec![msg], 1024);
        let oai = OpenAiProvider::build_oai_request(&request);

        let json = serde_json::to_value(&oai);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["messages"][0]["role"], "tool");
            assert_eq!(v["messages"][0]["tool_call_id"], "call_123");
            assert_eq!(v["messages"][0]["content"], "file.txt");
        }
    }

    #[test]
    fn response_parsing_text() {
        let json = r#"{
            "id": "chatcmpl-123",
            "model": "gpt-4o",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }"#;

        let oai: std::result::Result<OaiResponse, _> = serde_json::from_str(json);
        assert!(oai.is_ok());
        if let Ok(oai) = oai {
            let resp = OpenAiProvider::parse_oai_response(oai);
            assert!(resp.is_ok());
            if let Ok(resp) = resp {
                assert_eq!(resp.id, "chatcmpl-123");
                assert_eq!(resp.model, "gpt-4o");
                assert_eq!(resp.stop_reason, Some(StopReason::EndTurn));
                assert_eq!(resp.usage.input_tokens, 10);
                assert_eq!(resp.usage.output_tokens, 5);
                assert_eq!(resp.content.len(), 1);
                match &resp.content[0] {
                    ContentBlock::Text { text } => assert_eq!(text, "Hello!"),
                    _ => unreachable!("expected Text content block"),
                }
            }
        }
    }

    #[test]
    fn response_parsing_tool_calls() {
        let json = r#"{
            "id": "chatcmpl-456",
            "model": "gpt-4o",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_abc",
                        "type": "function",
                        "function": {
                            "name": "bash",
                            "arguments": "{\"command\": \"ls\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {
                "prompt_tokens": 20,
                "completion_tokens": 10,
                "total_tokens": 30
            }
        }"#;

        let oai: std::result::Result<OaiResponse, _> = serde_json::from_str(json);
        assert!(oai.is_ok());
        if let Ok(oai) = oai {
            let resp = OpenAiProvider::parse_oai_response(oai);
            assert!(resp.is_ok());
            if let Ok(resp) = resp {
                assert_eq!(resp.stop_reason, Some(StopReason::ToolUse));
                assert_eq!(resp.content.len(), 1);
                match &resp.content[0] {
                    ContentBlock::ToolUse { id, name, input } => {
                        assert_eq!(id, "call_abc");
                        assert_eq!(name, "bash");
                        assert_eq!(input["command"], "ls");
                    }
                    _ => unreachable!("expected ToolUse content block"),
                }
            }
        }
    }

    #[test]
    fn response_parsing_length_finish() {
        let json = r#"{
            "id": "chatcmpl-789",
            "model": "gpt-4o",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Truncated..."
                },
                "finish_reason": "length"
            }],
            "usage": {
                "prompt_tokens": 50,
                "completion_tokens": 100,
                "total_tokens": 150
            }
        }"#;

        let oai: std::result::Result<OaiResponse, _> = serde_json::from_str(json);
        assert!(oai.is_ok());
        if let Ok(oai) = oai {
            let resp = OpenAiProvider::parse_oai_response(oai);
            assert!(resp.is_ok());
            if let Ok(resp) = resp {
                assert_eq!(resp.stop_reason, Some(StopReason::MaxTokens));
            }
        }
    }

    #[test]
    fn response_empty_choices_returns_error() {
        let json = r#"{
            "id": "chatcmpl-err",
            "model": "gpt-4o",
            "choices": [],
            "usage": {
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0
            }
        }"#;

        let oai: std::result::Result<OaiResponse, _> = serde_json::from_str(json);
        assert!(oai.is_ok());
        if let Ok(oai) = oai {
            let resp = OpenAiProvider::parse_oai_response(oai);
            assert!(resp.is_err());
        }
    }

    #[test]
    fn parse_sse_text_delta() {
        let data =
            r#"{"id":"chatcmpl-1","choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let event = OpenAiProvider::parse_sse_event(data);
        match event {
            Some(StreamEvent::ContentBlockDelta { index, delta }) => {
                assert_eq!(index, 0);
                match delta {
                    ContentDelta::TextDelta { text } => assert_eq!(text, "Hello"),
                    _ => unreachable!("expected TextDelta"),
                }
            }
            _ => unreachable!("expected ContentBlockDelta"),
        }
    }

    #[test]
    fn parse_sse_done() {
        let event = OpenAiProvider::parse_sse_event("[DONE]");
        assert!(matches!(event, Some(StreamEvent::MessageStop)));
    }

    #[test]
    fn parse_sse_finish_reason() {
        let data = r#"{"id":"chatcmpl-1","choices":[{"delta":{},"finish_reason":"stop"}]}"#;
        let event = OpenAiProvider::parse_sse_event(data);
        match event {
            Some(StreamEvent::MessageDelta {
                stop_reason,
                usage: _,
            }) => {
                assert_eq!(stop_reason, Some(StopReason::EndTurn));
            }
            _ => unreachable!("expected MessageDelta"),
        }
    }

    #[test]
    fn parse_sse_tool_call_delta() {
        let data = r#"{"id":"chatcmpl-1","choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"cmd\":"}}]},"finish_reason":null}]}"#;
        let event = OpenAiProvider::parse_sse_event(data);
        match event {
            Some(StreamEvent::ContentBlockDelta { index, delta }) => {
                assert_eq!(index, 0);
                match delta {
                    ContentDelta::InputJsonDelta { partial_json } => {
                        assert!(partial_json.contains("cmd"));
                    }
                    _ => unreachable!("expected InputJsonDelta"),
                }
            }
            _ => unreachable!("expected ContentBlockDelta"),
        }
    }

    #[test]
    fn request_with_temperature_and_stop() {
        let request =
            CompletionRequest::new("gpt-4o", vec![Message::user("Hi")], 1024).temperature(0.5);
        let mut req_with_stop = request;
        req_with_stop.stop_sequences = vec!["END".to_string()];

        let oai = OpenAiProvider::build_oai_request(&req_with_stop);

        let json = serde_json::to_value(&oai);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["temperature"], 0.5);
            assert_eq!(v["stop"][0], "END");
        }
    }
}
