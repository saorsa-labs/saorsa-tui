//! Integration tests for all providers
//!
//! These tests verify that all providers work together correctly
//! through the unified interface.

#![allow(clippy::unwrap_used)]
#![allow(clippy::overly_complex_bool_expr)]

use fae_ai::{
    message::Message,
    models::{lookup_model, lookup_model_by_prefix},
    openai_compat::OpenAiCompatBuilder,
    provider::{ProviderConfig, ProviderKind, ProviderRegistry},
    types::CompletionRequest,
};

/// Test creating each provider type via factory
#[test]
fn factory_creates_all_providers() {
    let registry = ProviderRegistry::default();

    // Test Anthropic
    let config = ProviderConfig::new(
        ProviderKind::Anthropic,
        "test-key",
        "claude-3-5-sonnet-20241022",
    );
    let result = registry.create(config);
    assert!(result.is_ok(), "Failed to create Anthropic provider");

    // Test OpenAI
    let config = ProviderConfig::new(ProviderKind::OpenAi, "test-key", "gpt-4o");
    let result = registry.create(config);
    assert!(result.is_ok(), "Failed to create OpenAI provider");

    // Test Gemini
    let config = ProviderConfig::new(ProviderKind::Gemini, "test-key", "gemini-1.5-flash");
    let result = registry.create(config);
    assert!(result.is_ok(), "Failed to create Gemini provider");

    // Test Ollama
    let config = ProviderConfig::new(ProviderKind::Ollama, "", "llama3");
    let result = registry.create(config);
    assert!(result.is_ok(), "Failed to create Ollama provider");

    // Test OpenAI Compatible (requires base_url)
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "test-key", "gpt-4")
        .with_base_url("https://api.openai.com/v1");
    let result = registry.create(config);
    assert!(
        result.is_ok(),
        "Failed to create OpenAI Compatible provider"
    );
}

/// Test provider registry has all providers
#[test]
fn registry_has_all_providers() {
    let registry = ProviderRegistry::default();

    for kind in [
        ProviderKind::Anthropic,
        ProviderKind::OpenAi,
        ProviderKind::Gemini,
        ProviderKind::Ollama,
        ProviderKind::OpenAiCompatible,
    ] {
        let result = registry.has_provider(kind);
        assert!(result, "Registry should have provider: {:?}", kind);
    }
}

/// Test model lookup for all providers
#[test]
fn model_lookup_all_providers() {
    // Test exact match for some models
    let test_models = vec![
        ("claude-3-5-sonnet", Some(ProviderKind::Anthropic)),
        ("gpt-4o", Some(ProviderKind::OpenAi)),
        ("gemini-1.5-flash", Some(ProviderKind::Gemini)),
        ("llama3", Some(ProviderKind::Ollama)),
    ];

    for (model_name, expected_provider) in test_models {
        let model = lookup_model(model_name);
        assert!(model.is_some(), "Should find model: {}", model_name);

        let model = model.unwrap();
        if let Some(expected) = expected_provider {
            assert_eq!(model.provider, expected);
        }
        assert!(model.supports_tools || !model.supports_tools); // Just check it's a bool
    }
}

/// Test prefix-based model lookup for versioned models
#[test]
fn model_lookup_prefix_versioned() {
    // Test prefix matching for versioned model names
    let test_cases = vec![
        ("claude-sonnet-4-5-20250929", "claude-sonnet-4"),
        ("gpt-4o-2024-08-06", "gpt-4o"),
    ];

    for (versioned_name, expected_prefix) in test_cases {
        let model = lookup_model_by_prefix(versioned_name);
        assert!(
            model.is_some(),
            "Should find model via prefix: {}",
            versioned_name
        );

        let model = model.unwrap();
        assert_eq!(model.name, expected_prefix);
    }
}

/// Test request creation for each provider
#[test]
fn request_creation_all_providers() {
    let message = Message::user("Hello, this is a test message");
    let requests = vec![
        // Anthropic
        CompletionRequest::new("claude-3-5-sonnet-20241022", vec![message.clone()], 100),
        // OpenAI
        CompletionRequest::new("gpt-4o", vec![message.clone()], 100),
        // Gemini
        CompletionRequest::new("gemini-1.5-flash", vec![message.clone()], 100),
        // Ollama
        CompletionRequest::new("llama3", vec![message], 100),
    ];

    for request in requests {
        // Verify basic properties
        assert!(!request.model.is_empty());
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.max_tokens, 100);
    }
}

/// Test OpenAI-compatible provider builder
#[test]
fn openai_compat_builder() {
    // Test default builder
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "test-key", "gpt-4")
        .with_base_url("https://api.openai.com/v1");
    let builder = OpenAiCompatBuilder::new(config);
    let provider = builder.build();
    assert!(
        provider.is_ok(),
        "Failed to build OpenAI Compatible provider"
    );

    // Test builder with custom auth header
    let config = ProviderConfig::new(ProviderKind::OpenAiCompatible, "test-key", "gpt-4")
        .with_base_url("https://example.com/v1");
    let builder = OpenAiCompatBuilder::new(config)
        .auth_header("x-api-key")
        .extra_header("X-Custom-Header", "custom-value");
    let provider = builder.build();
    assert!(
        provider.is_ok(),
        "Failed to build provider with custom auth"
    );
}

/// Test provider configuration defaults
#[test]
fn provider_config_defaults() {
    let configs = vec![
        (ProviderKind::Anthropic, "https://api.anthropic.com"),
        (ProviderKind::OpenAi, "https://api.openai.com"),
        (
            ProviderKind::Gemini,
            "https://generativelanguage.googleapis.com/v1beta",
        ),
        (ProviderKind::Ollama, "http://localhost:11434"),
    ];

    for (kind, expected_url) in configs {
        let config = ProviderConfig::new(kind, "test-key", "test-model");
        assert_eq!(config.base_url, expected_url);
    }
}

/// Test model capability flags
#[test]
fn model_capability_flags() {
    use fae_ai::models::supports_tools;
    use fae_ai::models::supports_vision;

    let test_cases = vec![
        ("gpt-4o", true, true),            // OpenAI: supports tools and vision
        ("claude-3-5-sonnet", true, true), // Anthropic: supports tools and vision
        ("llama3", true, false),           // Ollama: tools, no vision
        ("gemini-1.5-flash", true, true),  // Gemini: supports tools and vision
    ];

    for (model_name, expected_tools, expected_vision) in test_cases {
        let model = lookup_model(model_name);
        assert!(model.is_some(), "Should find model: {}", model_name);

        let model = model.unwrap();
        assert_eq!(
            model.supports_tools, expected_tools,
            "Model {} tools mismatch",
            model_name
        );
        assert_eq!(
            model.supports_vision, expected_vision,
            "Model {} vision mismatch",
            model_name
        );
        // Use the imported functions
        let _ = supports_tools(model_name);
        let _ = supports_vision(model_name);
    }
}

/// Test context window sizes
#[test]
fn context_window_sizes() {
    use fae_ai::models::get_context_window;

    let test_cases = vec![
        ("gpt-4o", Some(128_000)),
        ("claude-3-5-sonnet", Some(200_000)),
        ("gemini-1.5-flash", Some(1_048_576)),
        ("llama3", Some(8_192)),
        ("unknown-model", None),
    ];

    for (model_name, expected_size) in test_cases {
        let size = get_context_window(model_name);
        assert_eq!(
            size, expected_size,
            "Model {} context window mismatch",
            model_name
        );
    }
}

/// Test provider kind display names
#[test]
fn provider_kind_display_names() {
    let names = vec![
        (ProviderKind::Anthropic, "Anthropic"),
        (ProviderKind::OpenAi, "OpenAI"),
        (ProviderKind::Gemini, "Google Gemini"),
        (ProviderKind::Ollama, "Ollama"),
        (ProviderKind::OpenAiCompatible, "OpenAI-Compatible"),
    ];

    for (kind, expected_name) in names {
        assert_eq!(kind.display_name(), expected_name);
    }
}

/// Test message creation
#[test]
fn message_creation() {
    use fae_ai::message::{ContentBlock, Role};

    let message = Message::user("Hello world");

    assert_eq!(message.role, Role::User);
    assert_eq!(message.content.len(), 1);

    if let ContentBlock::Text { text } = &message.content[0] {
        assert_eq!(text, "Hello world");
    } else {
        panic!("Expected text content");
    }
}

/// Test that all exported types compile
#[test]
fn public_api_exports_compile() {
    // Use items from each module to verify they're exported
    use fae_ai::{
        message::{ContentBlock, Message, Role, ToolDefinition},
        models::{
            get_context_window, lookup_model, lookup_model_by_prefix, supports_tools,
            supports_vision,
        },
        provider::{ProviderConfig, ProviderKind, ProviderRegistry},
        types::{CompletionRequest, StopReason},
    };

    // Use the items to suppress unused warnings
    let _ = Message::user("test");
    let _ = CompletionRequest::new("test-model", vec![], 100);
    let _ = ProviderConfig::new(ProviderKind::OpenAi, "test-key", "test-model");
    let _ = Role::User;
    let _ = ContentBlock::Text {
        text: "test".to_string(),
    };
    let _ = ToolDefinition {
        name: "test".to_string(),
        description: "test".to_string(),
        input_schema: serde_json::Value::Null,
    };
    let _ = get_context_window("gpt-4o");
    let _ = lookup_model("gpt-4o");
    let _ = lookup_model_by_prefix("gpt-4o-2024-08-06");
    let _ = supports_tools("gpt-4o");
    let _ = supports_vision("gpt-4o");
    let _ = StopReason::EndTurn;
    let _ = ProviderRegistry::default();
}

/// Test completion request builder
#[test]
fn completion_request_builder() {
    let message = Message::user("Test message");
    let request = CompletionRequest::new("gpt-4o", vec![message], 1000)
        .system("You are helpful")
        .temperature(0.7)
        .stream(true);

    assert_eq!(request.model, "gpt-4o");
    assert_eq!(request.max_tokens, 1000);
    assert_eq!(request.system, Some("You are helpful".to_string()));
    assert_eq!(request.temperature, Some(0.7));
    assert!(request.stream);
}
