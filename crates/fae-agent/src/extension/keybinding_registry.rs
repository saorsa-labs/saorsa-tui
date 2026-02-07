//! Keybinding registration system for extensions.

use crate::error::{FaeAgentError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Type alias for keybinding handler functions.
pub type KeybindingHandler = Arc<dyn Fn() -> Result<()> + Send + Sync>;

/// Keybinding definition registered by an extension.
pub struct KeybindingDefinition {
    /// Key combination (e.g., "ctrl+k", "alt+shift+p").
    pub key: String,
    /// Human-readable description.
    pub description: String,
    /// Keybinding handler function.
    pub handler: KeybindingHandler,
}

impl KeybindingDefinition {
    /// Creates a new keybinding definition.
    pub fn new(key: String, description: String, handler: KeybindingHandler) -> Self {
        Self {
            key,
            description,
            handler,
        }
    }
}

/// Registry for extension-provided keybindings.
pub struct KeybindingRegistry {
    keybindings: HashMap<String, KeybindingDefinition>,
}

impl KeybindingRegistry {
    /// Creates a new empty keybinding registry.
    pub fn new() -> Self {
        Self {
            keybindings: HashMap::new(),
        }
    }

    /// Registers a keybinding.
    ///
    /// Returns an error if a keybinding with the same key is already registered.
    pub fn register_keybinding(&mut self, def: KeybindingDefinition) -> Result<()> {
        if self.keybindings.contains_key(&def.key) {
            return Err(FaeAgentError::Extension(format!(
                "keybinding '{}' is already registered",
                def.key
            )));
        }
        self.keybindings.insert(def.key.clone(), def);
        Ok(())
    }

    /// Unregisters a keybinding by key.
    ///
    /// Returns an error if the keybinding is not found.
    pub fn unregister_keybinding(&mut self, key: &str) -> Result<()> {
        self.keybindings
            .remove(key)
            .ok_or_else(|| FaeAgentError::Extension(format!("keybinding '{}' not found", key)))?;
        Ok(())
    }

    /// Gets a keybinding definition by key.
    pub fn get_keybinding(&self, key: &str) -> Option<&KeybindingDefinition> {
        self.keybindings.get(key)
    }

    /// Lists all registered keybindings.
    pub fn list_keybindings(&self) -> Vec<&KeybindingDefinition> {
        self.keybindings.values().collect()
    }

    /// Executes a keybinding handler by key.
    ///
    /// Returns an error if the keybinding is not found or execution fails.
    pub fn execute_keybinding(&self, key: &str) -> Result<()> {
        let def = self
            .keybindings
            .get(key)
            .ok_or_else(|| FaeAgentError::Extension(format!("keybinding '{}' not found", key)))?;
        (def.handler)()
    }
}

impl Default for KeybindingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handler() -> Result<()> {
        Ok(())
    }

    #[test]
    fn register_keybinding() {
        let mut registry = KeybindingRegistry::new();
        let def = KeybindingDefinition::new(
            "ctrl+k".to_string(),
            "Test keybinding".to_string(),
            Arc::new(test_handler),
        );
        let result = registry.register_keybinding(def);
        assert!(result.is_ok());
        assert!(registry.get_keybinding("ctrl+k").is_some());
    }

    #[test]
    fn duplicate_keybinding_fails() {
        let mut registry = KeybindingRegistry::new();
        let def1 = KeybindingDefinition::new(
            "ctrl+k".to_string(),
            "Test 1".to_string(),
            Arc::new(test_handler),
        );
        let def2 = KeybindingDefinition::new(
            "ctrl+k".to_string(),
            "Test 2".to_string(),
            Arc::new(test_handler),
        );
        assert!(registry.register_keybinding(def1).is_ok());
        let result = registry.register_keybinding(def2);
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("already registered"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn unregister_keybinding() {
        let mut registry = KeybindingRegistry::new();
        let def = KeybindingDefinition::new(
            "ctrl+k".to_string(),
            "Test".to_string(),
            Arc::new(test_handler),
        );
        assert!(registry.register_keybinding(def).is_ok());
        assert!(registry.unregister_keybinding("ctrl+k").is_ok());
        assert!(registry.get_keybinding("ctrl+k").is_none());
    }

    #[test]
    fn unregister_nonexistent_fails() {
        let mut registry = KeybindingRegistry::new();
        let result = registry.unregister_keybinding("nonexistent");
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn list_keybindings() {
        let mut registry = KeybindingRegistry::new();
        let def1 = KeybindingDefinition::new(
            "ctrl+k".to_string(),
            "Test 1".to_string(),
            Arc::new(test_handler),
        );
        let def2 = KeybindingDefinition::new(
            "alt+p".to_string(),
            "Test 2".to_string(),
            Arc::new(test_handler),
        );
        assert!(registry.register_keybinding(def1).is_ok());
        assert!(registry.register_keybinding(def2).is_ok());
        let list = registry.list_keybindings();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn execute_keybinding() {
        let mut registry = KeybindingRegistry::new();
        let def = KeybindingDefinition::new(
            "ctrl+k".to_string(),
            "Test".to_string(),
            Arc::new(test_handler),
        );
        assert!(registry.register_keybinding(def).is_ok());
        let result = registry.execute_keybinding("ctrl+k");
        assert!(result.is_ok());
    }

    #[test]
    fn execute_nonexistent_fails() {
        let registry = KeybindingRegistry::new();
        let result = registry.execute_keybinding("nonexistent");
        assert!(result.is_err());
        match result {
            Err(FaeAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }
}
