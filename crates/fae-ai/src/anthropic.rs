//! Anthropic Messages API provider.

use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Deserialize;
use tracing::debug;

use crate::error::{FaeAiError, Result};
use crate::message::ContentBlock;
use crate::provider::{Provider, ProviderConfig, StreamingProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, Usage,
};

/// Anthropic-specific API version header.
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Messages API provider.
pub struct AnthropicProvider {
    config: ProviderConfig,
    client: reqwest::Client,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider with the given configuration.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(|e| FaeAiError::Network(e.to_string()))?;
        Ok(Self { config, client })
    }

    /// Build the request headers.
    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.config.api_key)
                .map_err(|e| FaeAiError::Auth(format!("invalid API key: {e}")))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(ANTHROPIC_VERSION),
        );
        Ok(headers)
    }

    /// Build the API URL.
    fn url(&self) -> String {
        format!("{}/v1/messages", self.config.base_url)
    }

    /// Parse an SSE event line into a StreamEvent.
    pub fn parse_sse_event(event_type: &str, data: &str) -> Option<StreamEvent> {
        match event_type {
            "message_start" => {
                let parsed: std::result::Result<SseMessageStart, _> = serde_json::from_str(data);
                parsed
                    .ok()
                    .map(|m| StreamEvent::MessageStart {
                        id: m.message.id,
                        model: m.message.model,
                        usage: m.message.usage,
                    })
            }
            "content_block_start" => {
                let parsed: std::result::Result<SseContentBlockStart, _> =
                    serde_json::from_str(data);
                parsed.ok().map(|c| StreamEvent::ContentBlockStart {
                    index: c.index,
                    content_block: c.content_block,
                })
            }
            "content_block_delta" => {
                let parsed: std::result::Result<SseContentBlockDelta, _> =
                    serde_json::from_str(data);
                parsed.ok().map(|c| StreamEvent::ContentBlockDelta {
                    index: c.index,
                    delta: c.delta,
                })
            }
            "content_block_stop" => {
                let parsed: std::result::Result<SseContentBlockStop, _> =
                    serde_json::from_str(data);
                parsed
                    .ok()
                    .map(|c| StreamEvent::ContentBlockStop { index: c.index })
            }
            "message_delta" => {
                let parsed: std::result::Result<SseMessageDelta, _> = serde_json::from_str(data);
                parsed.ok().map(|m| StreamEvent::MessageDelta {
                    stop_reason: m.delta.stop_reason,
                    usage: m.usage,
                })
            }
            "message_stop" => Some(StreamEvent::MessageStop),
            "ping" => Some(StreamEvent::Ping),
            "error" => {
                let parsed: std::result::Result<SseError, _> = serde_json::from_str(data);
                parsed.ok().map(|e| StreamEvent::Error {
                    message: e.error.message,
                })
            }
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl Provider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let headers = self.headers()?;
        let url = self.url();

        debug!(model = %request.model, "Sending completion request");

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| FaeAiError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".into());
            return match status.as_u16() {
                401 => Err(FaeAiError::Auth(body)),
                429 => Err(FaeAiError::RateLimit(body)),
                _ => Err(FaeAiError::Provider {
                    provider: "anthropic".into(),
                    message: format!("HTTP {status}: {body}"),
                }),
            };
        }

        let resp: CompletionResponse = response
            .json()
            .await
            .map_err(|e| FaeAiError::Provider {
                provider: "anthropic".into(),
                message: format!("response parse error: {e}"),
            })?;

        Ok(resp)
    }
}

#[async_trait::async_trait]
impl StreamingProvider for AnthropicProvider {
    async fn stream(
        &self,
        mut request: CompletionRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamEvent>>> {
        request.stream = true;
        let headers = self.headers()?;
        let url = self.url();

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&request)
            .send()
            .await
            .map_err(|e| FaeAiError::Network(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".into());
            return match status.as_u16() {
                401 => Err(FaeAiError::Auth(body)),
                429 => Err(FaeAiError::RateLimit(body)),
                _ => Err(FaeAiError::Provider {
                    provider: "anthropic".into(),
                    message: format!("HTTP {status}: {body}"),
                }),
            };
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut event_type = String::new();

            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx
                            .send(Err(FaeAiError::Streaming(e.to_string())))
                            .await;
                        break;
                    }
                };

                let text = String::from_utf8_lossy(&chunk);
                buffer.push_str(&text);

                // Parse SSE lines from buffer
                while let Some(pos) = buffer.find("\n\n") {
                    let event_text = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();

                    for line in event_text.lines() {
                        if let Some(et) = line.strip_prefix("event: ") {
                            event_type = et.to_string();
                        } else if let Some(data) = line.strip_prefix("data: ")
                            && let Some(event) =
                                AnthropicProvider::parse_sse_event(&event_type, data)
                            && tx.send(Ok(event)).await.is_err()
                        {
                            return;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}

// Internal SSE event structures for deserialization

#[derive(Deserialize)]
struct SseMessageStart {
    message: SseMessageInfo,
}

#[derive(Deserialize)]
struct SseMessageInfo {
    id: String,
    model: String,
    usage: Usage,
}

#[derive(Deserialize)]
struct SseContentBlockStart {
    index: u32,
    content_block: ContentBlock,
}

#[derive(Deserialize)]
struct SseContentBlockDelta {
    index: u32,
    delta: ContentDelta,
}

#[derive(Deserialize)]
struct SseContentBlockStop {
    index: u32,
}

#[derive(Deserialize)]
struct SseMessageDelta {
    delta: SseMessageDeltaInner,
    usage: Usage,
}

#[derive(Deserialize)]
struct SseMessageDeltaInner {
    stop_reason: Option<StopReason>,
}

#[derive(Deserialize)]
struct SseError {
    error: SseErrorInner,
}

#[derive(Deserialize)]
struct SseErrorInner {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message_start() {
        let data = r#"{"type":"message_start","message":{"id":"msg_1","type":"message","role":"assistant","content":[],"model":"claude-sonnet-4-5-20250929","stop_reason":null,"usage":{"input_tokens":10,"output_tokens":0}}}"#;
        let event = AnthropicProvider::parse_sse_event("message_start", data);
        assert!(event.is_some());
        if let Some(StreamEvent::MessageStart { id, model, usage }) = event {
            assert_eq!(id, "msg_1");
            assert_eq!(model, "claude-sonnet-4-5-20250929");
            assert_eq!(usage.input_tokens, 10);
        } else {
            panic!("Expected MessageStart");
        }
    }

    #[test]
    fn parse_content_block_delta() {
        let data =
            r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
        let event = AnthropicProvider::parse_sse_event("content_block_delta", data);
        assert!(event.is_some());
        if let Some(StreamEvent::ContentBlockDelta { index, delta }) = event {
            assert_eq!(index, 0);
            if let ContentDelta::TextDelta { text } = delta {
                assert_eq!(text, "Hello");
            } else {
                panic!("Expected TextDelta");
            }
        } else {
            panic!("Expected ContentBlockDelta");
        }
    }

    #[test]
    fn parse_message_stop() {
        let event = AnthropicProvider::parse_sse_event("message_stop", "{}");
        assert!(matches!(event, Some(StreamEvent::MessageStop)));
    }

    #[test]
    fn parse_ping() {
        let event = AnthropicProvider::parse_sse_event("ping", "{}");
        assert!(matches!(event, Some(StreamEvent::Ping)));
    }

    #[test]
    fn parse_error() {
        let data = r#"{"type":"error","error":{"type":"rate_limit_error","message":"Rate limited"}}"#;
        let event = AnthropicProvider::parse_sse_event("error", data);
        assert!(event.is_some());
        if let Some(StreamEvent::Error { message }) = event {
            assert_eq!(message, "Rate limited");
        } else {
            panic!("Expected Error event");
        }
    }

    #[test]
    fn parse_message_delta() {
        let data = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":15}}"#;
        let event = AnthropicProvider::parse_sse_event("message_delta", data);
        assert!(event.is_some());
        if let Some(StreamEvent::MessageDelta {
            stop_reason,
            usage,
        }) = event
        {
            assert_eq!(stop_reason, Some(StopReason::EndTurn));
            assert_eq!(usage.output_tokens, 15);
        } else {
            panic!("Expected MessageDelta");
        }
    }

    #[test]
    fn parse_unknown_event_returns_none() {
        let event = AnthropicProvider::parse_sse_event("unknown_event", "{}");
        assert!(event.is_none());
    }

    #[test]
    fn provider_creation() {
        let config = ProviderConfig::new("sk-test", "claude-sonnet-4-5-20250929");
        let provider = AnthropicProvider::new(config);
        assert!(provider.is_ok());
    }
}
