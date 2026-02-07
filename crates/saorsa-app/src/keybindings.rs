//! Keybinding customization.

use std::collections::HashMap;

/// Action identifier.
pub type Action = String;

/// Key combination.
pub type KeyCombo = String;

/// Keybinding map.
#[derive(Clone, Debug)]
pub struct KeybindingMap {
    /// Action -> Key mapping.
    bindings: HashMap<Action, KeyCombo>,
}

impl KeybindingMap {
    /// Create a new keybinding map with defaults.
    pub fn new() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("send".to_string(), "Ctrl+Enter".to_string());
        bindings.insert("cancel".to_string(), "Escape".to_string());
        bindings.insert("new_chat".to_string(), "Ctrl+N".to_string());
        bindings.insert("model_selector".to_string(), "Ctrl+L".to_string());
        bindings.insert("settings".to_string(), "Ctrl+,".to_string());
        bindings.insert("queue".to_string(), "Ctrl+Q".to_string());
        bindings.insert("save".to_string(), "Ctrl+S".to_string());

        Self { bindings }
    }

    /// Get the key combo for an action.
    pub fn get(&self, action: &str) -> Option<&str> {
        self.bindings.get(action).map(|s| s.as_str())
    }

    /// Set a keybinding.
    pub fn set(&mut self, action: Action, key: KeyCombo) {
        self.bindings.insert(action, key);
    }

    /// Remove a keybinding.
    pub fn remove(&mut self, action: &str) {
        self.bindings.remove(action);
    }

    /// Check if a keybinding conflicts with existing bindings.
    pub fn has_conflict(&self, key: &str) -> Option<&str> {
        self.bindings
            .iter()
            .find(|(_, v)| v.as_str() == key)
            .map(|(k, _)| k.as_str())
    }

    /// Reset to defaults.
    pub fn reset_to_defaults(&mut self) {
        *self = Self::new();
    }

    /// Export all bindings.
    pub fn export(&self) -> HashMap<String, String> {
        self.bindings.clone()
    }

    /// Import bindings.
    pub fn import(&mut self, bindings: HashMap<String, String>) {
        self.bindings = bindings;
    }
}

impl Default for KeybindingMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_keybinding_map() {
        let map = KeybindingMap::new();
        assert!(map.get("send").is_some());
        assert_eq!(map.get("send"), Some("Ctrl+Enter"));
    }

    #[test]
    fn set_keybinding() {
        let mut map = KeybindingMap::new();
        map.set("custom".to_string(), "Ctrl+X".to_string());
        assert_eq!(map.get("custom"), Some("Ctrl+X"));
    }

    #[test]
    fn remove_keybinding() {
        let mut map = KeybindingMap::new();
        map.remove("send");
        assert!(map.get("send").is_none());
    }

    #[test]
    fn detect_conflict() {
        let map = KeybindingMap::new();
        assert_eq!(map.has_conflict("Ctrl+Enter"), Some("send"));
        assert!(map.has_conflict("Ctrl+Z").is_none());
    }

    #[test]
    fn reset_to_defaults() {
        let mut map = KeybindingMap::new();
        map.set("custom".to_string(), "Ctrl+X".to_string());
        map.reset_to_defaults();
        assert!(map.get("custom").is_none());
        assert!(map.get("send").is_some());
    }

    #[test]
    fn export_import() {
        let map = KeybindingMap::new();
        let exported = map.export();
        assert!(!exported.is_empty());

        let mut new_map = KeybindingMap::new();
        new_map.import(exported.clone());
        assert_eq!(new_map.export(), exported);
    }
}
