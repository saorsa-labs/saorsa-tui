//! Google Gemini `generateContent` / `streamGenerateContent` API provider.

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::error::{Result, SaorsaAiError};
use crate::message::{ContentBlock, Message, Role, ToolDefinition};
use crate::provider::{Provider, ProviderConfig, StreamingProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, Usage,
};

// ---------------------------------------------------------------------------
// Public provider
// ---------------------------------------------------------------------------

/// Google Gemini `generateContent` API provider.
///
/// Supports both non-streaming (`generateContent`) and streaming
/// (`streamGenerateContent?alt=sse`) endpoints.  Auth is via the
/// `x-goog-api-key` header.
pub struct GeminiProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl GeminiProvider {
    /// Create a new Gemini provider with the given configuration.
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
        headers.insert(
            "x-goog-api-key",
            HeaderValue::from_str(&self.config.api_key)
                .map_err(|e| SaorsaAiError::Auth(format!("invalid API key: {e}")))?,
        );
        Ok(headers)
    }

    /// Build the non-streaming API URL.
    fn url(&self, model: &str) -> String {
        format!("{}/models/{}:generateContent", self.config.base_url, model)
    }

    /// Build the streaming API URL.
    fn stream_url(&self, model: &str) -> String {
        format!(
            "{}/models/{}:streamGenerateContent?alt=sse",
            self.config.base_url, model
        )
    }
}

// ---------------------------------------------------------------------------
// Request / response mapping (internal helpers)
// ---------------------------------------------------------------------------

/// Convert an internal `CompletionRequest` to Gemini's JSON format.
fn build_gemini_request(request: &CompletionRequest) -> GeminiRequest {
    let mut contents: Vec<GeminiContent> = Vec::new();

    // Gemini has no separate "system" field in the REST API body.
    // Prepend a user-role content with the system prompt if present.
    if let Some(system) = &request.system {
        contents.push(GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart::Text {
                text: system.clone(),
            }],
        });
        // Add a placeholder model response so the real conversation starts
        // from the correct turn (Gemini requires alternating user/model).
        contents.push(GeminiContent {
            role: "model".to_string(),
            parts: vec![GeminiPart::Text {
                text: "Understood.".to_string(),
            }],
        });
    }

    for msg in &request.messages {
        let converted = convert_message(msg);
        contents.extend(converted);
    }

    // Tools → functionDeclarations
    let tools = if request.tools.is_empty() {
        None
    } else {
        let declarations: Vec<GeminiFunctionDeclaration> =
            request.tools.iter().map(convert_tool_definition).collect();
        Some(vec![GeminiToolGroup {
            function_declarations: declarations,
        }])
    };

    let generation_config = GeminiGenerationConfig {
        max_output_tokens: Some(request.max_tokens),
        temperature: request.temperature,
        stop_sequences: if request.stop_sequences.is_empty() {
            None
        } else {
            Some(request.stop_sequences.clone())
        },
    };

    GeminiRequest {
        contents,
        tools,
        generation_config: Some(generation_config),
    }
}

/// Convert an internal `Message` into one or more Gemini content entries.
///
/// Tool results must be sent as a separate content entry with the
/// `functionResponse` part so Gemini can pair them.
fn convert_message(msg: &Message) -> Vec<GeminiContent> {
    let role = match msg.role {
        Role::User => "user",
        Role::Assistant => "model",
    };

    let mut result: Vec<GeminiContent> = Vec::new();
    let mut parts: Vec<GeminiPart> = Vec::new();

    for block in &msg.content {
        match block {
            ContentBlock::Text { text } => {
                parts.push(GeminiPart::Text { text: text.clone() });
            }
            ContentBlock::ToolUse { id: _, name, input } => {
                parts.push(GeminiPart::FunctionCall {
                    function_call: GeminiFunctionCall {
                        name: name.clone(),
                        args: input.clone(),
                    },
                });
            }
            ContentBlock::ToolResult {
                tool_use_id: _,
                content,
            } => {
                // Gemini requires functionResponse in a separate "user" content.
                // Flush any accumulated parts first.
                if !parts.is_empty() {
                    result.push(GeminiContent {
                        role: role.to_string(),
                        parts: std::mem::take(&mut parts),
                    });
                }
                result.push(GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart::FunctionResponse {
                        function_response: GeminiFunctionResponse {
                            name: String::new(), // name unknown from ToolResult; API tolerates empty
                            response: serde_json::json!({ "result": content }),
                        },
                    }],
                });
            }
        }
    }

    if !parts.is_empty() {
        result.push(GeminiContent {
            role: role.to_string(),
            parts,
        });
    }

    result
}

/// Convert an internal `ToolDefinition` to a Gemini function declaration.
fn convert_tool_definition(tool: &ToolDefinition) -> GeminiFunctionDeclaration {
    GeminiFunctionDeclaration {
        name: tool.name.clone(),
        description: tool.description.clone(),
        parameters: tool.input_schema.clone(),
    }
}

/// Parse a Gemini response into an internal `CompletionResponse`.
fn parse_gemini_response(response: &GeminiResponse) -> Result<CompletionResponse> {
    let candidate = response
        .candidates
        .first()
        .ok_or_else(|| SaorsaAiError::Provider {
            provider: "Google Gemini".to_string(),
            message: "response contained no candidates".to_string(),
        })?;

    let mut content_blocks: Vec<ContentBlock> = Vec::new();
    for part in &candidate.content.parts {
        match part {
            GeminiPart::Text { text } => {
                content_blocks.push(ContentBlock::Text { text: text.clone() });
            }
            GeminiPart::FunctionCall { function_call } => {
                // Generate a deterministic ID from the function name + index.
                let id = format!("call_{}", content_blocks.len());
                content_blocks.push(ContentBlock::ToolUse {
                    id,
                    name: function_call.name.clone(),
                    input: function_call.args.clone(),
                });
            }
            GeminiPart::FunctionResponse { .. } => {
                // We don't expect functionResponse in model output; skip.
            }
        }
    }

    let stop_reason = candidate.finish_reason.as_deref().map(map_finish_reason);

    let usage = response
        .usage_metadata
        .as_ref()
        .map(|u| Usage {
            input_tokens: u.prompt_token_count.unwrap_or(0),
            output_tokens: u.candidates_token_count.unwrap_or(0),
        })
        .unwrap_or_default();

    Ok(CompletionResponse {
        id: String::new(), // Gemini doesn't return a response ID
        content: content_blocks,
        model: String::new(), // Gemini doesn't echo the model name back
        stop_reason,
        usage,
    })
}

/// Map a Gemini `finishReason` string to our `StopReason`.
fn map_finish_reason(reason: &str) -> StopReason {
    match reason {
        "STOP" => StopReason::EndTurn,
        "MAX_TOKENS" => StopReason::MaxTokens,
        "STOP_SEQUENCE" => StopReason::StopSequence,
        // Gemini doesn't have a dedicated "tool_use" finish reason but we
        // infer it from the presence of function calls in the response.
        _ => StopReason::EndTurn,
    }
}

/// Parse a single SSE `data:` payload from the streaming endpoint.
fn parse_sse_event(data: &str) -> Option<StreamEvent> {
    if data == "[DONE]" {
        return Some(StreamEvent::MessageStop);
    }

    let chunk: GeminiStreamChunk = serde_json::from_str(data).ok()?;

    let candidate = chunk.candidates.as_ref().and_then(|c| c.first());

    // Check for usage metadata with no candidate content (final chunk).
    if candidate.is_none() {
        if let Some(usage) = &chunk.usage_metadata {
            return Some(StreamEvent::MessageDelta {
                stop_reason: None,
                usage: Usage {
                    input_tokens: usage.prompt_token_count.unwrap_or(0),
                    output_tokens: usage.candidates_token_count.unwrap_or(0),
                },
            });
        }
        return None;
    }

    let candidate = candidate?;

    // Check for finish reason.
    if let Some(reason) = &candidate.finish_reason {
        if let Some(usage) = &chunk.usage_metadata {
            return Some(StreamEvent::MessageDelta {
                stop_reason: Some(map_finish_reason(reason)),
                usage: Usage {
                    input_tokens: usage.prompt_token_count.unwrap_or(0),
                    output_tokens: usage.candidates_token_count.unwrap_or(0),
                },
            });
        }
        return Some(StreamEvent::MessageDelta {
            stop_reason: Some(map_finish_reason(reason)),
            usage: Usage::default(),
        });
    }

    // Extract content parts from the candidate.
    let parts = candidate
        .content
        .as_ref()
        .map(|c| c.parts.as_slice())
        .unwrap_or(&[]);

    for (i, part) in parts.iter().enumerate() {
        match part {
            GeminiPart::Text { text } => {
                return Some(StreamEvent::ContentBlockDelta {
                    index: i as u32,
                    delta: ContentDelta::TextDelta { text: text.clone() },
                });
            }
            GeminiPart::FunctionCall { function_call } => {
                return Some(StreamEvent::ContentBlockDelta {
                    index: i as u32,
                    delta: ContentDelta::InputJsonDelta {
                        partial_json: serde_json::to_string(&function_call.args)
                            .unwrap_or_default(),
                    },
                });
            }
            GeminiPart::FunctionResponse { .. } => {}
        }
    }

    None
}

/// Map an HTTP status code to the appropriate `SaorsaAiError`.
fn handle_http_error(status: reqwest::StatusCode, body: &str) -> SaorsaAiError {
    match status.as_u16() {
        401 | 403 => SaorsaAiError::Auth(format!("Gemini auth error ({}): {}", status, body)),
        429 => SaorsaAiError::RateLimit(format!("Gemini rate limit ({}): {}", status, body)),
        _ => SaorsaAiError::Provider {
            provider: "Google Gemini".to_string(),
            message: format!("HTTP {} — {}", status, body),
        },
    }
}

// ---------------------------------------------------------------------------
// Trait implementations
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl Provider for GeminiProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let model = request.model.clone();
        let gemini_req = build_gemini_request(&request);
        let body = serde_json::to_string(&gemini_req).map_err(SaorsaAiError::Json)?;

        debug!("Gemini request to {}", self.url(&model));

        let response = self
            .client
            .post(self.url(&model))
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

        let gemini_resp: GeminiResponse =
            serde_json::from_str(&response_body).map_err(|e| SaorsaAiError::Provider {
                provider: "Google Gemini".to_string(),
                message: format!("failed to parse response: {e}"),
            })?;

        parse_gemini_response(&gemini_resp)
    }
}

#[async_trait::async_trait]
impl StreamingProvider for GeminiProvider {
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamEvent>>> {
        let model = request.model.clone();
        let mut gemini_req = build_gemini_request(&request);
        // Streaming doesn't need a special flag — the URL endpoint differs.
        let _ = &mut gemini_req; // no mutation needed

        let body = serde_json::to_string(&gemini_req).map_err(SaorsaAiError::Json)?;

        debug!("Gemini stream request to {}", self.stream_url(&model));

        let response = self
            .client
            .post(self.stream_url(&model))
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

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        // Send initial MessageStart event.
        let start_event = StreamEvent::MessageStart {
            id: String::new(),
            model: model.clone(),
            usage: Usage::default(),
        };
        let _ = tx.send(Ok(start_event)).await;

        tokio::spawn(async move {
            let bytes_stream = response.bytes_stream();

            use futures::StreamExt;
            let mut reader = bytes_stream;
            let mut buffer = String::new();

            while let Some(chunk_result) = reader.next().await {
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

                // Process complete SSE lines.
                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ")
                        && let Some(event) = parse_sse_event(data)
                    {
                        let is_stop = matches!(event, StreamEvent::MessageStop);
                        if tx.send(Ok(event)).await.is_err() {
                            return;
                        }
                        if is_stop {
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
// Internal Gemini API types (serde)
// ---------------------------------------------------------------------------

/// Top-level Gemini request body.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiToolGroup>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
}

/// A content entry (one "turn" in the conversation).
#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

/// A single part within a content entry.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    /// Plain text.
    Text { text: String },
    /// A function call from the model.
    FunctionCall {
        #[serde(rename = "functionCall")]
        function_call: GeminiFunctionCall,
    },
    /// A function response to the model.
    FunctionResponse {
        #[serde(rename = "functionResponse")]
        function_response: GeminiFunctionResponse,
    },
}

/// A function call emitted by the model.
#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

/// A function response sent back to the model.
#[derive(Debug, Serialize, Deserialize)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
}

/// A group of function declarations (tools).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiToolGroup {
    function_declarations: Vec<GeminiFunctionDeclaration>,
}

/// A single function declaration.
#[derive(Debug, Serialize)]
struct GeminiFunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// Generation configuration.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

// -- Response types --

/// Top-level Gemini response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    #[serde(default)]
    usage_metadata: Option<GeminiUsageMetadata>,
}

/// A single candidate in the response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiCandidate {
    content: GeminiContent,
    #[serde(default)]
    finish_reason: Option<String>,
}

/// Token usage metadata.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiUsageMetadata {
    #[serde(default)]
    prompt_token_count: Option<u32>,
    #[serde(default)]
    candidates_token_count: Option<u32>,
}

// -- Streaming types --

/// A single chunk in the streaming response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiStreamChunk {
    #[serde(default)]
    candidates: Option<Vec<GeminiStreamCandidate>>,
    #[serde(default)]
    usage_metadata: Option<GeminiUsageMetadata>,
}

/// A candidate within a streaming chunk.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiStreamCandidate {
    #[serde(default)]
    content: Option<GeminiContent>,
    #[serde(default)]
    finish_reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{ContentBlock, Message, ToolDefinition};
    use crate::provider::ProviderConfig;
    use crate::provider::ProviderKind;

    #[test]
    fn provider_creation() {
        let config = ProviderConfig::new(ProviderKind::Gemini, "test-key", "gemini-2.0-flash");
        let provider = GeminiProvider::new(config);
        assert!(provider.is_ok());
    }

    #[test]
    fn request_serialization_basic() {
        let request =
            CompletionRequest::new("gemini-2.0-flash", vec![Message::user("Hello")], 1024);
        let gemini_req = build_gemini_request(&request);
        let json = serde_json::to_value(&gemini_req);
        assert!(json.is_ok());
        if let Ok(val) = json {
            let contents = val.get("contents");
            assert!(contents.is_some());
            let contents = contents.and_then(|c| c.as_array());
            assert!(contents.is_some_and(|a| a.len() == 1));
        }
    }

    #[test]
    fn request_serialization_with_system() {
        let request =
            CompletionRequest::new("gemini-2.0-flash", vec![Message::user("Hello")], 1024)
                .system("You are helpful");
        let gemini_req = build_gemini_request(&request);
        let json = serde_json::to_value(&gemini_req);
        assert!(json.is_ok());
        if let Ok(val) = json {
            let contents = val.get("contents").and_then(|c| c.as_array());
            // system → user + model placeholder + actual user = 3
            assert!(contents.is_some_and(|a| a.len() == 3));
        }
    }

    #[test]
    fn request_serialization_with_tools() {
        let tool = ToolDefinition::new(
            "read_file",
            "Read a file",
            serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}}}),
        );
        let request = CompletionRequest::new(
            "gemini-2.0-flash",
            vec![Message::user("Read my file")],
            1024,
        )
        .tools(vec![tool]);

        let gemini_req = build_gemini_request(&request);
        let json = serde_json::to_value(&gemini_req);
        assert!(json.is_ok());
        if let Ok(val) = json {
            let tools = val.get("tools").and_then(|t| t.as_array());
            assert!(tools.is_some_and(|a| !a.is_empty()));
        }
    }

    #[test]
    fn request_serialization_tool_use_message() {
        let msg = Message {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolUse {
                id: "call_1".to_string(),
                name: "bash".to_string(),
                input: serde_json::json!({"command": "ls"}),
            }],
        };
        let request = CompletionRequest::new("gemini-2.0-flash", vec![msg], 1024);
        let gemini_req = build_gemini_request(&request);
        let json = serde_json::to_string(&gemini_req);
        assert!(json.is_ok());
        let json_str = json.as_deref().unwrap_or("");
        assert!(json_str.contains("functionCall"));
        assert!(json_str.contains("bash"));
    }

    #[test]
    fn request_serialization_tool_result_message() {
        let msg = Message::tool_result("call_1", "file.txt contents here");
        let request = CompletionRequest::new("gemini-2.0-flash", vec![msg], 1024);
        let gemini_req = build_gemini_request(&request);
        let json = serde_json::to_string(&gemini_req);
        assert!(json.is_ok());
        let json_str = json.as_deref().unwrap_or("");
        assert!(json_str.contains("functionResponse"));
        assert!(json_str.contains("file.txt contents here"));
    }

    #[test]
    fn response_parsing_text() {
        let json = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{"text": "Hello there!"}]
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 5
            }
        }"#;
        let resp: GeminiResponse = serde_json::from_str(json).unwrap_or_else(|e| {
            panic!("Failed to parse: {e}");
        });
        let parsed = parse_gemini_response(&resp);
        assert!(parsed.is_ok());
        if let Ok(response) = parsed {
            assert_eq!(response.content.len(), 1);
            match &response.content[0] {
                ContentBlock::Text { text } => assert_eq!(text, "Hello there!"),
                _ => unreachable!("Expected Text"),
            }
            assert_eq!(response.stop_reason, Some(StopReason::EndTurn));
            assert_eq!(response.usage.input_tokens, 10);
            assert_eq!(response.usage.output_tokens, 5);
        }
    }

    #[test]
    fn response_parsing_function_call() {
        let json = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "functionCall": {
                            "name": "read_file",
                            "args": {"path": "/tmp/test.txt"}
                        }
                    }]
                },
                "finishReason": "STOP"
            }]
        }"#;
        let resp: GeminiResponse = serde_json::from_str(json).unwrap_or_else(|e| {
            panic!("Failed to parse: {e}");
        });
        let parsed = parse_gemini_response(&resp);
        assert!(parsed.is_ok());
        if let Ok(response) = parsed {
            assert_eq!(response.content.len(), 1);
            match &response.content[0] {
                ContentBlock::ToolUse { id, name, input } => {
                    assert_eq!(id, "call_0");
                    assert_eq!(name, "read_file");
                    assert_eq!(input["path"], "/tmp/test.txt");
                }
                _ => unreachable!("Expected ToolUse"),
            }
        }
    }

    #[test]
    fn response_parsing_max_tokens_finish() {
        let json = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{"text": "truncated..."}]
                },
                "finishReason": "MAX_TOKENS"
            }]
        }"#;
        let resp: GeminiResponse = serde_json::from_str(json).unwrap_or_else(|e| {
            panic!("Failed to parse: {e}");
        });
        let parsed = parse_gemini_response(&resp);
        assert!(parsed.is_ok());
        if let Ok(response) = parsed {
            assert_eq!(response.stop_reason, Some(StopReason::MaxTokens));
        }
    }

    #[test]
    fn response_empty_candidates_returns_error() {
        let json = r#"{"candidates": []}"#;
        let resp: GeminiResponse = serde_json::from_str(json).unwrap_or_else(|e| {
            panic!("Failed to parse: {e}");
        });
        let parsed = parse_gemini_response(&resp);
        assert!(parsed.is_err());
    }

    #[test]
    fn parse_sse_text_delta() {
        let data = r#"{"candidates":[{"content":{"role":"model","parts":[{"text":"Hello "}]}}]}"#;
        let event = parse_sse_event(data);
        assert!(event.is_some());
        if let Some(StreamEvent::ContentBlockDelta { index, delta }) = event {
            assert_eq!(index, 0);
            match delta {
                ContentDelta::TextDelta { text } => assert_eq!(text, "Hello "),
                _ => unreachable!("Expected TextDelta"),
            }
        } else {
            unreachable!("Expected ContentBlockDelta");
        }
    }

    #[test]
    fn parse_sse_done() {
        let event = parse_sse_event("[DONE]");
        assert!(event.is_some());
        assert!(matches!(event, Some(StreamEvent::MessageStop)));
    }

    #[test]
    fn parse_sse_finish_reason() {
        let data = r#"{"candidates":[{"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":12,"candidatesTokenCount":8}}"#;
        let event = parse_sse_event(data);
        assert!(event.is_some());
        if let Some(StreamEvent::MessageDelta { stop_reason, usage }) = event {
            assert_eq!(stop_reason, Some(StopReason::EndTurn));
            assert_eq!(usage.input_tokens, 12);
            assert_eq!(usage.output_tokens, 8);
        } else {
            unreachable!("Expected MessageDelta");
        }
    }

    #[test]
    fn parse_sse_function_call_delta() {
        let data = r#"{"candidates":[{"content":{"role":"model","parts":[{"functionCall":{"name":"bash","args":{"command":"ls"}}}]}}]}"#;
        let event = parse_sse_event(data);
        assert!(event.is_some());
        if let Some(StreamEvent::ContentBlockDelta { index, delta }) = event {
            assert_eq!(index, 0);
            match delta {
                ContentDelta::InputJsonDelta { partial_json } => {
                    assert!(partial_json.contains("command"));
                }
                _ => unreachable!("Expected InputJsonDelta"),
            }
        } else {
            unreachable!("Expected ContentBlockDelta");
        }
    }

    #[test]
    fn parse_sse_usage_only_chunk() {
        let data = r#"{"usageMetadata":{"promptTokenCount":50,"candidatesTokenCount":25}}"#;
        let event = parse_sse_event(data);
        assert!(event.is_some());
        if let Some(StreamEvent::MessageDelta { stop_reason, usage }) = event {
            assert!(stop_reason.is_none());
            assert_eq!(usage.input_tokens, 50);
            assert_eq!(usage.output_tokens, 25);
        } else {
            unreachable!("Expected MessageDelta");
        }
    }

    #[test]
    fn request_with_temperature_and_stop() {
        let request = CompletionRequest::new("gemini-2.0-flash", vec![Message::user("Hi")], 512)
            .temperature(0.8);
        let gemini_req = build_gemini_request(&request);
        let json = serde_json::to_value(&gemini_req);
        assert!(json.is_ok());
        if let Ok(val) = json {
            let config = val.get("generationConfig");
            assert!(config.is_some());
            if let Some(config) = config {
                let temp = config.get("temperature").and_then(|t| t.as_f64());
                assert!(temp.is_some_and(|t| (t - 0.8_f64).abs() < 0.001));
                assert_eq!(
                    config.get("maxOutputTokens").and_then(|t| t.as_u64()),
                    Some(512)
                );
            }
        }
    }

    #[test]
    fn url_construction() {
        let config = ProviderConfig::new(ProviderKind::Gemini, "key", "gemini-2.0-flash");
        let provider = GeminiProvider::new(config);
        assert!(provider.is_ok());
        if let Ok(p) = provider {
            assert!(p.url("gemini-2.0-flash").contains("generateContent"));
            assert!(!p.url("gemini-2.0-flash").contains("stream"));
            assert!(
                p.stream_url("gemini-2.0-flash")
                    .contains("streamGenerateContent")
            );
            assert!(p.stream_url("gemini-2.0-flash").contains("alt=sse"));
        }
    }

    #[test]
    fn map_finish_reason_variants() {
        assert_eq!(map_finish_reason("STOP"), StopReason::EndTurn);
        assert_eq!(map_finish_reason("MAX_TOKENS"), StopReason::MaxTokens);
        assert_eq!(map_finish_reason("STOP_SEQUENCE"), StopReason::StopSequence);
        assert_eq!(map_finish_reason("UNKNOWN"), StopReason::EndTurn);
    }

    #[test]
    fn gemini_content_role_mapping() {
        let user_msg = Message::user("Hello");
        let converted = convert_message(&user_msg);
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0].role, "user");

        let assistant_msg = Message::assistant("Hi");
        let converted = convert_message(&assistant_msg);
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0].role, "model");
    }
}
