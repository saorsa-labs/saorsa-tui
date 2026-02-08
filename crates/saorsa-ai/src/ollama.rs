//! Ollama Chat API provider for local inference.
//!
//! Ollama runs locally and exposes a Chat API at `/api/chat`.
//! Streaming uses NDJSON (newline-delimited JSON), not SSE.

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::{Result, SaorsaAiError};
use crate::message::{ContentBlock, Message, Role, ToolDefinition};
use crate::provider::{Provider, ProviderConfig, StreamingProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, Usage,
};

/// Ollama Chat API provider for local inference.
///
/// Connects to a local (or remote) Ollama server. Authentication is optional —
/// set `api_key` to a non-empty string if the server requires a Bearer token.
pub struct OllamaProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Create a new Ollama provider with the given configuration.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| SaorsaAiError::Network(e.to_string()))?;
        Ok(Self { config, client })
    }

    /// Build request headers.
    ///
    /// Ollama normally requires no auth, but if `api_key` is non-empty we send
    /// it as a Bearer token (some deployments use reverse-proxy auth).
    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if !self.config.api_key.is_empty() {
            let auth_value = format!("Bearer {}", self.config.api_key);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth_value)
                    .map_err(|e| SaorsaAiError::Auth(format!("invalid API key: {e}")))?,
            );
        }
        Ok(headers)
    }

    /// Build the API URL for the chat endpoint.
    fn url(&self) -> String {
        format!("{}/api/chat", self.config.base_url)
    }
}

// ---------------------------------------------------------------------------
// Request building
// ---------------------------------------------------------------------------

/// Build an Ollama chat request from our unified `CompletionRequest`.
fn build_ollama_request(request: &CompletionRequest, stream: bool) -> OllamaRequest {
    let mut messages = Vec::new();

    // System prompt becomes a "system" role message.
    if let Some(system) = &request.system {
        messages.push(OllamaMessage {
            role: "system".to_string(),
            content: system.clone(),
            tool_calls: None,
        });
    }

    // Convert conversation messages.
    for msg in &request.messages {
        let converted = convert_message(msg);
        messages.extend(converted);
    }

    // Convert tools (Ollama uses OpenAI-compatible format).
    let tools = if request.tools.is_empty() {
        None
    } else {
        Some(request.tools.iter().map(convert_tool_definition).collect())
    };

    // Build options.
    let options = if request.temperature.is_some() {
        Some(OllamaOptions {
            temperature: request.temperature,
        })
    } else {
        None
    };

    OllamaRequest {
        model: request.model.clone(),
        messages,
        stream,
        tools,
        options,
    }
}

/// Convert an internal `Message` to one or more Ollama messages.
fn convert_message(msg: &Message) -> Vec<OllamaMessage> {
    let role_str = match msg.role {
        Role::User => "user",
        Role::Assistant => "assistant",
    };

    // Check for tool results — each becomes a "tool" role message.
    let has_tool_results = msg
        .content
        .iter()
        .any(|b| matches!(b, ContentBlock::ToolResult { .. }));

    if has_tool_results {
        return msg
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolResult { content, .. } => Some(OllamaMessage {
                    role: "tool".to_string(),
                    content: content.clone(),
                    tool_calls: None,
                }),
                _ => None,
            })
            .collect();
    }

    // Check for tool use — assistant messages with function calls.
    let has_tool_use = msg
        .content
        .iter()
        .any(|b| matches!(b, ContentBlock::ToolUse { .. }));

    if has_tool_use {
        let text: String = msg
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect();

        let tool_calls: Vec<OllamaToolCall> = msg
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::ToolUse { name, input, .. } => Some(OllamaToolCall {
                    function: OllamaFunctionCall {
                        name: name.clone(),
                        arguments: input.clone(),
                    },
                }),
                _ => None,
            })
            .collect();

        return vec![OllamaMessage {
            role: role_str.to_string(),
            content: text,
            tool_calls: Some(tool_calls),
        }];
    }

    // Standard text message.
    let content: String = msg
        .content
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();

    vec![OllamaMessage {
        role: role_str.to_string(),
        content,
        tool_calls: None,
    }]
}

/// Convert a `ToolDefinition` to Ollama tool format (OpenAI-compatible).
fn convert_tool_definition(tool: &ToolDefinition) -> OllamaTool {
    OllamaTool {
        tool_type: "function".to_string(),
        function: OllamaFunction {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: tool.input_schema.clone(),
        },
    }
}

// ---------------------------------------------------------------------------
// Response parsing
// ---------------------------------------------------------------------------

/// Parse a non-streaming Ollama response into a `CompletionResponse`.
fn parse_ollama_response(resp: &OllamaResponse) -> CompletionResponse {
    let mut content = Vec::new();

    // Add text content.
    if !resp.message.content.is_empty() {
        content.push(ContentBlock::Text {
            text: resp.message.content.clone(),
        });
    }

    // Add tool calls.
    if let Some(tool_calls) = &resp.message.tool_calls {
        for (i, tc) in tool_calls.iter().enumerate() {
            content.push(ContentBlock::ToolUse {
                id: format!("call_{i}"),
                name: tc.function.name.clone(),
                input: tc.function.arguments.clone(),
            });
        }
    }

    let stop_reason = resp.done_reason.as_deref().map(map_done_reason);

    let usage = Usage {
        input_tokens: resp.prompt_eval_count.unwrap_or(0),
        output_tokens: resp.eval_count.unwrap_or(0),
        ..Usage::default()
    };

    CompletionResponse {
        id: String::new(),
        content,
        model: resp.model.clone(),
        stop_reason,
        usage,
    }
}

/// Map an Ollama `done_reason` string to our `StopReason`.
fn map_done_reason(reason: &str) -> StopReason {
    match reason {
        "stop" => StopReason::EndTurn,
        "length" => StopReason::MaxTokens,
        _ => StopReason::EndTurn,
    }
}

/// Parse a single NDJSON streaming chunk into a `StreamEvent`.
///
/// Returns `None` for chunks that contain no actionable data.
pub fn parse_ndjson_chunk(data: &str) -> Option<StreamEvent> {
    let chunk: std::result::Result<OllamaStreamChunk, _> = serde_json::from_str(data);
    let chunk = chunk.ok()?;

    // Final chunk with done=true.
    if chunk.done {
        let usage = Usage {
            input_tokens: chunk.prompt_eval_count.unwrap_or(0),
            output_tokens: chunk.eval_count.unwrap_or(0),
            ..Usage::default()
        };
        let stop_reason = chunk.done_reason.as_deref().map(map_done_reason);
        return Some(StreamEvent::MessageDelta { stop_reason, usage });
    }

    // Tool call chunk.
    if let Some(msg) = &chunk.message {
        if let Some(tool_calls) = &msg.tool_calls
            && let Some(tc) = tool_calls.first()
        {
            return Some(StreamEvent::ContentBlockDelta {
                index: 0,
                delta: ContentDelta::InputJsonDelta {
                    partial_json: tc.function.arguments.to_string(),
                },
            });
        }

        // Text delta.
        if !msg.content.is_empty() {
            return Some(StreamEvent::ContentBlockDelta {
                index: 0,
                delta: ContentDelta::TextDelta {
                    text: msg.content.clone(),
                },
            });
        }
    }

    None
}

/// Map an HTTP status code to the appropriate `SaorsaAiError`.
fn handle_http_error(status: reqwest::StatusCode, body: &str) -> SaorsaAiError {
    match status.as_u16() {
        401 | 403 => SaorsaAiError::Auth(format!("Ollama auth error ({}): {}", status, body)),
        429 => SaorsaAiError::RateLimit(format!("Ollama rate limit ({}): {}", status, body)),
        _ => SaorsaAiError::Provider {
            provider: "Ollama".to_string(),
            message: format!("HTTP {} — {}", status, body),
        },
    }
}

// ---------------------------------------------------------------------------
// Trait implementations
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl Provider for OllamaProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let ollama_req = build_ollama_request(&request, false);
        let body = serde_json::to_string(&ollama_req).map_err(SaorsaAiError::Json)?;

        debug!("Ollama request to {}", self.url());

        let response = self
            .client
            .post(self.url())
            .headers(self.headers()?)
            .body(body)
            .send()
            .await
            .map_err(|e| SaorsaAiError::Network(e.to_string()))?;

        let status = response.status();
        let response_body = response
            .text()
            .await
            .map_err(|e| SaorsaAiError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(handle_http_error(status, &response_body));
        }

        let ollama_resp: OllamaResponse =
            serde_json::from_str(&response_body).map_err(|e| SaorsaAiError::Provider {
                provider: "Ollama".to_string(),
                message: format!("failed to parse response: {e}"),
            })?;

        Ok(parse_ollama_response(&ollama_resp))
    }
}

#[async_trait::async_trait]
impl StreamingProvider for OllamaProvider {
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamEvent>>> {
        let ollama_req = build_ollama_request(&request, true);
        let body = serde_json::to_string(&ollama_req).map_err(SaorsaAiError::Json)?;

        debug!("Ollama stream request to {}", self.url());

        let response = self
            .client
            .post(self.url())
            .headers(self.headers()?)
            .body(body)
            .send()
            .await
            .map_err(|e| SaorsaAiError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .map_err(|e| SaorsaAiError::Network(e.to_string()))?;
            return Err(handle_http_error(status, &error_body));
        }

        let model = request.model.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        // Send initial MessageStart event.
        let start_event = StreamEvent::MessageStart {
            id: String::new(),
            model,
            usage: Usage::default(),
        };
        let _ = tx.send(Ok(start_event)).await;

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        let _ = tx.send(Err(SaorsaAiError::Streaming(e.to_string()))).await;
                        break;
                    }
                };

                let text = match std::str::from_utf8(&chunk) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                buffer.push_str(text);

                // NDJSON: each line is a complete JSON object.
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if let Some(event) = parse_ndjson_chunk(&line) {
                        let is_done = matches!(event, StreamEvent::MessageDelta { .. });
                        if tx.send(Ok(event)).await.is_err() {
                            return;
                        }
                        if is_done {
                            // Send MessageStop after the final delta.
                            let _ = tx.send(Ok(StreamEvent::MessageStop)).await;
                            return;
                        }
                    }
                }
            }

            // Always send a final MessageStop if we haven't already.
            let _ = tx.send(Ok(StreamEvent::MessageStop)).await;
        });

        Ok(rx)
    }
}

// ---------------------------------------------------------------------------
// Internal Ollama-specific types
// ---------------------------------------------------------------------------

/// Ollama chat request body.
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OllamaTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

/// A message in the Ollama chat format.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaToolCall>>,
}

/// Tool call in Ollama format.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OllamaToolCall {
    function: OllamaFunctionCall,
}

/// Function call details.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OllamaFunctionCall {
    name: String,
    arguments: serde_json::Value,
}

/// Tool definition in Ollama format (OpenAI-compatible).
#[derive(Serialize)]
struct OllamaTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OllamaFunction,
}

/// Function definition within a tool.
#[derive(Serialize)]
struct OllamaFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// Ollama-specific options.
#[derive(Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

/// Ollama non-streaming response.
#[derive(Deserialize)]
struct OllamaResponse {
    model: String,
    message: OllamaMessage,
    #[serde(default)]
    done_reason: Option<String>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
}

/// Ollama NDJSON streaming chunk.
#[derive(Deserialize)]
struct OllamaStreamChunk {
    #[serde(default)]
    message: Option<OllamaMessage>,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    done_reason: Option<String>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Message;

    #[test]
    fn provider_creation() {
        let config = ProviderConfig::new(crate::provider::ProviderKind::Ollama, "", "llama3");
        let result = OllamaProvider::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn url_construction() {
        let config = ProviderConfig::new(crate::provider::ProviderKind::Ollama, "", "llama3");
        if let Ok(provider) = OllamaProvider::new(config) {
            assert_eq!(provider.url(), "http://localhost:11434/api/chat");
        }
    }

    #[test]
    fn url_construction_custom_base() {
        let config = ProviderConfig::new(crate::provider::ProviderKind::Ollama, "", "llama3")
            .with_base_url("http://remote-server:11434");
        if let Ok(provider) = OllamaProvider::new(config) {
            assert_eq!(provider.url(), "http://remote-server:11434/api/chat");
        }
    }

    #[test]
    fn request_serialization_basic() {
        let request = CompletionRequest::new("llama3", vec![Message::user("Hello")], 4096);
        let ollama_req = build_ollama_request(&request, false);
        let json = serde_json::to_value(&ollama_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["model"], "llama3");
            assert_eq!(v["stream"], false);
            assert_eq!(v["messages"][0]["role"], "user");
            assert_eq!(v["messages"][0]["content"], "Hello");
            assert!(v.get("tools").is_none());
            assert!(v.get("options").is_none());
        }
    }

    #[test]
    fn request_serialization_with_system() {
        let request = CompletionRequest::new("llama3", vec![Message::user("Hi")], 4096)
            .system("You are helpful");
        let ollama_req = build_ollama_request(&request, false);
        let json = serde_json::to_value(&ollama_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["messages"][0]["role"], "system");
            assert_eq!(v["messages"][0]["content"], "You are helpful");
            assert_eq!(v["messages"][1]["role"], "user");
            assert_eq!(v["messages"][1]["content"], "Hi");
        }
    }

    #[test]
    fn request_serialization_with_tools() {
        let tool = ToolDefinition::new(
            "get_weather",
            "Get current weather",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "city": {"type": "string"}
                },
                "required": ["city"]
            }),
        );
        let request = CompletionRequest::new("llama3", vec![Message::user("Weather?")], 4096)
            .tools(vec![tool]);
        let ollama_req = build_ollama_request(&request, false);
        let json = serde_json::to_value(&ollama_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
            let tools = &v["tools"];
            assert!(tools.is_array());
            if let Some(arr) = tools.as_array() {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr[0]["type"], "function");
                assert_eq!(arr[0]["function"]["name"], "get_weather");
            }
        }
    }

    #[test]
    fn request_serialization_with_temperature() {
        let request =
            CompletionRequest::new("llama3", vec![Message::user("Hi")], 4096).temperature(0.5);
        let ollama_req = build_ollama_request(&request, false);
        let json = serde_json::to_value(&ollama_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert!(v["options"]["temperature"].is_number());
        }
    }

    #[test]
    fn request_serialization_stream_flag() {
        let request = CompletionRequest::new("llama3", vec![Message::user("Hi")], 4096);
        let non_stream = build_ollama_request(&request, false);
        assert!(!non_stream.stream);
        let stream = build_ollama_request(&request, true);
        assert!(stream.stream);
    }

    #[test]
    fn request_serialization_tool_use_message() {
        let msg = Message {
            role: Role::Assistant,
            content: vec![
                ContentBlock::Text {
                    text: "Let me check.".into(),
                },
                ContentBlock::ToolUse {
                    id: "call_0".into(),
                    name: "get_weather".into(),
                    input: serde_json::json!({"city": "London"}),
                },
            ],
        };
        let request = CompletionRequest::new("llama3", vec![msg], 4096);
        let ollama_req = build_ollama_request(&request, false);
        let json = serde_json::to_value(&ollama_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["messages"][0]["role"], "assistant");
            assert_eq!(v["messages"][0]["content"], "Let me check.");
            let tool_calls = &v["messages"][0]["tool_calls"];
            assert!(tool_calls.is_array());
            if let Some(arr) = tool_calls.as_array() {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr[0]["function"]["name"], "get_weather");
            }
        }
    }

    #[test]
    fn request_serialization_tool_result_message() {
        let msg = Message::tool_result("call_0", "Sunny, 22C");
        let request = CompletionRequest::new("llama3", vec![msg], 4096);
        let ollama_req = build_ollama_request(&request, false);
        let json = serde_json::to_value(&ollama_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["messages"][0]["role"], "tool");
            assert_eq!(v["messages"][0]["content"], "Sunny, 22C");
        }
    }

    #[test]
    fn response_parsing_text() {
        let json = r#"{
            "model": "llama3",
            "created_at": "2024-01-01T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "Hello!"
            },
            "done": true,
            "done_reason": "stop",
            "eval_count": 50,
            "prompt_eval_count": 20
        }"#;
        let resp: std::result::Result<OllamaResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());
        if let Ok(resp) = resp {
            let parsed = parse_ollama_response(&resp);
            assert_eq!(parsed.model, "llama3");
            assert_eq!(parsed.content.len(), 1);
            match &parsed.content[0] {
                ContentBlock::Text { text } => assert_eq!(text, "Hello!"),
                _ => unreachable!("Expected text content"),
            }
            assert_eq!(parsed.stop_reason, Some(StopReason::EndTurn));
            assert_eq!(parsed.usage.input_tokens, 20);
            assert_eq!(parsed.usage.output_tokens, 50);
        }
    }

    #[test]
    fn response_parsing_with_tool_calls() {
        let json = r#"{
            "model": "llama3",
            "created_at": "2024-01-01T00:00:00Z",
            "message": {
                "role": "assistant",
                "content": "",
                "tool_calls": [
                    {
                        "function": {
                            "name": "get_weather",
                            "arguments": {"city": "London"}
                        }
                    }
                ]
            },
            "done": true,
            "done_reason": "stop",
            "eval_count": 30,
            "prompt_eval_count": 15
        }"#;
        let resp: std::result::Result<OllamaResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());
        if let Ok(resp) = resp {
            let parsed = parse_ollama_response(&resp);
            // Empty content string should not produce a text block.
            let tool_blocks: Vec<_> = parsed
                .content
                .iter()
                .filter(|b| matches!(b, ContentBlock::ToolUse { .. }))
                .collect();
            assert_eq!(tool_blocks.len(), 1);
            match &parsed.content[0] {
                ContentBlock::ToolUse { id, name, input } => {
                    assert_eq!(id, "call_0");
                    assert_eq!(name, "get_weather");
                    assert_eq!(input["city"], "London");
                }
                _ => unreachable!("Expected tool use content"),
            }
        }
    }

    #[test]
    fn response_parsing_length_done_reason() {
        let json = r#"{
            "model": "llama3",
            "created_at": "2024-01-01T00:00:00Z",
            "message": {"role": "assistant", "content": "Truncated..."},
            "done": true,
            "done_reason": "length"
        }"#;
        let resp: std::result::Result<OllamaResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());
        if let Ok(resp) = resp {
            let parsed = parse_ollama_response(&resp);
            assert_eq!(parsed.stop_reason, Some(StopReason::MaxTokens));
        }
    }

    #[test]
    fn parse_ndjson_text_delta() {
        let json = r#"{"model":"llama3","created_at":"2024-01-01T00:00:00Z","message":{"role":"assistant","content":"Hello"},"done":false}"#;
        let event = parse_ndjson_chunk(json);
        assert!(event.is_some());
        if let Some(StreamEvent::ContentBlockDelta { index, delta }) = event {
            assert_eq!(index, 0);
            match delta {
                ContentDelta::TextDelta { text } => assert_eq!(text, "Hello"),
                _ => unreachable!("Expected text delta"),
            }
        } else {
            unreachable!("Expected ContentBlockDelta");
        }
    }

    #[test]
    fn parse_ndjson_done_signal() {
        let json = r#"{"model":"llama3","created_at":"2024-01-01T00:00:00Z","message":{"role":"assistant","content":""},"done":true,"done_reason":"stop","eval_count":50,"prompt_eval_count":20}"#;
        let event = parse_ndjson_chunk(json);
        assert!(event.is_some());
        if let Some(StreamEvent::MessageDelta { stop_reason, usage }) = event {
            assert_eq!(stop_reason, Some(StopReason::EndTurn));
            assert_eq!(usage.input_tokens, 20);
            assert_eq!(usage.output_tokens, 50);
        } else {
            unreachable!("Expected MessageDelta");
        }
    }

    #[test]
    fn parse_ndjson_done_length() {
        let json =
            r#"{"done":true,"done_reason":"length","eval_count":100,"prompt_eval_count":50}"#;
        let event = parse_ndjson_chunk(json);
        assert!(event.is_some());
        if let Some(StreamEvent::MessageDelta { stop_reason, .. }) = event {
            assert_eq!(stop_reason, Some(StopReason::MaxTokens));
        } else {
            unreachable!("Expected MessageDelta");
        }
    }

    #[test]
    fn parse_ndjson_tool_call_delta() {
        let json = r#"{"model":"llama3","created_at":"2024-01-01T00:00:00Z","message":{"role":"assistant","content":"","tool_calls":[{"function":{"name":"get_weather","arguments":{"city":"London"}}}]},"done":false}"#;
        let event = parse_ndjson_chunk(json);
        assert!(event.is_some());
        if let Some(StreamEvent::ContentBlockDelta { delta, .. }) = event {
            match delta {
                ContentDelta::InputJsonDelta { partial_json } => {
                    assert!(partial_json.contains("London"));
                }
                _ => unreachable!("Expected InputJsonDelta"),
            }
        } else {
            unreachable!("Expected ContentBlockDelta");
        }
    }

    #[test]
    fn parse_ndjson_empty_content() {
        let json = r#"{"model":"llama3","created_at":"2024-01-01T00:00:00Z","message":{"role":"assistant","content":""},"done":false}"#;
        let event = parse_ndjson_chunk(json);
        assert!(event.is_none());
    }

    #[test]
    fn parse_ndjson_invalid_json() {
        let event = parse_ndjson_chunk("not valid json");
        assert!(event.is_none());
    }

    #[test]
    fn map_done_reason_variants() {
        assert_eq!(map_done_reason("stop"), StopReason::EndTurn);
        assert_eq!(map_done_reason("length"), StopReason::MaxTokens);
        assert_eq!(map_done_reason("unknown"), StopReason::EndTurn);
    }

    #[test]
    fn headers_no_auth() {
        let config = ProviderConfig::new(crate::provider::ProviderKind::Ollama, "", "llama3");
        if let Ok(provider) = OllamaProvider::new(config) {
            let headers = provider.headers();
            assert!(headers.is_ok());
            if let Ok(h) = headers {
                assert!(h.get(AUTHORIZATION).is_none());
            }
        }
    }

    #[test]
    fn headers_with_auth() {
        let config = ProviderConfig::new(
            crate::provider::ProviderKind::Ollama,
            "my-secret-key",
            "llama3",
        );
        if let Ok(provider) = OllamaProvider::new(config) {
            let headers = provider.headers();
            assert!(headers.is_ok());
            if let Ok(h) = headers {
                assert!(h.get(AUTHORIZATION).is_some());
            }
        }
    }
}
