//! Integration tests for saorsa-ai provider layer.
//!
//! Tests message construction, response parsing, streaming, and tool calls
//! without making actual network requests.

#![allow(clippy::unwrap_used)]

use saorsa_ai::{
    CompletionRequest, CompletionResponse, ContentBlock, ContentDelta, Message, Role, StopReason,
    ToolDefinition, Usage,
};

#[test]
fn message_construction_user() {
    let msg = Message::user("Hello, world!");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content.len(), 1);

    match &msg.content[0] {
        ContentBlock::Text { text } => {
            assert_eq!(text, "Hello, world!");
        }
        _ => unreachable!(),
    }
}

#[test]
fn message_construction_assistant() {
    let msg = Message::assistant("I can help you.");
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.content.len(), 1);

    match &msg.content[0] {
        ContentBlock::Text { text } => {
            assert_eq!(text, "I can help you.");
        }
        _ => unreachable!(),
    }
}

#[test]
fn message_construction_tool_result() {
    let msg = Message::tool_result("tool_123", "File contents here");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.content.len(), 1);

    match &msg.content[0] {
        ContentBlock::ToolResult {
            tool_use_id,
            content,
        } => {
            assert_eq!(tool_use_id, "tool_123");
            assert_eq!(content, "File contents here");
        }
        _ => unreachable!(),
    }
}

#[test]
fn request_serialization() {
    let req = CompletionRequest::new(
        "claude-sonnet-4-5-20250929",
        vec![Message::user("Test message")],
        1024,
    )
    .system("You are a helpful assistant")
    .temperature(0.7);

    let json = serde_json::to_string(&req).unwrap();

    assert!(json.contains("claude-sonnet-4-5-20250929"));
    assert!(json.contains("1024"));
    assert!(json.contains("Test message"));
    assert!(json.contains("You are a helpful assistant"));
    assert!(json.contains("0.7"));
}

#[test]
fn request_with_tools() {
    let tool = ToolDefinition::new(
        "read_file",
        "Read a file from disk",
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            },
            "required": ["path"]
        }),
    );

    let req = CompletionRequest::new("gpt-4", vec![Message::user("Read main.rs")], 1024)
        .tools(vec![tool]);

    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("read_file"));
    assert!(json.contains("Read a file from disk"));
}

#[test]
fn response_parsing_text_only() {
    let json = r#"{
        "id": "msg_abc123",
        "content": [{"type": "text", "text": "Hello!"}],
        "model": "claude-sonnet-4-5-20250929",
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 10, "output_tokens": 5}
    }"#;

    let resp: CompletionResponse = serde_json::from_str(json).unwrap();

    assert_eq!(resp.id, "msg_abc123");
    assert_eq!(resp.model, "claude-sonnet-4-5-20250929");
    assert_eq!(resp.stop_reason, Some(StopReason::EndTurn));
    assert_eq!(resp.usage.input_tokens, 10);
    assert_eq!(resp.usage.output_tokens, 5);
    assert_eq!(resp.usage.total(), 15);
    assert_eq!(resp.content.len(), 1);

    match &resp.content[0] {
        ContentBlock::Text { text } => {
            assert_eq!(text, "Hello!");
        }
        _ => unreachable!(),
    }
}

#[test]
fn response_parsing_tool_use() {
    let json = r#"{
        "id": "msg_def456",
        "content": [
            {"type": "text", "text": "Let me read that file."},
            {
                "type": "tool_use",
                "id": "tool_xyz",
                "name": "read",
                "input": {"file_path": "test.txt"}
            }
        ],
        "model": "claude-sonnet-4-5-20250929",
        "stop_reason": "tool_use",
        "usage": {"input_tokens": 50, "output_tokens": 20}
    }"#;

    let resp: CompletionResponse = serde_json::from_str(json).unwrap();

    assert_eq!(resp.id, "msg_def456");
    assert_eq!(resp.stop_reason, Some(StopReason::ToolUse));
    assert_eq!(resp.content.len(), 2);

    match &resp.content[0] {
        ContentBlock::Text { text } => {
            assert_eq!(text, "Let me read that file.");
        }
        _ => unreachable!(),
    }

    match &resp.content[1] {
        ContentBlock::ToolUse { id, name, input } => {
            assert_eq!(id, "tool_xyz");
            assert_eq!(name, "read");
            assert_eq!(input["file_path"], "test.txt");
        }
        _ => unreachable!(),
    }
}

#[test]
fn response_stop_reasons() {
    assert_eq!(
        serde_json::from_str::<StopReason>(r#""end_turn""#).unwrap(),
        StopReason::EndTurn
    );
    assert_eq!(
        serde_json::from_str::<StopReason>(r#""max_tokens""#).unwrap(),
        StopReason::MaxTokens
    );
    assert_eq!(
        serde_json::from_str::<StopReason>(r#""tool_use""#).unwrap(),
        StopReason::ToolUse
    );
    assert_eq!(
        serde_json::from_str::<StopReason>(r#""stop_sequence""#).unwrap(),
        StopReason::StopSequence
    );
}

#[test]
fn usage_calculation() {
    let usage = Usage {
        input_tokens: 100,
        output_tokens: 50,
    };
    assert_eq!(usage.total(), 150);

    let usage_default = Usage::default();
    assert_eq!(usage_default.input_tokens, 0);
    assert_eq!(usage_default.output_tokens, 0);
    assert_eq!(usage_default.total(), 0);
}

#[test]
fn content_delta_text() {
    let delta = ContentDelta::TextDelta {
        text: "Hello".to_string(),
    };

    let json = serde_json::to_string(&delta).unwrap();
    assert!(json.contains("text_delta"));
    assert!(json.contains("Hello"));

    let parsed: ContentDelta = serde_json::from_str(&json).unwrap();
    match parsed {
        ContentDelta::TextDelta { text } => {
            assert_eq!(text, "Hello");
        }
        _ => unreachable!(),
    }
}

#[test]
fn content_delta_input_json() {
    let delta = ContentDelta::InputJsonDelta {
        partial_json: r#"{"file"#.to_string(),
    };

    let json = serde_json::to_string(&delta).unwrap();
    assert!(json.contains("input_json_delta"));

    let parsed: ContentDelta = serde_json::from_str(&json).unwrap();
    match parsed {
        ContentDelta::InputJsonDelta { partial_json } => {
            assert_eq!(partial_json, r#"{"file"#);
        }
        _ => unreachable!(),
    }
}

#[test]
fn tool_use_extraction() {
    let json = r#"{
        "id": "msg_123",
        "content": [
            {"type": "text", "text": "I'll help with that."},
            {
                "type": "tool_use",
                "id": "tool_1",
                "name": "grep",
                "input": {"pattern": "TODO", "path": "src/"}
            },
            {
                "type": "tool_use",
                "id": "tool_2",
                "name": "read",
                "input": {"file_path": "README.md"}
            }
        ],
        "model": "claude-sonnet-4-5-20250929",
        "stop_reason": "tool_use",
        "usage": {"input_tokens": 100, "output_tokens": 30}
    }"#;

    let resp: CompletionResponse = serde_json::from_str(json).unwrap();

    // Extract tool uses
    let tool_uses: Vec<_> = resp
        .content
        .iter()
        .filter_map(|block| match block {
            ContentBlock::ToolUse { id, name, input } => Some((id.clone(), name.clone(), input)),
            _ => None,
        })
        .collect();

    assert_eq!(tool_uses.len(), 2);
    assert_eq!(tool_uses[0].0, "tool_1");
    assert_eq!(tool_uses[0].1, "grep");
    assert_eq!(tool_uses[1].0, "tool_2");
    assert_eq!(tool_uses[1].1, "read");
}

#[test]
fn request_builder_chaining() {
    let req = CompletionRequest::new("gpt-4", vec![Message::user("test")], 2048)
        .system("test system")
        .temperature(0.5)
        .stream(true);

    assert_eq!(req.model, "gpt-4");
    assert_eq!(req.max_tokens, 2048);
    assert_eq!(req.system, Some("test system".to_string()));
    assert_eq!(req.temperature, Some(0.5));
    assert!(req.stream);
}
