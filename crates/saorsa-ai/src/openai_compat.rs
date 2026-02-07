//! Generic OpenAI-compatible provider.
//!
//! Wraps [`OpenAiProvider`](crate::openai::OpenAiProvider) with configurable
//! base URL, authentication, and extra headers for services that expose an
//! OpenAI-compatible Chat Completions API (Azure, Groq, Mistral, xAI,
//! OpenRouter, Cerebras, etc.).

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use crate::error::{Result, SaorsaAiError};
use crate::message::{ContentBlock, Message, Role, ToolDefinition};
use crate::provider::{Provider, ProviderConfig, ProviderKind, StreamingProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, Usage,
};

/// A provider for any OpenAI-compatible Chat Completions API.
///
/// This re-uses the OpenAI request/response format with configurable:
/// - Base URL (required)
/// - Auth format (Bearer by default, or custom header name)
/// - Extra headers (e.g., Azure `api-version`, OpenRouter `HTTP-Referer`)
/// - URL path (defaults to `/v1/chat/completions`)
pub struct OpenAiCompatProvider {
    config: ProviderConfig,
    client: reqwest::Client,
    /// Custom URL path (e.g., `/v1/chat/completions`).
    url_path: String,
    /// Custom auth header name (defaults to `Authorization` with Bearer prefix).
    /// If set, the API key is sent as-is in this header (no "Bearer " prefix).
    auth_header: Option<String>,
    /// Extra headers to include in every request.
    extra_headers: HashMap<String, String>,
}

/// Builder for configuring an `OpenAiCompatProvider`.
pub struct OpenAiCompatBuilder {
    config: ProviderConfig,
    url_path: String,
    auth_header: Option<String>,
    extra_headers: HashMap<String, String>,
}

impl OpenAiCompatBuilder {
    /// Create a new builder with the given config.
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            url_path: "/v1/chat/completions".to_string(),
            auth_header: None,
            extra_headers: HashMap::new(),
        }
    }

    /// Set a custom URL path (default: `/v1/chat/completions`).
    #[must_use]
    pub fn url_path(mut self, path: impl Into<String>) -> Self {
        self.url_path = path.into();
        self
    }

    /// Use a custom auth header name instead of `Authorization: Bearer`.
    ///
    /// The API key will be sent as-is in this header (no "Bearer " prefix).
    /// For example, Azure uses `api-key` as the header name.
    #[must_use]
    pub fn auth_header(mut self, header_name: impl Into<String>) -> Self {
        self.auth_header = Some(header_name.into());
        self
    }

    /// Add an extra header to include in every request.
    #[must_use]
    pub fn extra_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_headers.insert(name.into(), value.into());
        self
    }

    /// Build the provider.
    pub fn build(self) -> Result<OpenAiCompatProvider> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| SaorsaAiError::Network(e.to_string()))?;

        Ok(OpenAiCompatProvider {
            config: self.config,
            client,
            url_path: self.url_path,
            auth_header: self.auth_header,
            extra_headers: self.extra_headers,
        })
    }
}

impl OpenAiCompatProvider {
    /// Create a new OpenAI-compatible provider with default settings.
    ///
    /// Uses Bearer auth and `/v1/chat/completions` path.
    /// For custom auth or paths, use [`OpenAiCompatBuilder`].
    pub fn new(config: ProviderConfig) -> Result<Self> {
        OpenAiCompatBuilder::new(config).build()
    }

    /// Create a builder for fine-grained configuration.
    pub fn builder(config: ProviderConfig) -> OpenAiCompatBuilder {
        OpenAiCompatBuilder::new(config)
    }

    /// Build request headers including auth and extra headers.
    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Auth header.
        if !self.config.api_key.is_empty() {
            if let Some(custom_header) = &self.auth_header {
                let header_name = HeaderName::from_bytes(custom_header.as_bytes())
                    .map_err(|e| SaorsaAiError::Auth(format!("invalid auth header name: {e}")))?;
                let header_value = HeaderValue::from_str(&self.config.api_key)
                    .map_err(|e| SaorsaAiError::Auth(format!("invalid API key: {e}")))?;
                headers.insert(header_name, header_value);
            } else {
                let auth_value = format!("Bearer {}", self.config.api_key);
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&auth_value)
                        .map_err(|e| SaorsaAiError::Auth(format!("invalid API key: {e}")))?,
                );
            }
        }

        // Extra headers.
        for (name, value) in &self.extra_headers {
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|e| SaorsaAiError::Auth(format!("invalid header name '{name}': {e}")))?;
            let header_value = HeaderValue::from_str(value).map_err(|e| {
                SaorsaAiError::Auth(format!("invalid header value for '{name}': {e}"))
            })?;
            headers.insert(header_name, header_value);
        }

        Ok(headers)
    }

    /// Build the API URL.
    fn url(&self) -> String {
        format!("{}{}", self.config.base_url, self.url_path)
    }
}

// ---------------------------------------------------------------------------
// Request/Response — reuses OpenAI format
// ---------------------------------------------------------------------------

// We duplicate the OpenAI types here rather than making them pub(crate),
// because they're simple serde structs and this avoids coupling modules.
// The conversion logic is identical to OpenAiProvider.

/// Build an OpenAI-compatible request body.
fn build_compat_request(request: &CompletionRequest, stream: bool) -> CompatRequest {
    let mut messages = Vec::new();

    if let Some(system) = &request.system {
        messages.push(CompatMessage {
            role: "system".to_string(),
            content: Some(system.clone()),
            tool_calls: None,
            tool_call_id: None,
        });
    }

    for msg in &request.messages {
        let converted = convert_message(msg);
        messages.extend(converted);
    }

    let tools = if request.tools.is_empty() {
        None
    } else {
        Some(request.tools.iter().map(convert_tool_definition).collect())
    };

    let stop = if request.stop_sequences.is_empty() {
        None
    } else {
        Some(request.stop_sequences.clone())
    };

    CompatRequest {
        model: request.model.clone(),
        messages,
        max_tokens: Some(request.max_tokens),
        temperature: request.temperature,
        tools,
        stream,
        stop,
    }
}

/// Convert an internal `Message` to OpenAI-compatible messages.
fn convert_message(msg: &Message) -> Vec<CompatMessage> {
    let role_str = match msg.role {
        Role::User => "user",
        Role::Assistant => "assistant",
    };

    // Tool results → separate "tool" role messages.
    let has_tool_results = msg
        .content
        .iter()
        .any(|b| matches!(b, ContentBlock::ToolResult { .. }));

    if has_tool_results {
        return msg
            .content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::ToolResult {
                    tool_use_id,
                    content,
                } => Some(CompatMessage {
                    role: "tool".to_string(),
                    content: Some(content.clone()),
                    tool_calls: None,
                    tool_call_id: Some(tool_use_id.clone()),
                }),
                _ => None,
            })
            .collect();
    }

    // Assistant messages with tool calls.
    let has_tool_use = msg
        .content
        .iter()
        .any(|b| matches!(b, ContentBlock::ToolUse { .. }));

    if has_tool_use {
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

        let tool_calls: Vec<CompatToolCall> = msg
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::ToolUse { id, name, input } => Some(CompatToolCall {
                    id: id.clone(),
                    call_type: "function".to_string(),
                    function: CompatFunctionCall {
                        name: name.clone(),
                        arguments: input.to_string(),
                    },
                }),
                _ => None,
            })
            .collect();

        return vec![CompatMessage {
            role: role_str.to_string(),
            content: text_content,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
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

    vec![CompatMessage {
        role: role_str.to_string(),
        content: Some(content),
        tool_calls: None,
        tool_call_id: None,
    }]
}

/// Convert a `ToolDefinition` to OpenAI-compatible format.
fn convert_tool_definition(tool: &ToolDefinition) -> CompatTool {
    CompatTool {
        tool_type: "function".to_string(),
        function: CompatFunction {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: tool.input_schema.clone(),
        },
    }
}

/// Parse a response into a `CompletionResponse`.
fn parse_compat_response(resp: CompatResponse) -> Result<CompletionResponse> {
    let choice = resp
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| SaorsaAiError::Provider {
            provider: "OpenAI-Compatible".into(),
            message: "response contained no choices".into(),
        })?;

    let mut content = Vec::new();

    if let Some(text) = choice.message.content
        && !text.is_empty()
    {
        content.push(ContentBlock::Text { text });
    }

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
        input_tokens: resp.usage.prompt_tokens,
        output_tokens: resp.usage.completion_tokens,
    };

    Ok(CompletionResponse {
        id: resp.id,
        content,
        model: resp.model,
        stop_reason,
        usage,
    })
}

/// Parse an SSE event from a streaming response.
fn parse_sse_event(data: &str) -> Option<StreamEvent> {
    if data == "[DONE]" {
        return Some(StreamEvent::MessageStop);
    }

    let chunk: std::result::Result<CompatStreamChunk, _> = serde_json::from_str(data);
    let chunk = chunk.ok()?;

    let choice = chunk.choices.into_iter().next()?;

    if let Some(content) = choice.delta.content
        && !content.is_empty()
    {
        return Some(StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentDelta::TextDelta { text: content },
        });
    }

    if let Some(tool_calls) = choice.delta.tool_calls
        && let Some(tc) = tool_calls.first()
        && let Some(function) = &tc.function
        && let Some(args) = &function.arguments
    {
        return Some(StreamEvent::ContentBlockDelta {
            index: tc.index.unwrap_or(0),
            delta: ContentDelta::InputJsonDelta {
                partial_json: args.clone(),
            },
        });
    }

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

/// Map an HTTP status code to the appropriate `SaorsaAiError`.
fn handle_http_error(status: reqwest::StatusCode, body: &str) -> SaorsaAiError {
    match status.as_u16() {
        401 | 403 => SaorsaAiError::Auth(format!(
            "OpenAI-Compatible auth error ({}): {}",
            status, body
        )),
        429 => SaorsaAiError::RateLimit(format!(
            "OpenAI-Compatible rate limit ({}): {}",
            status, body
        )),
        _ => SaorsaAiError::Provider {
            provider: "OpenAI-Compatible".to_string(),
            message: format!("HTTP {} — {}", status, body),
        },
    }
}

// ---------------------------------------------------------------------------
// Trait implementations
// ---------------------------------------------------------------------------

#[async_trait::async_trait]
impl Provider for OpenAiCompatProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let compat_req = build_compat_request(&request, false);
        let body = serde_json::to_string(&compat_req).map_err(SaorsaAiError::Json)?;

        debug!("OpenAI-Compatible request to {}", self.url());

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

        let compat_resp: CompatResponse =
            serde_json::from_str(&response_body).map_err(|e| SaorsaAiError::Provider {
                provider: "OpenAI-Compatible".to_string(),
                message: format!("failed to parse response: {e}"),
            })?;

        parse_compat_response(compat_resp)
    }
}

#[async_trait::async_trait]
impl StreamingProvider for OpenAiCompatProvider {
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamEvent>>> {
        let compat_req = build_compat_request(&request, true);
        let body = serde_json::to_string(&compat_req).map_err(SaorsaAiError::Json)?;

        debug!("OpenAI-Compatible stream request to {}", self.url());

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

                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_string();
                    buffer = buffer[pos + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ")
                        && let Some(event) = parse_sse_event(data)
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

// ---------------------------------------------------------------------------
// Pre-configured factory functions
// ---------------------------------------------------------------------------

/// Create an Azure OpenAI provider.
///
/// Azure uses a custom URL pattern and `api-key` header instead of Bearer.
pub fn azure_openai(
    api_key: impl Into<String>,
    endpoint: impl Into<String>,
    deployment: impl Into<String>,
    api_version: impl Into<String>,
) -> Result<OpenAiCompatProvider> {
    let deployment = deployment.into();
    let api_version = api_version.into();
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, api_key, &deployment)
        .with_base_url(endpoint);
    OpenAiCompatBuilder::new(config)
        .url_path(format!(
            "/openai/deployments/{deployment}/chat/completions?api-version={api_version}"
        ))
        .auth_header("api-key")
        .build()
}

/// Create a Groq provider.
pub fn groq(api_key: impl Into<String>, model: impl Into<String>) -> Result<OpenAiCompatProvider> {
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, api_key, model)
        .with_base_url("https://api.groq.com/openai");
    OpenAiCompatBuilder::new(config).build()
}

/// Create a Mistral provider.
pub fn mistral(
    api_key: impl Into<String>,
    model: impl Into<String>,
) -> Result<OpenAiCompatProvider> {
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, api_key, model)
        .with_base_url("https://api.mistral.ai");
    OpenAiCompatBuilder::new(config).build()
}

/// Create an OpenRouter provider.
pub fn openrouter(
    api_key: impl Into<String>,
    model: impl Into<String>,
) -> Result<OpenAiCompatProvider> {
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, api_key, model)
        .with_base_url("https://openrouter.ai/api");
    OpenAiCompatBuilder::new(config).build()
}

/// Create an xAI (Grok) provider.
pub fn xai(api_key: impl Into<String>, model: impl Into<String>) -> Result<OpenAiCompatProvider> {
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, api_key, model)
        .with_base_url("https://api.x.ai");
    OpenAiCompatBuilder::new(config).build()
}

/// Create a Cerebras provider.
pub fn cerebras(
    api_key: impl Into<String>,
    model: impl Into<String>,
) -> Result<OpenAiCompatProvider> {
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, api_key, model)
        .with_base_url("https://api.cerebras.ai");
    OpenAiCompatBuilder::new(config).build()
}

// ---------------------------------------------------------------------------
// Internal types (OpenAI-compatible format)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct CompatRequest {
    model: String,
    messages: Vec<CompatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<CompatTool>>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Serialize)]
struct CompatMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<CompatToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CompatToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: CompatFunctionCall,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CompatFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Serialize)]
struct CompatTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: CompatFunction,
}

#[derive(Serialize)]
struct CompatFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Deserialize)]
struct CompatResponse {
    id: String,
    model: String,
    choices: Vec<CompatChoice>,
    usage: CompatUsage,
}

#[derive(Deserialize)]
struct CompatChoice {
    message: CompatResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct CompatResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<CompatToolCall>>,
}

#[derive(Deserialize)]
struct CompatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Deserialize)]
struct CompatStreamChunk {
    choices: Vec<CompatStreamChoice>,
    usage: Option<CompatStreamUsage>,
}

#[derive(Deserialize)]
struct CompatStreamChoice {
    delta: CompatStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct CompatStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<CompatStreamToolCall>>,
}

#[derive(Deserialize)]
struct CompatStreamToolCall {
    index: Option<u32>,
    function: Option<CompatStreamFunctionDelta>,
}

#[derive(Deserialize)]
struct CompatStreamFunctionDelta {
    arguments: Option<String>,
}

#[derive(Deserialize)]
struct CompatStreamUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_creation() {
        let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "key", "model")
            .with_base_url("https://api.example.com");
        let result = OpenAiCompatProvider::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn url_construction_default_path() {
        let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "key", "model")
            .with_base_url("https://api.example.com");
        if let Ok(provider) = OpenAiCompatProvider::new(config) {
            assert_eq!(
                provider.url(),
                "https://api.example.com/v1/chat/completions"
            );
        }
    }

    #[test]
    fn url_construction_custom_path() {
        let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "key", "model")
            .with_base_url("https://api.example.com");
        let result = OpenAiCompatBuilder::new(config)
            .url_path("/custom/endpoint")
            .build();
        assert!(result.is_ok());
        if let Ok(provider) = result {
            assert_eq!(provider.url(), "https://api.example.com/custom/endpoint");
        }
    }

    #[test]
    fn headers_bearer_auth() {
        let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "my-key", "model")
            .with_base_url("https://api.example.com");
        if let Ok(provider) = OpenAiCompatProvider::new(config) {
            let headers = provider.headers();
            assert!(headers.is_ok());
            if let Ok(h) = headers {
                let auth = h.get(AUTHORIZATION);
                assert!(auth.is_some());
                if let Some(val) = auth {
                    assert_eq!(val.to_str().unwrap_or(""), "Bearer my-key");
                }
            }
        }
    }

    #[test]
    fn headers_custom_auth() {
        let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "azure-key", "model")
            .with_base_url("https://myresource.openai.azure.com");
        let result = OpenAiCompatBuilder::new(config)
            .auth_header("api-key")
            .build();
        assert!(result.is_ok());
        if let Ok(provider) = result {
            let headers = provider.headers();
            assert!(headers.is_ok());
            if let Ok(h) = headers {
                // Should NOT have Authorization header.
                assert!(h.get(AUTHORIZATION).is_none());
                // Should have api-key header.
                let api_key = h.get("api-key");
                assert!(api_key.is_some());
                if let Some(val) = api_key {
                    assert_eq!(val.to_str().unwrap_or(""), "azure-key");
                }
            }
        }
    }

    #[test]
    fn headers_extra_headers() {
        let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "key", "model")
            .with_base_url("https://openrouter.ai/api");
        let result = OpenAiCompatBuilder::new(config)
            .extra_header("HTTP-Referer", "https://myapp.com")
            .extra_header("X-Title", "My App")
            .build();
        assert!(result.is_ok());
        if let Ok(provider) = result {
            let headers = provider.headers();
            assert!(headers.is_ok());
            if let Ok(h) = headers {
                let referer = h.get("HTTP-Referer");
                assert!(referer.is_some());
                let title = h.get("X-Title");
                assert!(title.is_some());
            }
        }
    }

    #[test]
    fn headers_empty_api_key_no_auth() {
        let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "", "model")
            .with_base_url("https://api.example.com");
        if let Ok(provider) = OpenAiCompatProvider::new(config) {
            let headers = provider.headers();
            assert!(headers.is_ok());
            if let Ok(h) = headers {
                assert!(h.get(AUTHORIZATION).is_none());
            }
        }
    }

    #[test]
    fn request_serialization_basic() {
        let request = CompletionRequest::new("gpt-4o", vec![Message::user("Hello")], 1024);
        let compat_req = build_compat_request(&request, false);
        let json = serde_json::to_value(&compat_req);
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
            CompletionRequest::new("model", vec![Message::user("Hi")], 512).system("Be helpful");
        let compat_req = build_compat_request(&request, false);
        let json = serde_json::to_value(&compat_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
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
                "properties": {"command": {"type": "string"}}
            }),
        );
        let request =
            CompletionRequest::new("model", vec![Message::user("Hi")], 1024).tools(vec![tool]);
        let compat_req = build_compat_request(&request, false);
        let json = serde_json::to_value(&compat_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
            let tools = &v["tools"];
            assert!(tools.is_array());
            if let Some(arr) = tools.as_array() {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr[0]["type"], "function");
                assert_eq!(arr[0]["function"]["name"], "bash");
            }
        }
    }

    #[test]
    fn request_serialization_tool_result() {
        let msg = Message::tool_result("call_0", "result text");
        let request = CompletionRequest::new("model", vec![msg], 1024);
        let compat_req = build_compat_request(&request, false);
        let json = serde_json::to_value(&compat_req);
        assert!(json.is_ok());
        if let Ok(v) = json {
            assert_eq!(v["messages"][0]["role"], "tool");
            assert_eq!(v["messages"][0]["content"], "result text");
            assert_eq!(v["messages"][0]["tool_call_id"], "call_0");
        }
    }

    #[test]
    fn response_parsing_text() {
        let json = r#"{
            "id": "chatcmpl-123",
            "model": "gpt-4o",
            "choices": [{
                "message": {"content": "Hello!", "role": "assistant"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5}
        }"#;
        let resp: std::result::Result<CompatResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());
        if let Ok(resp) = resp {
            let parsed = parse_compat_response(resp);
            assert!(parsed.is_ok());
            if let Ok(p) = parsed {
                assert_eq!(p.id, "chatcmpl-123");
                assert_eq!(p.model, "gpt-4o");
                assert_eq!(p.stop_reason, Some(StopReason::EndTurn));
                assert_eq!(p.usage.input_tokens, 10);
                assert_eq!(p.usage.output_tokens, 5);
                assert_eq!(p.content.len(), 1);
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
                    "content": null,
                    "role": "assistant",
                    "tool_calls": [{
                        "id": "call_abc",
                        "type": "function",
                        "function": {"name": "bash", "arguments": "{\"command\":\"ls\"}"}
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"prompt_tokens": 20, "completion_tokens": 10}
        }"#;
        let resp: std::result::Result<CompatResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());
        if let Ok(resp) = resp {
            let parsed = parse_compat_response(resp);
            assert!(parsed.is_ok());
            if let Ok(p) = parsed {
                assert_eq!(p.stop_reason, Some(StopReason::ToolUse));
                assert_eq!(p.content.len(), 1);
                match &p.content[0] {
                    ContentBlock::ToolUse { id, name, input } => {
                        assert_eq!(id, "call_abc");
                        assert_eq!(name, "bash");
                        assert_eq!(input["command"], "ls");
                    }
                    _ => unreachable!("Expected ToolUse"),
                }
            }
        }
    }

    #[test]
    fn parse_sse_text_delta() {
        let data = r#"{"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let event = parse_sse_event(data);
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
    fn parse_sse_done() {
        let event = parse_sse_event("[DONE]");
        assert!(event.is_some());
        assert!(matches!(event, Some(StreamEvent::MessageStop)));
    }

    #[test]
    fn parse_sse_finish_reason() {
        let data = r#"{"choices":[{"delta":{},"finish_reason":"stop"}]}"#;
        let event = parse_sse_event(data);
        assert!(event.is_some());
        if let Some(StreamEvent::MessageDelta { stop_reason, .. }) = event {
            assert_eq!(stop_reason, Some(StopReason::EndTurn));
        } else {
            unreachable!("Expected MessageDelta");
        }
    }

    #[test]
    fn factory_groq() {
        let result = groq("key", "llama-3.3-70b-versatile");
        assert!(result.is_ok());
        if let Ok(provider) = result {
            assert_eq!(
                provider.url(),
                "https://api.groq.com/openai/v1/chat/completions"
            );
        }
    }

    #[test]
    fn factory_mistral() {
        let result = mistral("key", "mistral-large-latest");
        assert!(result.is_ok());
        if let Ok(provider) = result {
            assert_eq!(provider.url(), "https://api.mistral.ai/v1/chat/completions");
        }
    }

    #[test]
    fn factory_openrouter() {
        let result = openrouter("key", "anthropic/claude-3-opus");
        assert!(result.is_ok());
        if let Ok(provider) = result {
            assert_eq!(
                provider.url(),
                "https://openrouter.ai/api/v1/chat/completions"
            );
        }
    }

    #[test]
    fn factory_xai() {
        let result = xai("key", "grok-2");
        assert!(result.is_ok());
        if let Ok(provider) = result {
            assert_eq!(provider.url(), "https://api.x.ai/v1/chat/completions");
        }
    }

    #[test]
    fn factory_cerebras() {
        let result = cerebras("key", "llama3.1-8b");
        assert!(result.is_ok());
        if let Ok(provider) = result {
            assert_eq!(
                provider.url(),
                "https://api.cerebras.ai/v1/chat/completions"
            );
        }
    }

    #[test]
    fn factory_azure() {
        let result = azure_openai(
            "azure-key",
            "https://myresource.openai.azure.com",
            "gpt-4o",
            "2024-02-15-preview",
        );
        assert!(result.is_ok());
        if let Ok(provider) = result {
            assert_eq!(
                provider.url(),
                "https://myresource.openai.azure.com/openai/deployments/gpt-4o/chat/completions?api-version=2024-02-15-preview"
            );
            // Verify Azure uses api-key header.
            assert_eq!(provider.auth_header, Some("api-key".to_string()));
        }
    }

    #[test]
    fn stream_flag_set() {
        let request = CompletionRequest::new("model", vec![Message::user("Hi")], 1024);
        let non_stream = build_compat_request(&request, false);
        assert!(!non_stream.stream);
        let stream = build_compat_request(&request, true);
        assert!(stream.stream);
    }
}
