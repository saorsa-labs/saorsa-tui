# Phase 6.5: Full UI Features

**Status:** Complete
**Tasks Completed:** 8/8

## Task List

### Task 1: Model Selector Widget
**Status:** Complete
**Description:** Create ModelSelector widget with fuzzy search, favorites cycling (Ctrl+L), and provider filtering. Uses SelectList internally with custom rendering for model metadata (provider, context window, pricing).

**Acceptance Criteria:**
- Ctrl+L opens model selector overlay
- Fuzzy search by model name
- Display provider, context window, pricing
- F key toggles favorite status
- Escape closes, Enter selects
- Persists favorites to config

---

### Task 2: Settings Screen
**Status:** Complete
**Description:** Create SettingsScreen widget with tabs for General, Models, Keybindings, Context. Uses Tabs widget with form controls (Switch, OptionList, TextArea). Persists to AgentConfig.

**Acceptance Criteria:**
- Ctrl+, opens settings screen
- Tabs for General/Models/Keybindings/Context
- General: compact mode, thinking mode, auto-save
- Models: default model, temperature, max tokens
- Keybindings: editable key mappings
- Context: discovery config, max tokens per section
- Save/Cancel buttons

---

### Task 3: Remaining Slash Commands
**Status:** Complete
**Description:** Implement missing slash commands: /model, /thinking, /compact, /share, /login, /logout, /settings, /hotkeys, /clear, /help. Each command has handler in commands/ module.

**Acceptance Criteria:**
- /model [name] - switch model
- /thinking - toggle thinking mode
- /compact - toggle compact mode
- /share - export conversation with shareable link
- /login - authenticate with API providers
- /logout - clear credentials
- /settings - open settings screen
- /hotkeys - show keybindings help
- /clear - clear conversation
- /help - show command list

---

### Task 4: Message Queuing System
**Status:** Complete
**Description:** Implement MessageQueue for steering + follow-up messages. Allows queueing multiple messages before sending, with editing and reordering. Uses RichLog for queue display.

**Acceptance Criteria:**
- Ctrl+Q toggles queue panel
- Messages added to queue instead of immediate send
- Edit queued messages with Ctrl+E
- Reorder with Ctrl+Up/Down
- Delete with Ctrl+D
- Send all with Ctrl+Enter
- Clear queue with Ctrl+X

---

### Task 5: Multi-line Editor with Autocomplete
**Status:** Complete
**Description:** Enhance TextArea with autocomplete for @files and /commands. Shows popup with fuzzy-matched suggestions. Tab/Enter completes, Escape cancels.

**Acceptance Criteria:**
- @ triggers file autocomplete (uses file discovery)
- / triggers command autocomplete
- Fuzzy matching with highlighting
- Up/Down navigate suggestions
- Tab or Enter completes
- Escape cancels
- Multiple @files in one message
- Autocomplete popup overlays editor

---

### Task 6: Operating Modes
**Status:** Complete
**Description:** Implement OperatingMode enum (Interactive, Print, Json, Rpc) with mode-specific rendering and I/O. Interactive is default, others for CLI integration.

**Acceptance Criteria:**
- OperatingMode enum with 4 variants
- --mode flag in saorsa-cli
- Interactive: full TUI (existing)
- Print: plain text streaming to stdout
- Json: JSON Lines output
- Rpc: JSON-RPC 2.0 over stdio
- Mode persisted in session config

---

### Task 7: Keybinding Customization
**Status:** Complete
**Description:** Create KeybindingMap with customizable key mappings. Persists to config, loaded at startup. Settings screen allows editing. Default bindings in constants.

**Acceptance Criteria:**
- KeybindingMap in saorsa-app/src/keybindings.rs
- Default bindings match current hardcoded keys
- Load from config at startup
- Edit in Settings screen Keybindings tab
- Validation prevents conflicts
- Reset to defaults button
- Export/import keybindings

---

### Task 8: Integration Tests & Documentation
**Status:** Complete
**Description:** Add integration tests for all new features. Update docs with feature descriptions, keybindings reference, command reference. Add examples for each operating mode.

**Acceptance Criteria:**
- Integration tests for model selector
- Integration tests for settings screen
- Integration tests for message queue
- Integration tests for autocomplete
- Integration tests for each slash command
- Operating mode examples in examples/
- Keybindings reference in docs/
- Command reference in docs/
- All tests pass, zero clippy warnings
