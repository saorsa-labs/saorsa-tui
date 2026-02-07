//! UI widget registration system for extensions.

use crate::error::{Result, SaorsaAgentError};
use std::collections::HashMap;

/// Widget factory trait for creating custom widgets.
///
/// Extensions implement this trait to provide custom widget types
/// that can be instantiated on demand.
pub trait WidgetFactory: Send + Sync {
    /// Creates a new instance of the widget.
    ///
    /// Returns a boxed widget as a string representation for now,
    /// since we can't directly depend on saorsa-core Widget trait here
    /// (it would create a circular dependency).
    fn create(&self) -> Result<String>;

    /// Returns the unique name of this widget type.
    fn name(&self) -> &str;

    /// Returns a human-readable description.
    fn description(&self) -> &str;
}

/// Configuration for an overlay widget.
#[derive(Debug, Clone)]
pub struct OverlayConfig {
    /// Position (x, y) on screen.
    pub position: (u16, u16),
    /// Size (width, height).
    pub size: (u16, u16),
    /// Z-index for layering.
    pub z_index: u16,
    /// Whether the overlay can be closed by the user.
    pub closeable: bool,
}

impl OverlayConfig {
    /// Creates a new overlay configuration.
    pub fn new(position: (u16, u16), size: (u16, u16), z_index: u16, closeable: bool) -> Self {
        Self {
            position,
            size,
            z_index,
            closeable,
        }
    }
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            position: (0, 0),
            size: (80, 24),
            z_index: 1000,
            closeable: true,
        }
    }
}

/// Registry for extension-provided widget factories.
pub struct WidgetRegistry {
    factories: HashMap<String, Box<dyn WidgetFactory>>,
}

impl WidgetRegistry {
    /// Creates a new empty widget registry.
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Registers a widget factory.
    ///
    /// Returns an error if a factory with the same name is already registered.
    pub fn register_widget(&mut self, factory: Box<dyn WidgetFactory>) -> Result<()> {
        let name = factory.name().to_string();
        if self.factories.contains_key(&name) {
            return Err(SaorsaAgentError::Extension(format!(
                "widget factory '{}' is already registered",
                name
            )));
        }
        self.factories.insert(name, factory);
        Ok(())
    }

    /// Unregisters a widget factory by name.
    ///
    /// Returns an error if the factory is not found.
    pub fn unregister_widget(&mut self, name: &str) -> Result<()> {
        self.factories.remove(name).ok_or_else(|| {
            SaorsaAgentError::Extension(format!("widget factory '{}' not found", name))
        })?;
        Ok(())
    }

    /// Creates a widget instance by factory name.
    ///
    /// Returns an error if the factory is not found or creation fails.
    pub fn create_widget(&self, name: &str) -> Result<String> {
        let factory = self.factories.get(name).ok_or_else(|| {
            SaorsaAgentError::Extension(format!("widget factory '{}' not found", name))
        })?;
        factory.create()
    }

    /// Lists all registered widget factories.
    pub fn list_widgets(&self) -> Vec<&dyn WidgetFactory> {
        self.factories.values().map(|b| &**b).collect()
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidgetFactory {
        name: String,
    }

    impl TestWidgetFactory {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl WidgetFactory for TestWidgetFactory {
        fn create(&self) -> Result<String> {
            Ok(format!("TestWidget({})", self.name))
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "A test widget"
        }
    }

    #[test]
    fn register_widget() {
        let mut registry = WidgetRegistry::new();
        let factory = Box::new(TestWidgetFactory::new("test"));
        let result = registry.register_widget(factory);
        assert!(result.is_ok());
        assert_eq!(registry.list_widgets().len(), 1);
    }

    #[test]
    fn duplicate_widget_fails() {
        let mut registry = WidgetRegistry::new();
        let factory1 = Box::new(TestWidgetFactory::new("test"));
        let factory2 = Box::new(TestWidgetFactory::new("test"));
        assert!(registry.register_widget(factory1).is_ok());
        let result = registry.register_widget(factory2);
        assert!(result.is_err());
        match result {
            Err(SaorsaAgentError::Extension(msg)) => {
                assert!(msg.contains("already registered"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn unregister_widget() {
        let mut registry = WidgetRegistry::new();
        let factory = Box::new(TestWidgetFactory::new("test"));
        assert!(registry.register_widget(factory).is_ok());
        assert!(registry.unregister_widget("test").is_ok());
        assert_eq!(registry.list_widgets().len(), 0);
    }

    #[test]
    fn unregister_nonexistent_fails() {
        let mut registry = WidgetRegistry::new();
        let result = registry.unregister_widget("nonexistent");
        assert!(result.is_err());
        match result {
            Err(SaorsaAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn create_widget() {
        let mut registry = WidgetRegistry::new();
        let factory = Box::new(TestWidgetFactory::new("test"));
        assert!(registry.register_widget(factory).is_ok());
        let result = registry.create_widget("test");
        assert!(result.is_ok());
        let widget_str = result.ok().unwrap_or_default();
        assert_eq!(widget_str, "TestWidget(test)");
    }

    #[test]
    fn create_nonexistent_fails() {
        let registry = WidgetRegistry::new();
        let result = registry.create_widget("nonexistent");
        assert!(result.is_err());
        match result {
            Err(SaorsaAgentError::Extension(msg)) => {
                assert!(msg.contains("not found"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn list_widgets() {
        let mut registry = WidgetRegistry::new();
        let factory1 = Box::new(TestWidgetFactory::new("test1"));
        let factory2 = Box::new(TestWidgetFactory::new("test2"));
        assert!(registry.register_widget(factory1).is_ok());
        assert!(registry.register_widget(factory2).is_ok());
        let list = registry.list_widgets();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn overlay_config_default() {
        let config = OverlayConfig::default();
        assert_eq!(config.position, (0, 0));
        assert_eq!(config.size, (80, 24));
        assert_eq!(config.z_index, 1000);
        assert!(config.closeable);
    }

    #[test]
    fn overlay_config_new() {
        let config = OverlayConfig::new((10, 20), (100, 50), 2000, false);
        assert_eq!(config.position, (10, 20));
        assert_eq!(config.size, (100, 50));
        assert_eq!(config.z_index, 2000);
        assert!(!config.closeable);
    }
}
