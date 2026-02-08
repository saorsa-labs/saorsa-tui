# Saorsa TUI — Project Roadmap

> A Retained-Mode, CSS-Styled Terminal UI Framework in Rust, with a Full AI Coding Agent as Showcase Application

**License:** MIT OR Apache-2.0
**Author:** David Irvine / Saorsa Labs

---

## Milestone 1: Foundation

**Goal**: Minimal saorsa-core rendering pipeline + basic saorsa-app that can chat with Claude via streaming responses.

### Phase 1.1: Workspace & Core Types
- Set up Cargo workspace (saorsa-core, saorsa-ai, saorsa-agent, saorsa-app, saorsa-cli)
- Define core types: Segment, Style, Color, Cell, Size, Rect
- Terminal abstraction trait + crossterm backend
- Error types with thiserror for each crate
- CLAUDE.md / AGENTS.md for the project

### Phase 1.2: Screen Buffer & Rendering
- ScreenBuffer with Cell grid
- Differential rendering (diff current vs previous)
- ANSI escape sequence emission
- CSI 2026 synchronized output
- Basic color support (truecolor, 256, 16, auto-detect)

### Phase 1.3: Basic Layout & Widgets
- Vertical/Horizontal stacking layout
- Dock positioning (top/bottom)
- Basic widgets: Static, Label, Container, ScrollView, Input
- Event loop (keyboard, resize)
- Focus management (basic)

### Phase 1.4: Anthropic Provider (saorsa-ai)
- Anthropic Messages API client
- Streaming response handling (SSE)
- Tool calling abstraction
- Context/Message types
- Token counting basics

### Phase 1.5: Agent Loop (saorsa-agent)
- Core agent loop with tool execution
- Bash tool (PTY, streaming output, timeout)
- Tool registry
- Agent event types for UI integration
- Basic error recovery

### Phase 1.6: Minimal Chat App (saorsa-app)
- CLI argument parsing (clap)
- Main chat screen layout (header, messages, editor, footer)
- Streaming markdown display (basic — pre-compositor)
- Input editor (single-line first)
- Connect saorsa-ai + saorsa-agent + saorsa-core into working chat

**Milestone 1 Deliverable**: `saorsa-tui` binary that can chat with Claude, execute bash commands, and display streaming responses in the terminal.

---

## Milestone 2: CSS & Layout Engine

**Goal**: Full TCSS engine and layout system. Widgets styled entirely via CSS.

### Phase 2.1: TCSS Parser
- Tokenizer (using cssparser from Servo)
- Selector parsing (type, class, ID, compound, child, descendant, pseudo-class)
- Property parsing (~70+ terminal CSS properties)
- AST representation

### Phase 2.2: Selector Matching & Cascade
- Specificity calculation
- Widget tree walking for selector matching
- Cascade resolution (specificity + source order)
- Match result caching with invalidation

### Phase 2.3: CSS Variables & Theming
- Variable definition (:root, .dark, .light)
- Variable resolution ($variable syntax)
- Theme system (dark/light/custom)
- Live CSS file reloading (file watcher)

### Phase 2.4: Taffy Layout Integration
- Taffy crate integration
- Map TCSS properties to Taffy styles
- Flexbox layout
- Grid layout with fr units
- Box model (margin, border, padding)
- Dock layout (top/bottom/left/right)
- Scroll regions with overflow handling

**Milestone 2 Deliverable**: All widgets styled via .tcss files. Theme switching works. Live reload during development.

---

## Milestone 3: Compositor & Advanced Rendering

**Goal**: Full compositing with overlapping widgets, correct Unicode handling.

### Phase 3.1: Compositor Core
- Layer management (widget_id, region, z_index, segments)
- Cut-finding algorithm (x-offsets at widget edges)
- Chop extraction per cut region
- Z-order selection for overlapping regions
- Concatenation into final segment list per line

### Phase 3.2: Clipping & Scrolling
- Clipping to parent boundaries
- Scroll offset handling in compositor
- Virtual scrolling support
- Dirty region tracking for partial updates

### Phase 3.3: Unicode & Double-Width
- Unicode width tables (unicode-width crate)
- Grapheme cluster handling (unicode-segmentation)
- Double-width character split at cut boundaries
- CJK character rendering
- Emoji sequence handling
- Zero-width character support

### Phase 3.4: Modal & Overlay Rendering
- Modal overlay with dim/fade effect
- Toast notification overlay
- Tooltip positioning
- Screen stack (push/pop/modal)

**Milestone 3 Deliverable**: Modal dialogs overlay correctly. CJK text renders properly. Smooth scrolling. No visual glitches.

---

## Milestone 4: Widget Library

**Goal**: All Tier 1 + Tier 2 widgets. Everything needed for saorsa-app.

### Phase 4.1: Text Widgets
- TextArea with tree-sitter syntax highlighting
- Incremental parsing on edit
- Selection, undo/redo, soft wrap, line numbers
- Autocomplete overlay
- Streaming Markdown renderer (incremental, cache previous blocks)

### Phase 4.2: Data Widgets
- RichLog (scrollable log with rich content)
- SelectList with fuzzy filtering
- DataTable (sortable, scrollable, column resizing)
- Tree / DirectoryTree (expandable, lazy loading)
- DiffView (side-by-side + unified)

### Phase 4.3: UI Widgets
- Tabs (tabbed content switcher)
- ProgressBar (determinate + indeterminate)
- LoadingIndicator (animated spinner/dots)
- Modal dialog
- Toast notifications
- Collapsible sections
- Switch, RadioButton, Checkbox
- OptionList
- Sparkline

**Milestone 4 Deliverable**: Complete widget library sufficient for the full saorsa-app UI.

---

## Milestone 5: Reactive System

**Goal**: Signal-based reactivity. Change a value, UI updates automatically.

### Phase 5.1: Signals & Computed
- Signal<T> with watchers
- Computed/derived values
- Side effects on state change
- Automatic re-render triggering

### Phase 5.2: Data Binding
- Widget property binding to data sources
- Bidirectional binding (input <-> model)
- Batch updates (coalesce multiple changes)

**Milestone 5 Deliverable**: Changing a signal value automatically updates all dependent widgets. No manual refresh needed.

---

## Milestone 6: Full Agent Features

**Goal**: Complete feature parity with pi. Single binary, zero runtime dependencies.

### Phase 6.1: Additional Providers
- OpenAI (Completions + Responses API)
- Google Gemini
- Azure OpenAI, Bedrock, Mistral, Groq, Cerebras, xAI, OpenRouter
- Ollama (local inference)
- **In-process local inference (GGUF): mistralrs provider (feature-gated)**
  - Add `saorsa-ai` provider impls: `Provider` + `StreamingProvider`
  - Backed by `mistralrs::Model::stream_chat_request`
  - Feature flag: `features = ["mistralrs"]` so `mistralrs`/`candle` deps are optional
  - API:
    - `pub struct MistralrsProvider { model: Arc<mistralrs::Model>, config: MistralrsConfig }`
    - `pub struct MistralrsConfig { temperature: f64, top_p: f64 }` (stop seqs/tools later)
    - Constructor takes an already-loaded `Arc<Model>` (apps manage download/load)
  - Streaming behavior (MVP text-only):
    - Emit `StreamEvent::MessageStart`
    - Emit one `ContentBlockStart(Text)` then `ContentBlockDelta(TextDelta)` per token
    - Emit `MessageDelta(stop_reason=EndTurn)` then `MessageStop`
    - Provider metadata: “no tools supported” for now
  - Acceptance:
    - Example or test that runs `AgentLoop::new(Box::new(MistralrsProvider), ...)` and streams a prompt
    - `cargo test` passes with and without the `mistralrs` feature enabled
- GitHub Copilot (OAuth)
- Custom OpenAI-compatible providers (models.json)

### Phase 6.2: Full Tool Suite
- Read tool (file reading with line ranges)
- Write tool (file writing with diff display)
- Edit tool (surgical editing with ambiguity detection)
- Grep, Find, Ls tools (via skills)

### Phase 6.3: Session Management
- Tree-structured session storage
- Auto-save, continue (-c), resume (-r), ephemeral
- /tree command with navigation
- Branching, forking, bookmarks
- HTML export, GitHub gist sharing

### Phase 6.4: Context Engineering
- AGENTS.md discovery (global + parent dirs + CWD)
- SYSTEM.md (replace/append system prompt)
- Context compaction (auto + manual /compact)
- Skills system (on-demand capabilities)
- Prompt templates

### Phase 6.5: Full UI Features
- Model selector (Ctrl+L, fuzzy search, favourites cycling)
- All slash commands (/model, /thinking, /compact, /tree, /fork, /export, /share, /login, /logout, /settings, /hotkeys, /clear, /help)
- Settings screen
- Message queuing (steering + follow-up)
- Multi-line editor with autocomplete (@files, /commands)
- Operating modes (interactive, print, JSON, RPC)
- Keybinding customization

### Phase 6.6: Extension System
- WASM extension loading (wasmtime)
- Lifecycle hooks (tool_call, message, turn_start, turn_end)
- Custom tool/command/keybinding registration
- Custom UI widgets and overlays
- Package management (install, update, list, config)

**Milestone 6 Deliverable**: Complete pi feature parity. All providers, tools, session management, extensions working.

---

## Milestone 7: Polish & Release

**Goal**: Production-quality release. Performance validated. Docs complete. CI/CD pipeline.

### Phase 7.1: Testing & Quality
- Snapshot testing for all widgets and screens (insta/SVG)
- Property-based tests for layout engine and CSS parser
- Integration tests with mock LLM responses
- Performance benchmarks (criterion)

### Phase 7.2: Terminal Compatibility
- iTerm2, Kitty, Alacritty, WezTerm, Terminal.app
- Windows Terminal, ConPTY
- tmux, screen, Zellij multiplexers
- Color mode fallback testing

### Phase 7.3: Themes & Documentation
- Theme library (Catppuccin, Dracula, Solarized, Nord)
- Complete API documentation
- User guide / README
- Architecture documentation
- Examples and tutorials

### Phase 7.4: CI/CD & Releases
- GitHub Actions pipeline (fmt, clippy, test, audit)
- Binary releases: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows
- Cross-compilation with cargo-zigbuild
- crates.io publishing

**Milestone 7 Deliverable**: v0.1.0 release. Published to crates.io. Binary releases for all platforms.

---

## Milestone 8: Comprehensive README Documentation

**Goal**: Every crate has a production-ready README.md with architecture, API reference, examples, and getting-started guides.

### Phase 8.1: saorsa-core README
- Overview and architecture (retained-mode, CSS-styled TUI framework)
- Quick start example (basic widget rendering)
- Widget catalog (all 24+ widgets with descriptions)
- TCSS guide (Terminal CSS syntax, properties, theming)
- Reactive system guide (signals, computed, effects)
- Layout engine guide (flexbox/grid via Taffy)
- Compositor and rendering pipeline
- Terminal backend abstraction
- Testing guide (snapshots, property-based, integration)

### Phase 8.2: saorsa-ai README
- Overview and architecture (unified multi-provider LLM API)
- Quick start for each provider (Anthropic, OpenAI, Gemini, Ollama)
- Streaming vs non-streaming usage
- Tool calling examples
- Model metadata and capabilities
- OpenAI-compatible provider setup (Azure, Groq, Mistral, xAI)
- Error handling reference

### Phase 8.3: Update saorsa-agent README
- Refresh existing README
- Add context engineering system documentation
- Add session management documentation
- Add extension system documentation
- Add skills and templates documentation

### Phase 8.4: saorsa-app README
- Overview (AI coding agent TUI application)
- Installation and setup
- CLI argument reference
- Slash command reference (all 13 commands)
- Keybinding reference
- Session management user guide
- Architecture overview

### Phase 8.5: saorsa-cli README + Root README
- saorsa-cli README (installation, basic usage)
- Root workspace README (project overview, architecture diagram, dependency graph, quick start, dev guide)

**Milestone 8 Deliverable**: All 5 crate READMEs + workspace root README complete and production-ready.

---

## Milestone 9: UX Overhaul — Performance, Scrollback, Commands & Model Management

**Goal**: Fix input responsiveness, add scrollback, make slash commands functional, add model management, wire up existing unused widgets.

### Phase 9.1: Render Throttling
- Add 30fps frame cap to render_ui() using Instant tracking
- Batch TextDelta events during AI streaming (accumulate, render at frame boundary)
- Mark frames dirty to skip no-op renders
- **Files:** main.rs, ui.rs

### Phase 9.2: Scrollback
- Add `scroll_offset: usize` and `auto_scroll: bool` to AppState
- Add InputAction variants: ScrollUp, ScrollDown, PageUp, PageDown, ScrollToBottom
- Handle PageUp/PageDown keys and mouse scroll wheel in input.rs
- Update render_messages() in ui.rs to use scroll_offset
- Auto-scroll to bottom on new messages (unless user has scrolled up)
- **Files:** app.rs, input.rs, ui.rs, main.rs

### Phase 9.3: Non-blocking Input During Streaming
- Allow scroll keys while AI is streaming (currently all input returns None when !idle)
- Allow Escape to cancel/interrupt agent run
- Allow Ctrl+P model switch to take effect on next interaction
- **Files:** input.rs, main.rs

### Phase 9.4: Command Dispatch
- Add `InputAction::Command(String, String)` variant (name + args)
- Detect `/` prefix in Submit handler before sending to agent
- Route to command modules: parse first word as command, rest as args
- Display command output as system message
- Handle unknown commands with error message
- **Files:** input.rs, main.rs, commands/mod.rs

### Phase 9.5: Functional Slash Commands
- `/model` - No args: list available models with current highlighted. With arg: switch model
- `/clear` - Clear message history and reset scroll
- `/thinking [off|low|medium|high]` - Set thinking level, persist to settings
- `/compact` - Toggle compact rendering mode
- `/help` - Dynamic command list with descriptions
- `/hotkeys` - Show keybindings
- `/config` - Show config paths and current settings summary
- Refactor commands to take `&mut AppState` instead of returning strings
- **Files:** All commands/*.rs, app.rs, main.rs

### Phase 9.6: Model Management
- `--show-models` CLI flag: list all known models with provider, context window, pricing
- `/model` interactive: show enabled models, allow toggling active/inactive for Ctrl+P cycling
- `/model add <provider> <key>` - Add API key for a new provider
- `/model enable/disable <name>` - Toggle model in Ctrl+P rotation
- Persist enabled models to ~/.saorsa/settings.json
- **Files:** cli.rs, commands/model.rs, app.rs, config/settings.rs

### Phase 9.7: New Slash Commands
- `/providers` - List configured providers with auth status
- `/cost` - Show session cost breakdown (uses CostTracker)
- `/agents` - List available agent tools
- `/skills` - List available skills (scan ~/.saorsa/skills/)
- `/status` - Show session info (model, provider, messages, tokens)
- **Files:** New command files, commands/mod.rs

### Phase 9.8: Widget Integration
- Wire ModelSelector widget to Ctrl+L (full interactive picker with fuzzy search)
- Wire SettingsScreen widget to /settings command
- Add OperatingMode to AppState for overlay management (Normal, ModelSelector, Settings)
- Route key events to active overlay widget when in overlay mode
- **Files:** app.rs, main.rs, ui.rs, input.rs

### Phase 9.9: Autocomplete Integration
- Instantiate Autocomplete in AppState
- Handle Tab key in input.rs for command/file completion
- Render suggestion popup in ui.rs
- Populate file paths from working directory
- **Files:** app.rs, input.rs, ui.rs, autocomplete.rs

**Milestone 9 Deliverable**: Responsive TUI with scrollback, working slash commands, model management, and integrated widgets.

---

## Architecture Decision Records

| ADR | Decision | Rationale |
|-----|----------|-----------|
| 001 | Retained mode | CSS styling, reactive updates, compositing, higher-level programming model |
| 002 | TCSS engine | Theming, live reloading, separation of presentation from logic |
| 003 | WASM extensions | Sandboxed, language-agnostic, Lua fallback if needed for V1 |
| 004 | Taffy for layout | Proven CSS Flexbox/Grid from Servo project |
| 005 | Single binary | Zero-dependency deployment, cross-platform |
