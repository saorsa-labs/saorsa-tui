//! Core agent loop for interacting with LLM providers.
//!
//! The agent loop sends messages to the LLM, processes streaming responses,
//! executes tool calls, and continues until the model stops or a turn limit is reached.

use tracing::{debug, error, warn};

use saorsa_ai::{
    CompletionRequest, ContentBlock, ContentDelta, Message, StopReason, StreamEvent,
    StreamingProvider,
};

use crate::config::AgentConfig;
use crate::error::{Result, SaorsaAgentError};
use crate::event::{AgentEvent, EventSender, TurnEndReason};
use crate::tool::ToolRegistry;

/// The core agent loop.
pub struct AgentLoop {
    /// Provider for LLM completions.
    provider: Box<dyn StreamingProvider>,
    /// Configuration.
    config: AgentConfig,
    /// Tool registry.
    tools: ToolRegistry,
    /// Event sender for UI integration.
    event_tx: EventSender,
    /// Conversation history.
    messages: Vec<Message>,
}

impl AgentLoop {
    /// Create a new agent loop.
    pub fn new(
        provider: Box<dyn StreamingProvider>,
        config: AgentConfig,
        tools: ToolRegistry,
        event_tx: EventSender,
    ) -> Self {
        Self {
            provider,
            config,
            tools,
            event_tx,
            messages: Vec::new(),
        }
    }

    /// Add a user message and run the agent loop until completion.
    ///
    /// Returns the final assistant text response, or an error.
    pub async fn run(&mut self, user_message: &str) -> Result<String> {
        self.messages.push(Message::user(user_message));

        let mut turn = 0u32;
        let mut final_text = String::new();

        loop {
            turn += 1;

            if turn > self.config.max_turns {
                debug!(turn, max = self.config.max_turns, "Max turns reached");
                let _ = self
                    .event_tx
                    .send(AgentEvent::TurnEnd {
                        turn,
                        reason: TurnEndReason::MaxTurns,
                    })
                    .await;
                break;
            }

            let _ = self.event_tx.send(AgentEvent::TurnStart { turn }).await;

            let request = CompletionRequest::new(
                &self.config.model,
                self.messages.clone(),
                self.config.max_tokens,
            )
            .system(&self.config.system_prompt)
            .tools(self.tools.definitions());

            // Stream the response.
            let mut rx = self.provider.stream(request).await?;

            let mut text_content = String::new();
            let mut tool_calls: Vec<ToolCallInfo> = Vec::new();
            let mut stop_reason = None;

            while let Some(event) = rx.recv().await {
                match event {
                    Ok(StreamEvent::ContentBlockStart {
                        content_block: ContentBlock::ToolUse { id, name, .. },
                        ..
                    }) => {
                        tool_calls.push(ToolCallInfo {
                            id,
                            name,
                            input_json: String::new(),
                        });
                    }
                    Ok(StreamEvent::ContentBlockDelta {
                        delta: ContentDelta::TextDelta { text },
                        ..
                    }) => {
                        text_content.push_str(&text);
                        let _ = self.event_tx.send(AgentEvent::TextDelta { text }).await;
                    }
                    Ok(StreamEvent::ContentBlockDelta {
                        delta: ContentDelta::InputJsonDelta { partial_json },
                        ..
                    }) => {
                        if let Some(tc) = tool_calls.last_mut() {
                            tc.input_json.push_str(&partial_json);
                        }
                    }
                    Ok(StreamEvent::ContentBlockDelta {
                        delta: ContentDelta::ThinkingDelta { text },
                        ..
                    }) => {
                        let _ = self.event_tx.send(AgentEvent::ThinkingDelta { text }).await;
                    }
                    Ok(StreamEvent::MessageDelta {
                        stop_reason: sr, ..
                    }) => {
                        stop_reason = sr;
                    }
                    Ok(StreamEvent::Error { message }) => {
                        error!(message = %message, "Stream error");
                        let _ = self
                            .event_tx
                            .send(AgentEvent::Error {
                                message: message.clone(),
                            })
                            .await;
                        return Err(SaorsaAgentError::Internal(message));
                    }
                    _ => {}
                }
            }

            // Emit text complete event if we got text.
            if !text_content.is_empty() {
                final_text.clone_from(&text_content);
                let _ = self
                    .event_tx
                    .send(AgentEvent::TextComplete {
                        text: text_content.clone(),
                    })
                    .await;
            }

            // Build the assistant message for history.
            let mut assistant_content: Vec<ContentBlock> = Vec::new();
            if !text_content.is_empty() {
                assistant_content.push(ContentBlock::Text { text: text_content });
            }

            // Parse tool call inputs once and emit events.
            let mut parsed_inputs = Vec::with_capacity(tool_calls.len());
            for tc in &tool_calls {
                let input: serde_json::Value =
                    serde_json::from_str(&tc.input_json).unwrap_or_else(|e| {
                        warn!(
                            tool = %tc.name,
                            error = %e,
                            "Malformed tool call JSON, using empty object"
                        );
                        serde_json::Value::Object(serde_json::Map::new())
                    });

                let _ = self
                    .event_tx
                    .send(AgentEvent::ToolCall {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        input: input.clone(),
                    })
                    .await;

                assistant_content.push(ContentBlock::ToolUse {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    input: input.clone(),
                });

                parsed_inputs.push(input);
            }

            self.messages.push(Message {
                role: saorsa_ai::Role::Assistant,
                content: assistant_content,
            });

            // Handle tool calls.
            match stop_reason {
                Some(StopReason::ToolUse) if !tool_calls.is_empty() => {
                    let tool_results = self.execute_tool_calls(&tool_calls, &parsed_inputs).await;

                    for result in &tool_results {
                        self.messages
                            .push(Message::tool_result(&result.id, &result.output));
                    }

                    let _ = self
                        .event_tx
                        .send(AgentEvent::TurnEnd {
                            turn,
                            reason: TurnEndReason::ToolUse,
                        })
                        .await;

                    // Continue the loop for the next turn.
                }
                Some(StopReason::MaxTokens) => {
                    let _ = self
                        .event_tx
                        .send(AgentEvent::TurnEnd {
                            turn,
                            reason: TurnEndReason::MaxTokens,
                        })
                        .await;
                    break;
                }
                _ => {
                    // EndTurn, StopSequence, or None — we're done.
                    let _ = self
                        .event_tx
                        .send(AgentEvent::TurnEnd {
                            turn,
                            reason: TurnEndReason::EndTurn,
                        })
                        .await;
                    break;
                }
            }
        }

        Ok(final_text)
    }

    /// Execute a list of tool calls with pre-parsed inputs and return results.
    async fn execute_tool_calls(
        &self,
        tool_calls: &[ToolCallInfo],
        inputs: &[serde_json::Value],
    ) -> Vec<ToolResultInfo> {
        let mut results = Vec::new();

        for (tc, input) in tool_calls.iter().zip(inputs.iter()) {
            let (output, success) = match self.tools.get(&tc.name) {
                Some(tool) => match tool.execute(input.clone()).await {
                    Ok(result) => (result, true),
                    Err(e) => (format!("Error: {e}"), false),
                },
                None => (format!("Unknown tool: {}", tc.name), false),
            };

            let _ = self
                .event_tx
                .send(AgentEvent::ToolResult {
                    id: tc.id.clone(),
                    name: tc.name.clone(),
                    output: output.clone(),
                    success,
                })
                .await;

            results.push(ToolResultInfo {
                id: tc.id.clone(),
                output,
            });
        }

        results
    }

    /// Get the current conversation messages.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }
}

/// Internal tracking for a tool call being assembled from stream events.
#[derive(Debug)]
struct ToolCallInfo {
    /// Tool use ID.
    id: String,
    /// Tool name.
    name: String,
    /// Accumulated input JSON string.
    input_json: String,
}

/// Internal tracking for a tool result.
#[derive(Debug)]
struct ToolResultInfo {
    /// Tool use ID.
    id: String,
    /// Tool output.
    output: String,
}

/// Create a default tool registry with all built-in tools.
///
/// This includes:
/// - BashTool: Execute shell commands
/// - ReadTool: Read file contents with optional line ranges
/// - WriteTool: Write files with diff display
/// - EditTool: Surgical file editing with ambiguity detection
/// - GrepTool: Search file contents with regex
/// - FindTool: Find files by name pattern
/// - LsTool: List directory contents with metadata
/// - WebSearchTool: Search the web via DuckDuckGo (no API key required)
pub fn default_tools(working_dir: impl Into<std::path::PathBuf>) -> ToolRegistry {
    use crate::tools::{
        BashTool, EditTool, FindTool, GrepTool, LsTool, ReadTool, WebSearchTool, WriteTool,
    };
    use std::path::PathBuf;

    let wd: PathBuf = working_dir.into();
    let mut registry = ToolRegistry::new();

    registry.register(Box::new(BashTool::new(wd.clone())));
    registry.register(Box::new(ReadTool::new(wd.clone())));
    registry.register(Box::new(WriteTool::new(wd.clone())));
    registry.register(Box::new(EditTool::new(wd.clone())));
    registry.register(Box::new(GrepTool::new(wd.clone())));
    registry.register(Box::new(FindTool::new(wd.clone())));
    registry.register(Box::new(LsTool::new(wd)));
    registry.register(Box::new(WebSearchTool::new()));

    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::event_channel;

    /// Mock provider that returns a fixed response.
    struct MockProvider {
        events: Vec<StreamEvent>,
    }

    #[async_trait::async_trait]
    impl saorsa_ai::Provider for MockProvider {
        async fn complete(
            &self,
            _request: CompletionRequest,
        ) -> saorsa_ai::Result<saorsa_ai::CompletionResponse> {
            Err(saorsa_ai::SaorsaAiError::Internal("not implemented".into()))
        }
    }

    #[async_trait::async_trait]
    impl StreamingProvider for MockProvider {
        async fn stream(
            &self,
            _request: CompletionRequest,
        ) -> saorsa_ai::Result<tokio::sync::mpsc::Receiver<saorsa_ai::Result<StreamEvent>>>
        {
            let (tx, rx) = tokio::sync::mpsc::channel(64);
            let events = self.events.clone();
            tokio::spawn(async move {
                for event in events {
                    if tx.send(Ok(event)).await.is_err() {
                        break;
                    }
                }
            });
            Ok(rx)
        }
    }

    fn mock_text_provider(text: &str) -> Box<dyn StreamingProvider> {
        Box::new(MockProvider {
            events: vec![
                StreamEvent::MessageStart {
                    id: "msg_1".into(),
                    model: "test".into(),
                    usage: saorsa_ai::Usage::default(),
                },
                StreamEvent::ContentBlockStart {
                    index: 0,
                    content_block: ContentBlock::Text {
                        text: String::new(),
                    },
                },
                StreamEvent::ContentBlockDelta {
                    index: 0,
                    delta: ContentDelta::TextDelta {
                        text: text.to_string(),
                    },
                },
                StreamEvent::ContentBlockStop { index: 0 },
                StreamEvent::MessageDelta {
                    stop_reason: Some(StopReason::EndTurn),
                    usage: saorsa_ai::Usage::default(),
                },
                StreamEvent::MessageStop,
            ],
        })
    }

    #[tokio::test]
    async fn agent_simple_text_response() {
        let provider = mock_text_provider("Hello, world!");
        let config = AgentConfig::default();
        let tools = ToolRegistry::new();
        let (tx, mut rx) = event_channel(64);

        let mut agent = AgentLoop::new(provider, config, tools, tx);

        let handle = tokio::spawn(async move { agent.run("Hi").await });

        // Collect events.
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }

        let result = handle.await;
        assert!(result.is_ok());
        if let Ok(Ok(text)) = result {
            assert_eq!(text, "Hello, world!");
        }

        // Should have: TurnStart, TextDelta, TextComplete, TurnEnd.
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AgentEvent::TurnStart { turn: 1 }))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AgentEvent::TextDelta { .. }))
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AgentEvent::TextComplete { .. }))
        );
        assert!(events.iter().any(|e| matches!(
            e,
            AgentEvent::TurnEnd {
                reason: TurnEndReason::EndTurn,
                ..
            }
        )));
    }

    #[tokio::test]
    async fn agent_max_turns_limit() {
        let provider = mock_text_provider("response");
        let config = AgentConfig::default().max_turns(0);
        let tools = ToolRegistry::new();
        let (tx, _rx) = event_channel(64);

        let mut agent = AgentLoop::new(provider, config, tools, tx);
        let result = agent.run("Hi").await;
        assert!(result.is_ok());
        // With max_turns=0, it should break immediately.
    }

    #[tokio::test]
    async fn agent_tracks_messages() {
        let provider = mock_text_provider("response");
        let config = AgentConfig::default();
        let tools = ToolRegistry::new();
        let (tx, _rx) = event_channel(64);

        let mut agent = AgentLoop::new(provider, config, tools, tx);
        let _ = agent.run("Hello").await;

        let msgs = agent.messages();
        // Should have user message + assistant message.
        assert_eq!(msgs.len(), 2);
    }

    #[test]
    fn default_tools_registers_all() {
        let cwd = std::env::current_dir();
        assert!(cwd.is_ok());
        let Ok(dir) = cwd else { unreachable!() };
        let registry = super::default_tools(dir);

        // Verify all 8 tools are registered
        assert_eq!(registry.len(), 8);

        let names = registry.names();
        assert!(names.contains(&"bash"));
        assert!(names.contains(&"read"));
        assert!(names.contains(&"write"));
        assert!(names.contains(&"edit"));
        assert!(names.contains(&"grep"));
        assert!(names.contains(&"find"));
        assert!(names.contains(&"ls"));
        assert!(names.contains(&"web_search"));
    }
}
