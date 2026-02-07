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

## Architecture Decision Records

| ADR | Decision | Rationale |
|-----|----------|-----------|
| 001 | Retained mode | CSS styling, reactive updates, compositing, higher-level programming model |
| 002 | TCSS engine | Theming, live reloading, separation of presentation from logic |
| 003 | WASM extensions | Sandboxed, language-agnostic, Lua fallback if needed for V1 |
| 004 | Taffy for layout | Proven CSS Flexbox/Grid from Servo project |
| 005 | Single binary | Zero-dependency deployment, cross-platform |
