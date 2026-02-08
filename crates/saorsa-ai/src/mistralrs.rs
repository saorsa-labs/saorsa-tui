//! In-process local inference provider backed by `mistralrs` (GGUF).
//!
//! This module is feature-gated behind `saorsa-ai`'s `mistralrs` feature to avoid
//! pulling in heavy `candle`/`mistralrs` dependencies by default.
//!
//! ## Model Download Location
//!
//! `mistralrs` downloads models via the Hugging Face Hub cache. By default this is:
//! - `HF_HOME/hub` if `HF_HOME` is set
//! - otherwise `~/.cache/huggingface/hub`

use std::sync::Arc;

use crate::error::{Result, SaorsaAiError};
use crate::message::{ContentBlock, Role};
use crate::provider::{Provider, StreamingProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentDelta, StopReason, StreamEvent, Usage,
};

/// Configuration for the in-process mistralrs provider.
#[derive(Clone, Copy, Debug)]
pub struct MistralrsConfig {
    /// Sampling temperature.
    pub temperature: f64,
    /// Nucleus sampling probability.
    pub top_p: f64,
}

impl Default for MistralrsConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.95,
        }
    }
}

/// Provider backed by an already-loaded `mistralrs::Model`.
pub struct MistralrsProvider {
    model: Arc<mistralrs::Model>,
    config: MistralrsConfig,
}

/// Return the default on-disk download/cache directory used by `mistralrs`.
///
/// This is the Hugging Face hub cache directory. It can be overridden by setting
/// the `HF_HOME` environment variable.
#[must_use]
pub fn default_download_dir() -> std::path::PathBuf {
    hf_hub::Cache::from_env().path().clone()
}

impl MistralrsProvider {
    /// Create a new provider using an already-loaded model.
    #[must_use]
    pub fn new(model: Arc<mistralrs::Model>, config: MistralrsConfig) -> Self {
        Self { model, config }
    }

    fn validate_request(request: &CompletionRequest) -> Result<()> {
        if !request.tools.is_empty() {
            return Err(SaorsaAiError::InvalidRequest(
                "mistralrs provider: tools are not supported (MVP)".to_string(),
            ));
        }
        if !request.stop_sequences.is_empty() {
            return Err(SaorsaAiError::InvalidRequest(
                "mistralrs provider: stop sequences are not supported (MVP)".to_string(),
            ));
        }
        if request.thinking.is_some() {
            return Err(SaorsaAiError::InvalidRequest(
                "mistralrs provider: thinking config is not supported (MVP)".to_string(),
            ));
        }
        for m in &request.messages {
            for b in &m.content {
                match b {
                    ContentBlock::Text { .. } => {}
                    ContentBlock::ToolUse { .. } | ContentBlock::ToolResult { .. } => {
                        return Err(SaorsaAiError::InvalidRequest(
                            "mistralrs provider: tool blocks are not supported (MVP)".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn build_request(
        config: MistralrsConfig,
        request: &CompletionRequest,
    ) -> Result<mistralrs::RequestBuilder> {
        // Map request into a mistralrs request builder (OpenAI-like chat request).
        let mut rb = mistralrs::RequestBuilder::new();

        rb = rb.set_sampler_max_len(request.max_tokens as usize);

        let temperature = request
            .temperature
            .map(f64::from)
            .unwrap_or(config.temperature);
        rb = rb.set_sampler_temperature(temperature);
        rb = rb.set_sampler_topp(config.top_p);

        if let Some(system) = request.system.as_ref() {
            rb = rb.add_message(mistralrs::TextMessageRole::System, system.clone());
        }

        for msg in &request.messages {
            let role = match msg.role {
                Role::User => mistralrs::TextMessageRole::User,
                Role::Assistant => mistralrs::TextMessageRole::Assistant,
            };
            let mut text = String::new();
            for b in &msg.content {
                if let ContentBlock::Text { text: t } = b {
                    text.push_str(t);
                }
            }
            rb = rb.add_message(role, text);
        }

        Ok(rb)
    }

    fn usage_from_mistralrs(u: &mistralrs::Usage) -> Usage {
        fn u32_from_usize(v: usize) -> u32 {
            u32::try_from(v).unwrap_or(u32::MAX)
        }

        Usage {
            input_tokens: u32_from_usize(u.prompt_tokens),
            output_tokens: u32_from_usize(u.completion_tokens),
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        }
    }
}

#[async_trait::async_trait]
impl Provider for MistralrsProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let mut rx = self.stream(request.clone()).await?;
        let mut out = String::new();
        let mut id = "mistralrs".to_string();
        let mut model = request.model.clone();
        let mut usage = Usage::default();
        let mut stop_reason = None;

        while let Some(ev) = rx.recv().await {
            let ev = ev?;
            match ev {
                StreamEvent::MessageStart {
                    id: i,
                    model: m,
                    usage: u,
                } => {
                    id = i;
                    model = m;
                    usage = u;
                }
                StreamEvent::ContentBlockDelta {
                    delta: ContentDelta::TextDelta { text },
                    ..
                } => {
                    out.push_str(&text);
                }
                StreamEvent::MessageDelta {
                    stop_reason: sr,
                    usage: u,
                } => {
                    stop_reason = sr;
                    usage = u;
                }
                StreamEvent::Error { message } => {
                    return Err(SaorsaAiError::Provider {
                        provider: "mistralrs".into(),
                        message,
                    });
                }
                _ => {}
            }
        }

        Ok(CompletionResponse {
            id,
            model,
            content: vec![ContentBlock::Text { text: out }],
            stop_reason,
            usage,
        })
    }
}

#[async_trait::async_trait]
impl StreamingProvider for MistralrsProvider {
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamEvent>>> {
        Self::validate_request(&request)?;

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        let model = Arc::clone(&self.model);
        let config = self.config;

        tokio::spawn(async move {
            let provider_name = "mistralrs";

            let rb = match MistralrsProvider::build_request(config, &request) {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                    return;
                }
            };

            let mut stream = match model.stream_chat_request(rb).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx
                        .send(Err(SaorsaAiError::Provider {
                            provider: provider_name.into(),
                            message: e.to_string(),
                        }))
                        .await;
                    return;
                }
            };

            let mut started = false;
            let mut block_started = false;
            let mut id: Option<String> = None;
            let mut model_name: Option<String> = None;
            let mut last_usage = Usage::default();

            while let Some(item) = stream.next().await {
                match item {
                    mistralrs::Response::Chunk(chunk) => {
                        if id.is_none() {
                            id = Some(chunk.id.clone());
                        }
                        if model_name.is_none() {
                            model_name = Some(chunk.model.clone());
                        }
                        if let Some(u) = chunk.usage.as_ref() {
                            last_usage = MistralrsProvider::usage_from_mistralrs(u);
                        }

                        if !started {
                            let ev = StreamEvent::MessageStart {
                                id: id.clone().unwrap_or_else(|| "mistralrs".to_string()),
                                model: model_name.clone().unwrap_or_else(|| request.model.clone()),
                                usage: last_usage.clone(),
                            };
                            if tx.send(Ok(ev)).await.is_err() {
                                return;
                            }
                            started = true;
                        }

                        if !block_started {
                            let ev = StreamEvent::ContentBlockStart {
                                index: 0,
                                content_block: ContentBlock::Text {
                                    text: String::new(),
                                },
                            };
                            if tx.send(Ok(ev)).await.is_err() {
                                return;
                            }
                            block_started = true;
                        }

                        for choice in &chunk.choices {
                            if let Some(content) = choice.delta.content.as_ref() {
                                let ev = StreamEvent::ContentBlockDelta {
                                    index: 0,
                                    delta: ContentDelta::TextDelta {
                                        text: content.clone(),
                                    },
                                };
                                if tx.send(Ok(ev)).await.is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    mistralrs::Response::Done(_) => {
                        break;
                    }
                    mistralrs::Response::InternalError(err)
                    | mistralrs::Response::ValidationError(err) => {
                        let _ = tx
                            .send(Err(SaorsaAiError::Provider {
                                provider: provider_name.into(),
                                message: err.to_string(),
                            }))
                            .await;
                        return;
                    }
                    mistralrs::Response::ModelError(msg, _) => {
                        let _ = tx
                            .send(Err(SaorsaAiError::Provider {
                                provider: provider_name.into(),
                                message: msg,
                            }))
                            .await;
                        return;
                    }
                    mistralrs::Response::ImageGeneration(_) => {
                        let _ = tx
                            .send(Err(SaorsaAiError::Provider {
                                provider: provider_name.into(),
                                message: "image generation responses are not supported".to_string(),
                            }))
                            .await;
                        return;
                    }
                    _other => {
                        let _ = tx
                            .send(Err(SaorsaAiError::Provider {
                                provider: provider_name.into(),
                                message: "unsupported mistralrs response variant".to_string(),
                            }))
                            .await;
                        return;
                    }
                }
            }

            // Ensure start events exist even if the model produced no chunks.
            if !started {
                let ev = StreamEvent::MessageStart {
                    id: "mistralrs".to_string(),
                    model: request.model.clone(),
                    usage: Usage::default(),
                };
                if tx.send(Ok(ev)).await.is_err() {
                    return;
                }
            }
            if !block_started {
                let ev = StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Text {
                        text: String::new(),
                    },
                };
                if tx.send(Ok(ev)).await.is_err() {
                    return;
                }
            }

            let _ = tx
                .send(Ok(StreamEvent::ContentBlockStop { index: 0 }))
                .await;
            let _ = tx
                .send(Ok(StreamEvent::MessageDelta {
                    stop_reason: Some(StopReason::EndTurn),
                    usage: last_usage,
                }))
                .await;
            let _ = tx.send(Ok(StreamEvent::MessageStop)).await;
        });

        Ok(rx)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::types::ThinkingConfig;

    #[test]
    fn validate_rejects_tools() {
        let mut req =
            CompletionRequest::new("local", vec![crate::message::Message::user("hi")], 16);
        req.tools.push(crate::message::ToolDefinition::new(
            "t",
            "d",
            serde_json::json!({"type":"object"}),
        ));
        let res = MistralrsProvider::validate_request(&req);
        assert!(matches!(res, Err(SaorsaAiError::InvalidRequest(_))));
    }

    #[test]
    fn validate_rejects_stop_sequences() {
        let mut req =
            CompletionRequest::new("local", vec![crate::message::Message::user("hi")], 16);
        req.stop_sequences.push("STOP".to_string());
        let res = MistralrsProvider::validate_request(&req);
        assert!(matches!(res, Err(SaorsaAiError::InvalidRequest(_))));
    }

    #[test]
    fn validate_rejects_thinking() {
        let req = CompletionRequest::new("local", vec![crate::message::Message::user("hi")], 16)
            .thinking(ThinkingConfig {
                enabled: true,
                budget_tokens: Some(8),
            });
        let res = MistralrsProvider::validate_request(&req);
        assert!(matches!(res, Err(SaorsaAiError::InvalidRequest(_))));
    }

    #[test]
    fn validate_rejects_tool_blocks_in_messages() {
        let req = CompletionRequest::new(
            "local",
            vec![crate::message::Message::tool_result("tool_1", "ok")],
            16,
        );
        let res = MistralrsProvider::validate_request(&req);
        assert!(matches!(res, Err(SaorsaAiError::InvalidRequest(_))));
    }

    #[test]
    fn build_request_includes_system_and_messages() {
        use mistralrs::RequestLike as _;

        let req = CompletionRequest::new(
            "local",
            vec![
                crate::message::Message::user("hi"),
                crate::message::Message::assistant("hello"),
            ],
            16,
        )
        .system("sys");

        let rb = MistralrsProvider::build_request(MistralrsConfig::default(), &req).unwrap();

        let msgs = rb.messages_ref();
        assert_eq!(msgs.len(), 3);

        let role0_owned = msgs[0].get("role").cloned().and_then(|mc| mc.left());
        let content0_owned = msgs[0].get("content").cloned().and_then(|mc| mc.left());
        let role0 = role0_owned.as_deref();
        let content0 = content0_owned.as_deref();
        assert_eq!(role0, Some("system"));
        assert_eq!(content0, Some("sys"));
    }
}
