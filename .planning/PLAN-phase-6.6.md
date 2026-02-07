# Phase 6.6: Extension System

## Overview
Implement a trait-based extension system that allows dynamic plugin loading without WASM dependencies. The system uses dynamic dispatch (Box<dyn Extension>) with a registry pattern that could be backed by WASM in the future.

## Tasks

### Task 1: Extension Trait & Lifecycle Hooks
**File**: `crates/saorsa-agent/src/extension/mod.rs`
- Define `Extension` trait with lifecycle methods:
  - `fn name(&self) -> &str`
  - `fn version(&self) -> &str`
  - `fn on_load(&mut self) -> Result<()>`
  - `fn on_unload(&mut self) -> Result<()>`
  - `fn on_tool_call(&mut self, tool: &str, args: &str) -> Result<Option<String>>`
  - `fn on_message(&mut self, message: &str) -> Result<Option<String>>`
  - `fn on_turn_start(&mut self) -> Result<()>`
  - `fn on_turn_end(&mut self) -> Result<()>`
- Define `ExtensionMetadata` struct with name, version, description, author
- Define `SaorsaAgentError` variants for extension errors
- Add module to `crates/saorsa-agent/src/lib.rs`

### Task 2: Extension Registry
**File**: `crates/saorsa-agent/src/extension/registry.rs`
- Implement `ExtensionRegistry` struct:
  - `HashMap<String, Box<dyn Extension>>` for loaded extensions
  - `fn register(&mut self, ext: Box<dyn Extension>) -> Result<()>`
  - `fn unregister(&mut self, name: &str) -> Result<()>`
  - `fn get(&self, name: &str) -> Option<&dyn Extension>`
  - `fn get_mut(&mut self, name: &str) -> Option<&mut dyn Extension>`
  - `fn list(&self) -> Vec<&dyn Extension>`
  - `fn notify_tool_call(&mut self, tool: &str, args: &str) -> Result<Vec<String>>`
  - `fn notify_message(&mut self, message: &str) -> Result<Vec<String>>`
  - `fn notify_turn_start(&mut self) -> Result<()>`
  - `fn notify_turn_end(&mut self) -> Result<()>`
- Thread-safe with `Arc<RwLock<ExtensionRegistry>>`
- Export from `extension/mod.rs`

### Task 3: Tool Registration System
**File**: `crates/saorsa-agent/src/extension/tool_registry.rs`
- Define `ToolDefinition` struct:
  - `name: String`
  - `description: String`
  - `parameters: Vec<ToolParameter>`
  - `handler: Arc<dyn Fn(&str) -> Result<String> + Send + Sync>`
- Define `ToolParameter` struct with name, type, description, required flag
- Implement `ToolRegistry`:
  - `fn register_tool(&mut self, def: ToolDefinition) -> Result<()>`
  - `fn unregister_tool(&mut self, name: &str) -> Result<()>`
  - `fn get_tool(&self, name: &str) -> Option<&ToolDefinition>`
  - `fn list_tools(&self) -> Vec<&ToolDefinition>`
  - `fn execute_tool(&self, name: &str, args: &str) -> Result<String>`
- Export from `extension/mod.rs`

### Task 4: Command Registration System
**File**: `crates/saorsa-agent/src/extension/command_registry.rs`
- Define `CommandDefinition` struct:
  - `name: String`
  - `description: String`
  - `usage: String`
  - `handler: Arc<dyn Fn(&[&str]) -> Result<String> + Send + Sync>`
- Implement `CommandRegistry`:
  - `fn register_command(&mut self, def: CommandDefinition) -> Result<()>`
  - `fn unregister_command(&mut self, name: &str) -> Result<()>`
  - `fn get_command(&self, name: &str) -> Option<&CommandDefinition>`
  - `fn list_commands(&self) -> Vec<&CommandDefinition>`
  - `fn execute_command(&self, name: &str, args: &[&str]) -> Result<String>`
- Export from `extension/mod.rs`

### Task 5: Keybinding Registration System
**File**: `crates/saorsa-agent/src/extension/keybinding_registry.rs`
- Define `KeybindingDefinition` struct:
  - `key: String` (e.g., "ctrl+k")
  - `description: String`
  - `handler: Arc<dyn Fn() -> Result<()> + Send + Sync>`
- Implement `KeybindingRegistry`:
  - `fn register_keybinding(&mut self, def: KeybindingDefinition) -> Result<()>`
  - `fn unregister_keybinding(&mut self, key: &str) -> Result<()>`
  - `fn get_keybinding(&self, key: &str) -> Option<&KeybindingDefinition>`
  - `fn list_keybindings(&self) -> Vec<&KeybindingDefinition>`
  - `fn execute_keybinding(&self, key: &str) -> Result<()>`
- Export from `extension/mod.rs`

### Task 6: UI Widget Registration System
**File**: `crates/saorsa-agent/src/extension/widget_registry.rs`
- Define `WidgetFactory` trait:
  - `fn create(&self) -> Box<dyn Widget>` (where Widget is from saorsa-core)
  - `fn name(&self) -> &str`
  - `fn description(&self) -> &str`
- Define `OverlayConfig` struct:
  - `position: (u16, u16)`
  - `size: (u16, u16)`
  - `z_index: u16`
  - `closeable: bool`
- Implement `WidgetRegistry`:
  - `fn register_widget(&mut self, factory: Box<dyn WidgetFactory>) -> Result<()>`
  - `fn unregister_widget(&mut self, name: &str) -> Result<()>`
  - `fn create_widget(&self, name: &str) -> Result<Box<dyn Widget>>`
  - `fn list_widgets(&self) -> Vec<&dyn WidgetFactory>`
- Export from `extension/mod.rs`

### Task 7: Package Management System
**File**: `crates/saorsa-agent/src/extension/package_manager.rs`
- Define `ExtensionPackage` struct:
  - `metadata: ExtensionMetadata`
  - `path: PathBuf`
  - `config: HashMap<String, serde_json::Value>`
  - `enabled: bool`
- Implement `PackageManager`:
  - `fn new(extensions_dir: PathBuf) -> Self`
  - `fn install(&mut self, path: &Path) -> Result<()>`
  - `fn uninstall(&mut self, name: &str) -> Result<()>`
  - `fn list(&self) -> Vec<&ExtensionPackage>`
  - `fn enable(&mut self, name: &str) -> Result<()>`
  - `fn disable(&mut self, name: &str) -> Result<()>`
  - `fn get_config(&self, name: &str) -> Option<&HashMap<String, serde_json::Value>>`
  - `fn set_config(&mut self, name: &str, key: String, value: serde_json::Value) -> Result<()>`
  - `fn load_package(&self, name: &str) -> Result<Box<dyn Extension>>`
- Persist package list to `extensions.json` in extensions dir
- Export from `extension/mod.rs`

### Task 8: Integration & Testing
**Files**:
- `crates/saorsa-agent/src/extension/tests.rs` (integration tests)
- Update `crates/saorsa-agent/src/lib.rs` to re-export extension types

**Tests**:
1. Test extension lifecycle (load, tool_call, message, turn hooks, unload)
2. Test extension registry (register, unregister, list, notify)
3. Test tool registry (register, execute, unregister)
4. Test command registry (register, execute, unregister)
5. Test keybinding registry (register, execute, unregister)
6. Test widget registry (register, create, unregister)
7. Test package manager (install, enable, disable, uninstall, config)
8. Test error handling (duplicate names, not found, lifecycle failures)

**Integration**:
- Add `pub use extension::*;` to `saorsa-agent/src/lib.rs`
- Update agent runtime to call extension hooks at appropriate times
- Document extension system in module docs

## Success Criteria
- All 8 tasks complete
- Zero clippy warnings
- All tests pass (1709+ tests)
- `cargo fmt` clean
- Extension system ready for use (trait-based, no WASM dependency)
- Full build gates pass
