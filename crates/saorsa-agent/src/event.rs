//! Agent events for UI integration.
//!
//! The agent emits events as it processes turns, allowing the UI
//! to display streaming text, tool calls, and status updates.

/// An event emitted by the agent during execution.
#[derive(Clone, Debug)]
pub enum AgentEvent {
    /// A new agent turn has started.
    TurnStart {
        /// The turn number (1-indexed).
        turn: u32,
    },

    /// Streaming text delta from the assistant.
    TextDelta {
        /// The incremental text content.
        text: String,
    },

    /// The assistant is requesting a tool call.
    ToolCall {
        /// The tool use ID.
        id: String,
        /// The tool name.
        name: String,
        /// The tool input as JSON.
        input: serde_json::Value,
    },

    /// A tool has returned a result.
    ToolResult {
        /// The tool use ID this result corresponds to.
        id: String,
        /// The tool name.
        name: String,
        /// The tool output.
        output: String,
        /// Whether the tool succeeded.
        success: bool,
    },

    /// The assistant's text response is complete for this turn.
    TextComplete {
        /// The full text of the assistant's response.
        text: String,
    },

    /// A turn has ended.
    TurnEnd {
        /// The turn number.
        turn: u32,
        /// Why the turn ended.
        reason: TurnEndReason,
    },

    /// An error occurred during agent execution.
    Error {
        /// The error message.
        message: String,
    },
}

/// Why an agent turn ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnEndReason {
    /// The model finished responding naturally.
    EndTurn,
    /// The model wants to use a tool (another turn will follow).
    ToolUse,
    /// The maximum turn limit was reached.
    MaxTurns,
    /// The model hit the max_tokens limit.
    MaxTokens,
    /// An error occurred.
    Error,
}

/// Sender for agent events.
pub type EventSender = tokio::sync::mpsc::Sender<AgentEvent>;

/// Receiver for agent events.
pub type EventReceiver = tokio::sync::mpsc::Receiver<AgentEvent>;

/// Create a new event channel with the given buffer size.
pub fn event_channel(buffer: usize) -> (EventSender, EventReceiver) {
    tokio::sync::mpsc::channel(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_end_reason_equality() {
        assert_eq!(TurnEndReason::EndTurn, TurnEndReason::EndTurn);
        assert_ne!(TurnEndReason::EndTurn, TurnEndReason::ToolUse);
    }

    #[tokio::test]
    async fn event_channel_send_receive() {
        let (tx, mut rx) = event_channel(8);
        let send_result = tx.send(AgentEvent::TurnStart { turn: 1 }).await;
        assert!(send_result.is_ok());

        let event = rx.recv().await;
        match event {
            Some(AgentEvent::TurnStart { turn }) => {
                assert_eq!(turn, 1);
            }
            _ => panic!("Expected TurnStart event"),
        }
    }

    #[tokio::test]
    async fn event_channel_text_delta() {
        let (tx, mut rx) = event_channel(8);
        let _ = tx
            .send(AgentEvent::TextDelta {
                text: "Hello".into(),
            })
            .await;

        match rx.recv().await {
            Some(AgentEvent::TextDelta { text }) => {
                assert_eq!(text, "Hello");
            }
            _ => panic!("Expected TextDelta event"),
        }
    }

    #[tokio::test]
    async fn event_channel_tool_result() {
        let (tx, mut rx) = event_channel(8);
        let _ = tx
            .send(AgentEvent::ToolResult {
                id: "tool_1".into(),
                name: "bash".into(),
                output: "done".into(),
                success: true,
            })
            .await;

        match rx.recv().await {
            Some(AgentEvent::ToolResult {
                id,
                name,
                output,
                success,
            }) => {
                assert_eq!(id, "tool_1");
                assert_eq!(name, "bash");
                assert_eq!(output, "done");
                assert!(success);
            }
            _ => panic!("Expected ToolResult event"),
        }
    }

    #[test]
    fn agent_event_debug() {
        let event = AgentEvent::Error {
            message: "test error".into(),
        };
        let debug_str = format!("{event:?}");
        assert!(debug_str.contains("test error"));
    }
}
