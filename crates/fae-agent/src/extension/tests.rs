//! Integration tests for the extension system.

#[cfg(test)]
mod integration_tests {
    use crate::error::Result;
    use crate::extension::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    // Test extension implementation
    struct TestExtension {
        name: String,
        tool_calls: Vec<String>,
        messages: Vec<String>,
        turn_count: usize,
    }

    impl TestExtension {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                tool_calls: Vec::new(),
                messages: Vec::new(),
                turn_count: 0,
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

        fn on_tool_call(&mut self, tool: &str, args: &str) -> Result<Option<String>> {
            self.tool_calls.push(format!("{}:{}", tool, args));
            if tool == "intercept" {
                Ok(Some("intercepted".to_string()))
            } else {
                Ok(None)
            }
        }

        fn on_message(&mut self, message: &str) -> Result<Option<String>> {
            self.messages.push(message.to_string());
            if message.contains("intercept") {
                Ok(Some("message intercepted".to_string()))
            } else {
                Ok(None)
            }
        }

        fn on_turn_start(&mut self) -> Result<()> {
            self.turn_count += 1;
            Ok(())
        }

        fn on_turn_end(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn extension_lifecycle() {
        struct LifecycleExtension {
            loaded: bool,
            unloaded: bool,
        }

        impl LifecycleExtension {
            fn new() -> Self {
                Self {
                    loaded: false,
                    unloaded: false,
                }
            }
        }

        impl Extension for LifecycleExtension {
            fn name(&self) -> &str {
                "lifecycle"
            }

            fn version(&self) -> &str {
                "1.0.0"
            }

            fn on_load(&mut self) -> Result<()> {
                self.loaded = true;
                Ok(())
            }

            fn on_unload(&mut self) -> Result<()> {
                self.unloaded = true;
                Ok(())
            }
        }

        let mut registry = ExtensionRegistry::new();
        let ext = Box::new(LifecycleExtension::new());
        let result = registry.register(ext);
        assert!(result.is_ok());

        let ext_ref = registry.get("lifecycle");
        assert!(ext_ref.is_some());

        let result = registry.unregister("lifecycle");
        assert!(result.is_ok());
    }

    #[test]
    fn extension_registry_notifications() {
        let mut registry = ExtensionRegistry::new();
        let ext = Box::new(TestExtension::new("test"));
        assert!(registry.register(ext).is_ok());

        let result = registry.notify_tool_call("test_tool", "args");
        assert!(result.is_ok());

        let result = registry.notify_tool_call("intercept", "args");
        assert!(result.is_ok());
        let outputs = result.ok().unwrap_or_default();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], "intercepted");

        let result = registry.notify_message("test message");
        assert!(result.is_ok());

        let result = registry.notify_message("intercept this");
        assert!(result.is_ok());
        let responses = result.ok().unwrap_or_default();
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], "message intercepted");

        assert!(registry.notify_turn_start().is_ok());
        assert!(registry.notify_turn_end().is_ok());
    }

    #[test]
    fn tool_registry_operations() {
        let mut registry = ToolRegistry::new();

        let handler: ToolHandler = Arc::new(|args| Ok(format!("result: {}", args)));
        let param = ToolParameter::new(
            "input".to_string(),
            "string".to_string(),
            "Input parameter".to_string(),
            true,
        );
        let def = ToolDefinition::new(
            "test_tool".to_string(),
            "Test tool".to_string(),
            vec![param],
            handler,
        );

        assert!(registry.register_tool(def).is_ok());
        assert!(registry.get_tool("test_tool").is_some());

        let result = registry.execute_tool("test_tool", "hello");
        assert!(result.is_ok());
        let output = result.ok().unwrap_or_default();
        assert_eq!(output, "result: hello");

        assert!(registry.unregister_tool("test_tool").is_ok());
        assert!(registry.get_tool("test_tool").is_none());
    }

    #[test]
    fn command_registry_operations() {
        let mut registry = CommandRegistry::new();

        let handler: CommandHandler = Arc::new(|args| Ok(format!("cmd: {}", args.join(" "))));
        let def = CommandDefinition::new(
            "test".to_string(),
            "Test command".to_string(),
            "test <args>".to_string(),
            handler,
        );

        assert!(registry.register_command(def).is_ok());
        assert!(registry.get_command("test").is_some());

        let result = registry.execute_command("test", &["arg1", "arg2"]);
        assert!(result.is_ok());
        let output = result.ok().unwrap_or_default();
        assert_eq!(output, "cmd: arg1 arg2");

        assert!(registry.unregister_command("test").is_ok());
        assert!(registry.get_command("test").is_none());
    }

    #[test]
    fn keybinding_registry_operations() {
        let mut registry = KeybindingRegistry::new();

        let handler: KeybindingHandler = Arc::new(|| Ok(()));
        let def =
            KeybindingDefinition::new("ctrl+k".to_string(), "Test keybinding".to_string(), handler);

        assert!(registry.register_keybinding(def).is_ok());
        assert!(registry.get_keybinding("ctrl+k").is_some());

        let result = registry.execute_keybinding("ctrl+k");
        assert!(result.is_ok());

        assert!(registry.unregister_keybinding("ctrl+k").is_ok());
        assert!(registry.get_keybinding("ctrl+k").is_none());
    }

    #[test]
    fn widget_registry_operations() {
        struct TestWidgetFactory;

        impl WidgetFactory for TestWidgetFactory {
            fn create(&self) -> Result<String> {
                Ok("TestWidget".to_string())
            }

            fn name(&self) -> &str {
                "test_widget"
            }

            fn description(&self) -> &str {
                "A test widget"
            }
        }

        let mut registry = WidgetRegistry::new();
        let factory: Box<dyn WidgetFactory> = Box::new(TestWidgetFactory);

        assert!(registry.register_widget(factory).is_ok());
        assert_eq!(registry.list_widgets().len(), 1);

        let result = registry.create_widget("test_widget");
        assert!(result.is_ok());
        let widget = result.ok().unwrap_or_default();
        assert_eq!(widget, "TestWidget");

        assert!(registry.unregister_widget("test_widget").is_ok());
        assert_eq!(registry.list_widgets().len(), 0);
    }

    #[test]
    fn package_manager_operations() {
        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let ext_dir = temp.path().join("test-extension");
        std::fs::create_dir(&ext_dir).unwrap_or_else(|_| unreachable!());

        let mut manager = PackageManager::new(temp.path().to_path_buf());

        assert!(manager.install(&ext_dir).is_ok());
        assert_eq!(manager.list().len(), 1);

        assert!(manager.enable("test-extension").is_ok());
        let pkg = manager.list()[0];
        assert!(pkg.enabled);

        assert!(manager.disable("test-extension").is_ok());
        let pkg = manager.list()[0];
        assert!(!pkg.enabled);

        let result = manager.set_config(
            "test-extension",
            "key".to_string(),
            serde_json::json!("value"),
        );
        assert!(result.is_ok());

        let config = manager.get_config("test-extension");
        assert!(config.is_some());
        let config = config.unwrap_or_else(|| unreachable!());
        assert_eq!(config.get("key"), Some(&serde_json::json!("value")));

        assert!(manager.uninstall("test-extension").is_ok());
        assert_eq!(manager.list().len(), 0);
    }

    #[test]
    fn error_handling() {
        let mut ext_registry = ExtensionRegistry::new();
        let result = ext_registry.unregister("nonexistent");
        assert!(result.is_err());

        let tool_registry = ToolRegistry::new();
        let result = tool_registry.execute_tool("nonexistent", "args");
        assert!(result.is_err());

        let cmd_registry = CommandRegistry::new();
        let result = cmd_registry.execute_command("nonexistent", &[]);
        assert!(result.is_err());

        let key_registry = KeybindingRegistry::new();
        let result = key_registry.execute_keybinding("nonexistent");
        assert!(result.is_err());

        let widget_registry = WidgetRegistry::new();
        let result = widget_registry.create_widget("nonexistent");
        assert!(result.is_err());

        let temp = TempDir::new().unwrap_or_else(|_| unreachable!());
        let mut pkg_manager = PackageManager::new(temp.path().to_path_buf());
        let result = pkg_manager.uninstall("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn overlay_config_test() {
        let config = OverlayConfig::new((10, 20), (100, 50), 2000, false);
        assert_eq!(config.position, (10, 20));
        assert_eq!(config.size, (100, 50));
        assert_eq!(config.z_index, 2000);
        assert!(!config.closeable);

        let default_config = OverlayConfig::default();
        assert!(default_config.closeable);
    }
}
