//! Extension registry for managing loaded extensions.

use super::Extension;
use crate::error::{Result, SaorsaAgentError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Registry for managing loaded extensions.
///
/// Provides thread-safe access to extensions and methods for
/// registration, lookup, and lifecycle notifications.
pub struct ExtensionRegistry {
    extensions: HashMap<String, Box<dyn Extension>>,
}

impl ExtensionRegistry {
    /// Creates a new empty extension registry.
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }

    /// Registers an extension with the runtime.
    ///
    /// Returns an error if an extension with the same name is already registered.
    pub fn register(&mut self, mut ext: Box<dyn Extension>) -> Result<()> {
        let name = ext.name().to_string();
        if self.extensions.contains_key(&name) {
            return Err(SaorsaAgentError::Extension(format!(
                "extension '{}' is already registered",
                name
            )));
        }
        ext.on_load()?;
        self.extensions.insert(name, ext);
        Ok(())
    }

    /// Unregisters an extension by name.
    ///
    /// Returns an error if the extension is not found.
    pub fn unregister(&mut self, name: &str) -> Result<()> {
        let mut ext = self.extensions.remove(name).ok_or_else(|| {
            SaorsaAgentError::Extension(format!("extension '{}' not found", name))
        })?;
        ext.on_unload()?;
        Ok(())
    }

    /// Gets an immutable reference to an extension by name.
    pub fn get(&self, name: &str) -> Option<&dyn Extension> {
        self.extensions.get(name).map(|b| &**b)
    }

    /// Gets a mutable reference to an extension by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut dyn Extension> {
        if let Some(ext) = self.extensions.get_mut(name) {
            Some(&mut **ext)
        } else {
            None
        }
    }

    /// Lists all registered extensions.
    pub fn list(&self) -> Vec<&dyn Extension> {
        self.extensions.values().map(|b| &**b).collect()
    }

    /// Notifies all extensions of a tool call.
    ///
    /// Returns outputs from extensions that intercepted the call.
    pub fn notify_tool_call(&mut self, tool: &str, args: &str) -> Result<Vec<String>> {
        let mut outputs = Vec::new();
        for ext in self.extensions.values_mut() {
            if let Some(output) = ext.on_tool_call(tool, args)? {
                outputs.push(output);
            }
        }
        Ok(outputs)
    }

    /// Notifies all extensions of a message.
    ///
    /// Returns responses from extensions that intercepted the message.
    pub fn notify_message(&mut self, message: &str) -> Result<Vec<String>> {
        let mut responses = Vec::new();
        for ext in self.extensions.values_mut() {
            if let Some(response) = ext.on_message(message)? {
                responses.push(response);
            }
        }
        Ok(responses)
    }

    /// Notifies all extensions of turn start.
    pub fn notify_turn_start(&mut self) -> Result<()> {
        for ext in self.extensions.values_mut() {
            ext.on_turn_start()?;
        }
        Ok(())
    }

    /// Notifies all extensions of turn end.
    pub fn notify_turn_end(&mut self) -> Result<()> {
        for ext in self.extensions.values_mut() {
            ext.on_turn_end()?;
        }
        Ok(())
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe shared extension registry.
pub type SharedExtensionRegistry = Arc<RwLock<ExtensionRegistry>>;

/// Creates a new shared extension registry.
pub fn shared_registry() -> SharedExtensionRegistry {
    Arc::new(RwLock::new(ExtensionRegistry::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExtension {
        name: String,
        loaded: bool,
    }

    impl TestExtension {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                loaded: false,
            }
        }
    }

    impl Extension for TestExtension {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn on_load(&mut self) -> Result<()> {
            self.loaded = true;
            Ok(())
        }

        fn on_unload(&mut self) -> Result<()> {
            self.loaded = false;
            Ok(())
        }
    }

    #[test]
    fn register_extension() {
        let mut registry = ExtensionRegistry::new();
        let ext = Box::new(TestExtension::new("test"));
        let result = registry.register(ext);
        assert!(result.is_ok());
        assert!(registry.get("test").is_some());
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut registry = ExtensionRegistry::new();
        let ext1 = Box::new(TestExtension::new("test"));
        let ext2 = Box::new(TestExtension::new("test"));
        assert!(registry.register(ext1).is_ok());
        let result = registry.register(ext2);
        assert!(result.is_err());
        match result {
            Err(SaorsaAgentError::Extension(msg)) => {
                assert!(msg.contains("already registered"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn unregister_extension() {
        let mut registry = ExtensionRegistry::new();
        let ext = Box::new(TestExtension::new("test"));
        assert!(registry.register(ext).is_ok());
        assert!(registry.unregister("test").is_ok());
        assert!(registry.get("test").is_none());
    }

    #[test]
    fn unregister_nonexistent_fails() {
        let mut registry = ExtensionRegistry::new();
        let result = registry.unregister("nonexistent");
        assert!(result.is_err());
        match result {
            Err(SaorsaAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn list_extensions() {
        let mut registry = ExtensionRegistry::new();
        let ext1 = Box::new(TestExtension::new("test1"));
        let ext2 = Box::new(TestExtension::new("test2"));
        assert!(registry.register(ext1).is_ok());
        assert!(registry.register(ext2).is_ok());
        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn notify_tool_call() {
        struct ToolExtension;
        impl Extension for ToolExtension {
            fn name(&self) -> &str {
                "tool"
            }
            fn version(&self) -> &str {
                "1.0.0"
            }
            fn on_tool_call(&mut self, tool: &str, _args: &str) -> Result<Option<String>> {
                if tool == "test" {
                    Ok(Some("intercepted".to_string()))
                } else {
                    Ok(None)
                }
            }
        }

        let mut registry = ExtensionRegistry::new();
        assert!(registry.register(Box::new(ToolExtension)).is_ok());
        let result = registry.notify_tool_call("test", "{}");
        assert!(result.is_ok());
        let outputs = result.ok().unwrap_or_default();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], "intercepted");
    }

    #[test]
    fn shared_registry_creation() {
        let shared = shared_registry();
        let read_guard = shared.read();
        assert!(read_guard.is_ok());
        let registry = read_guard.ok().unwrap_or_else(|| unreachable!());
        assert_eq!(registry.list().len(), 0);
    }
}
