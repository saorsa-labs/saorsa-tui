//! Integration tests for fae-app features.

use fae_app::autocomplete::Autocomplete;
use fae_app::keybindings::KeybindingMap;
use fae_app::operating_mode::OperatingMode;
use fae_app::widgets::{MessageQueue, ModelSelector, Settings, SettingsScreen};
use std::str::FromStr;

#[test]
fn autocomplete_integration() {
    let ac = Autocomplete::new();
    let suggestions = ac.suggest("/help");
    assert!(!suggestions.is_empty());
    assert!(suggestions.iter().any(|s| s.text == "/help"));
}

#[test]
fn keybindings_integration() {
    let map = KeybindingMap::new();
    assert_eq!(map.get("send"), Some("Ctrl+Enter"));
    assert_eq!(map.get("cancel"), Some("Escape"));
}

#[test]
fn operating_mode_integration() {
    let mode = OperatingMode::default();
    assert_eq!(mode, OperatingMode::Interactive);

    let parsed = OperatingMode::from_str("json");
    assert_eq!(parsed, Ok(OperatingMode::Json));
}

#[test]
fn message_queue_integration() {
    let mut queue = MessageQueue::new();
    queue.add_message("Test message".to_string());
    assert_eq!(queue.len(), 1);
    assert!(!queue.is_empty());
}

#[test]
fn model_selector_integration() {
    let selector = ModelSelector::new(vec!["claude-sonnet-4".to_string()]);
    assert!(!selector.is_visible());
}

#[test]
fn settings_screen_integration() {
    let screen = SettingsScreen::new();
    assert!(!screen.is_visible());
    assert!(!screen.is_dirty());
}

#[test]
fn settings_defaults() {
    let settings = Settings::default();
    assert!(!settings.compact_mode);
    assert!(settings.auto_save);
    assert_eq!(settings.default_model, "claude-sonnet-4");
}
