# Fae - Project Guidelines

## Overview

Fae is a retained-mode, CSS-styled terminal UI framework in Rust, with a full AI coding agent as the showcase application.

### Crates

| Crate | Type | Purpose |
|-------|------|---------|
| `fae-core` | lib | TUI framework: rendering, layout, CSS, widgets, compositor |
| `fae-ai` | lib | Unified multi-provider LLM API (Anthropic, OpenAI, etc.) |
| `fae-agent` | lib | Agent runtime: tool execution, sessions, context engineering |
| `fae-app` | bin+lib | The AI coding agent application |
| `fae-cli` | bin | Thin CLI entry point |

### Dependency Graph

```
fae-cli -> fae-app -> fae-core
                   -> fae-ai
                   -> fae-agent -> fae-ai
```

## Build Commands

```bash
cargo check --workspace                              # Type check
cargo clippy --workspace --all-targets -- -D warnings # Lint (zero warnings)
cargo fmt --all -- --check                            # Format check
cargo test --workspace                                # Run tests
cargo doc --workspace --no-deps                       # Build docs
```

## Quality Standards

- **Zero warnings** from clippy and rustc
- **Zero test failures**
- **No `.unwrap()` or `.expect()`** in production code (OK in tests)
- **No `panic!()`, `todo!()`, `unimplemented!()`** anywhere
- **Doc comments** on all public items
- **`thiserror`** for library error types, **`anyhow`** in application binaries
- Error type per crate: `FaeCoreError`, `FaeAiError`, `FaeAgentError`

## Architecture

- **Retained mode**: Widgets persist in a tree, framework handles diffing
- **CSS styling**: TCSS (Terminal CSS) for theming and layout
- **Compositor**: Overlapping widgets with z-ordering, clipping, scroll offsets
- **Reactive**: Signal-based state management with automatic re-renders
- **Segments**: `Segment` (styled text) is the fundamental rendering unit
- **Cells**: `Cell` represents a single terminal cell with grapheme + style + width
- **Screen buffer**: Double-buffered with differential rendering

## Key Types (fae-core)

- `Segment` - Styled text piece (text + style + control flag)
- `Cell` - Single terminal cell (grapheme + style + width)
- `Style` - Text attributes (fg, bg, bold, italic, etc.)
- `Color` - Rgb, Indexed, Named, Reset
- `Rect`, `Position`, `Size` - Geometry primitives
- `Terminal` trait - Backend abstraction (CrosstermBackend, TestBackend)

## License

MIT OR Apache-2.0
