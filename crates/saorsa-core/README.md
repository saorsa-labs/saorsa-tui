# saorsa-core

A retained-mode, CSS-styled terminal UI framework for Rust.

[![Crates.io](https://img.shields.io/crates/v/saorsa-core.svg)](https://crates.io/crates/saorsa-core)
[![Documentation](https://docs.rs/saorsa-core/badge.svg)](https://docs.rs/saorsa-core)
[![License](https://img.shields.io/crates/l/saorsa-core.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](#minimum-supported-rust-version)

## Overview

**saorsa-core** is a full-featured TUI framework that brings web-like development patterns to the terminal:

- **Retained-mode rendering** - Widgets persist in a tree; the framework handles diffing and efficient updates
- **CSS-styled** - Style everything with TCSS (Terminal CSS), including variables, themes, and live hot-reload
- **Reactive state** - Signal-based reactivity with automatic dependency tracking and batch updates
- **Rich widget library** - 24+ built-in widgets: tables, trees, markdown, diffs, modals, sparklines, and more
- **Compositor** - Layer-based rendering with z-ordering, clipping, and overlay support
- **Differential rendering** - Double-buffered with SGR-optimized escape sequences; only changed cells are written
- **Full Unicode support** - Grapheme clusters, CJK wide characters, emoji sequences, combining marks

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│  (Widget tree, CSS styles, reactive signals & bindings)      │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                  Layout Engine (Taffy)                       │
│  TCSS → ComputedStyle → taffy::Style → computed rects        │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│               Widget Rendering System                        │
│  Widget::render() → Vec<Segment> → Lines of styled text      │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│          Compositor (Layers, Z-ordering, Clipping)           │
│  Base layer + overlays → CompositorRegion → final buffer     │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│         Renderer (Differential, SGR optimization)            │
│  ScreenBuffer → DeltaBatch → optimized escape sequences      │
└─────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│               Terminal Backend (Crossterm)                    │
│  Raw mode, cursor control, alternate screen, events          │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

Add saorsa-core to your `Cargo.toml`:

```toml
[dependencies]
saorsa-core = "0.1"
```

Render a styled label into a screen buffer:

```rust
use saorsa_core::{
    Color, Label, Rect, ScreenBuffer, Segment, Size, Style, Widget,
};

// Create a screen buffer (80x24 terminal)
let size = Size::new(80, 24);
let mut buf = ScreenBuffer::new(size);

// Create a styled label
let style = Style::new()
    .fg(Color::Named(saorsa_core::color::NamedColor::Cyan))
    .bold(true);
let label = Label::new("Hello, saorsa!")
    .style(style);

// Render into a region
let area = Rect::new(0, 0, 80, 1);
label.render(area, &mut buf);
```

Split a terminal area into layout regions:

```rust
use saorsa_core::{Constraint, Direction, Layout, Rect};

let area = Rect::new(0, 0, 80, 24);

// Vertical layout: 3-line header, fill for content, 1-line footer
let regions = Layout::split(area, Direction::Vertical, &[
    Constraint::Fixed(3),
    Constraint::Fill,
    Constraint::Fixed(1),
]);
// regions[0] = header area (80x3)
// regions[1] = content area (80x20)
// regions[2] = footer area (80x1)
```

## Widget Catalog

### Text Widgets

| Widget | Description |
|--------|-------------|
| **`Label`** | Single-line styled text with alignment (left, center, right) |
| **`StaticWidget`** | Renders pre-built `Vec<Segment>` directly |
| **`TextArea`** | Multi-line editable text with undo/redo, selection, and soft wrap (Ropey-based) |
| **`RichLog`** | Scrollable log viewer with syntax-highlighted entries |
| **`MarkdownRenderer`** | Markdown to styled terminal output (via pulldown-cmark) |
| **`DiffView`** | Side-by-side or unified diff display (via similar) |

### Data Widgets

| Widget | Description |
|--------|-------------|
| **`DataTable`** | Scrollable table with sortable columns, row selection, and keyboard navigation |
| **`Tree`** | Hierarchical tree with expand/collapse and keyboard navigation |
| **`DirectoryTree`** | Filesystem tree navigator with lazy loading |
| **`SelectList`** | Searchable selection list with fuzzy filtering |
| **`OptionList`** | Radio-style option selector |

### UI Widgets

| Widget | Description |
|--------|-------------|
| **`Container`** | Layout container with optional titled border |
| **`Modal`** | Centered modal dialog with overlay dimming |
| **`Toast`** | Notification popup with configurable position and timeout |
| **`Tooltip`** | Contextual tooltip anchored to a position |
| **`Tabs`** | Multi-tab interface with configurable tab bar position |
| **`Collapsible`** | Expandable/collapsible section with header |
| **`ProgressBar`** | Determinate and indeterminate progress display |
| **`LoadingIndicator`** | Animated spinner with multiple styles (dots, braille, line, arc) |
| **`Sparkline`** | Inline data visualization with bar characters |

### Form Controls

| Widget | Description |
|--------|-------------|
| **`Checkbox`** | Toggle checkbox (`[x]` / `[ ]`) |
| **`RadioButton`** | Radio button (`(*)` / `( )`) |
| **`Switch`** | Toggle switch with on/off states |

### Widget Traits

All widgets implement the `Widget` trait. Interactive widgets additionally implement `InteractiveWidget`, and widgets with intrinsic dimensions implement `SizedWidget`:

```rust
/// Render into a screen buffer region.
pub trait Widget {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer);
}

/// Widget with size preferences for layout.
pub trait SizedWidget: Widget {
    fn min_size(&self) -> (u16, u16);
    fn preferred_size(&self) -> (u16, u16) { self.min_size() }
}

/// Widget that handles input events.
pub trait InteractiveWidget: Widget {
    fn handle_event(&mut self, event: &Event) -> EventResult;
}
```

## TCSS (Terminal CSS)

saorsa-core includes a full CSS engine adapted for terminals. Stylesheets are parsed using a Servo-derived `cssparser` backend.

### Selectors

```css
/* Type selector */
Label { color: white; }

/* Class selector */
.error { color: red; }

/* ID selector */
#sidebar { width: 20; }

/* Pseudo-classes */
Label:focus { color: green; }
Label:hover { background: blue; }
Label:disabled { dim: true; }

/* Child combinator */
Container > Label { margin: 1; }

/* Descendant combinator */
Container Label { padding: 2; }

/* Adjacent sibling */
Label + Container { margin-top: 1; }
```

### Properties

**Colors & Text**

| Property | Values | Description |
|----------|--------|-------------|
| `color` | Named, `#rgb`, `#rrggbb`, indexed | Foreground color |
| `background` | Named, `#rgb`, `#rrggbb`, indexed | Background color |
| `border-color` | Named, `#rgb`, `#rrggbb`, indexed | Border color |
| `text-style` | `bold`, `italic`, `underline`, `strikethrough`, `dim`, `reverse` | Text decorations |
| `text-align` | `left`, `center`, `right` | Horizontal text alignment |
| `content-align` | `top`, `middle`, `bottom` | Vertical content alignment |

**Dimensions**

| Property | Values | Description |
|----------|--------|-------------|
| `width` / `height` | Integer, percentage, `auto` | Widget dimensions |
| `min-width` / `min-height` | Integer | Minimum dimensions |
| `max-width` / `max-height` | Integer, `none` | Maximum dimensions |

**Box Model**

| Property | Values | Description |
|----------|--------|-------------|
| `margin` | 1-4 integers | Outer spacing |
| `margin-top/right/bottom/left` | Integer | Individual margins |
| `padding` | 1-4 integers | Inner spacing |
| `padding-top/right/bottom/left` | Integer | Individual padding |
| `border` | 1-4 values (`ascii`, `round`, `heavy`, `double`, `none`) | Border style |

**Flexbox Layout**

| Property | Values | Description |
|----------|--------|-------------|
| `display` | `flex`, `grid`, `block`, `none` | Display mode |
| `flex-direction` | `row`, `column`, `row-reverse`, `column-reverse` | Main axis |
| `flex-wrap` | `nowrap`, `wrap`, `wrap-reverse` | Wrapping behavior |
| `justify-content` | `flex-start`, `flex-end`, `center`, `space-between`, `space-around`, `space-evenly` | Main axis alignment |
| `align-items` | `flex-start`, `flex-end`, `center`, `stretch`, `baseline` | Cross axis alignment |
| `align-self` | `auto`, `flex-start`, `flex-end`, `center`, `stretch` | Individual cross alignment |
| `flex-grow` / `flex-shrink` | Number | Flex factors |
| `flex-basis` | Integer, `auto` | Initial size |
| `gap` | Integer | Gap between children |

**Grid Layout**

| Property | Values | Description |
|----------|--------|-------------|
| `grid-template-columns` | Sizes (integer, `fr`, `auto`) | Column track definitions |
| `grid-template-rows` | Sizes (integer, `fr`, `auto`) | Row track definitions |
| `grid-column` | `start / end` | Column placement |
| `grid-row` | `start / end` | Row placement |

**Positioning**

| Property | Values | Description |
|----------|--------|-------------|
| `dock` | `top`, `bottom`, `left`, `right` | Dock to edge |
| `overflow` | `visible`, `hidden`, `scroll`, `auto` | Overflow behavior |
| `overflow-x` / `overflow-y` | Same as `overflow` | Per-axis overflow |
| `visibility` | `visible`, `hidden` | Visibility |
| `opacity` | `0`-`1` | Opacity level |

### Variables & Theming

Define variables in `:root` or scoped selectors, reference with `$`:

```css
:root {
    $fg: white;
    $bg: #1e1e2e;
    $accent: #89b4fa;
    $border: #585b70;
}

.dark {
    $bg: #11111b;
}

Label {
    color: $fg;
    background: $bg;
}

Container {
    border-color: $border;
}
```

### Built-in Themes

saorsa-core ships with popular color schemes:

| Theme | Variants |
|-------|----------|
| **Catppuccin** | `catppuccin_latte`, `catppuccin_frappe`, `catppuccin_macchiato`, `catppuccin_mocha` |
| **Dracula** | `dracula_dark`, `dracula_light` |
| **Nord** | `nord_dark` |
| **Solarized** | `solarized_dark`, `solarized_light` |

### Live Hot-Reload

TCSS files can be watched for changes at runtime using the `notify`-based file watcher. When a stylesheet is modified, it is re-parsed and styles are re-applied without restarting the application.

## Layout Engine

### Manual Layout

Split areas using constraints:

```rust
use saorsa_core::{Constraint, Direction, Dock, Layout, Rect};

let terminal = Rect::new(0, 0, 120, 40);

// Split horizontally: 30-cell sidebar + fill for main content
let cols = Layout::split(terminal, Direction::Horizontal, &[
    Constraint::Fixed(30),
    Constraint::Fill,
]);

// Split the main content vertically
let rows = Layout::split(cols[1], Direction::Vertical, &[
    Constraint::Fixed(1),      // Status bar
    Constraint::Percentage(70), // Editor
    Constraint::Fill,           // Output panel
]);

// Dock a widget to the bottom
let (content, status_bar) = Layout::dock(terminal, Dock::Bottom, 1);
```

**Constraint types:**

| Constraint | Behavior |
|-----------|----------|
| `Fixed(n)` | Exactly `n` cells |
| `Min(n)` | At least `n` cells |
| `Max(n)` | At most `n` cells |
| `Percentage(p)` | `p`% of available space |
| `Fill` | Distribute remaining space equally among all `Fill` constraints |

### Taffy-Powered Flexbox & Grid

For complex layouts, saorsa-core integrates with [Taffy](https://github.com/DioxusLabs/taffy) (from the Servo/Dioxus project) for full CSS Flexbox and Grid support:

```rust
use saorsa_core::{LayoutEngine, LayoutRect};

let mut engine = LayoutEngine::new();

// Add nodes with Taffy styles (converted from TCSS ComputedStyle)
let root = engine.add_root(taffy_style);
let child_a = engine.add_child(root, child_a_style);
let child_b = engine.add_child(root, child_b_style);

// Compute layout for a given available space
engine.compute(120, 40);

// Retrieve computed rectangles
let rect_a: LayoutRect = engine.layout(child_a);
```

TCSS properties are automatically converted to Taffy styles via `computed_to_taffy()`.

### Scroll Management

```rust
use saorsa_core::{ScrollManager, ScrollState, OverflowBehavior};

let mut scroll_mgr = ScrollManager::new();
let widget_id = 42;

// Register a scroll region
scroll_mgr.register(widget_id, OverflowBehavior::Auto);

// Update when content changes
scroll_mgr.set_content_size(widget_id, 200, 500); // content 200w x 500h
scroll_mgr.set_viewport_size(widget_id, 80, 24);  // viewport 80w x 24h

// Scroll programmatically
scroll_mgr.scroll_to(widget_id, 0, 100);
```

## Reactive System

saorsa-core provides a fine-grained reactive system inspired by SolidJS. Changes to signals automatically propagate to computed values, effects, and bound widgets.

### Signals

A `Signal<T>` holds a mutable value. Reading it inside a tracking context records a dependency; setting it notifies all subscribers:

```rust
use saorsa_core::Signal;

let count = Signal::new(0);
assert_eq!(count.get(), 0);

count.set(5);
assert_eq!(count.get(), 5);

// Update with a closure
count.update(|n| *n += 1);
assert_eq!(count.get(), 6);
```

### Computed Values

A `Computed<T>` derives its value from one or more signals. It re-evaluates only when dependencies change:

```rust
use saorsa_core::{Signal, Computed};

let width = Signal::new(80);
let height = Signal::new(24);

let area = Computed::new(move || width.get() * height.get());
assert_eq!(area.get(), 1920);

width.set(120);
assert_eq!(area.get(), 2880); // Automatically recomputed
```

### Effects

An `Effect` runs a side-effect function whenever its dependencies change:

```rust
use saorsa_core::{Signal, Effect};

let theme = Signal::new("dark".to_string());

let _effect = Effect::new(move || {
    let current = theme.get();
    // This runs whenever `theme` changes
    println!("Theme changed to: {current}");
});
```

### Data Bindings

Bind signals to widget properties:

```rust
use saorsa_core::{Signal, OneWayBinding, TwoWayBinding};

let source = Signal::new(42);
let target = Signal::new(0);

// One-way: source → target
let _binding = OneWayBinding::new(source.clone(), /* property sink */);

// Two-way: changes propagate in both directions
let _binding = TwoWayBinding::new(source, target);
```

### Batch Updates

Coalesce multiple signal changes into a single notification pass:

```rust
use saorsa_core::{Signal, batch};

let x = Signal::new(0);
let y = Signal::new(0);

// Subscribers are notified only once, after the batch completes
batch(|| {
    x.set(10);
    y.set(20);
});
```

### Reactive Scopes

A `ReactiveScope` manages the lifetime of effects and subscriptions. When the scope is dropped, all its effects are cleaned up:

```rust
use saorsa_core::ReactiveScope;

let scope = ReactiveScope::new();
// Effects created within this scope are cleaned up when `scope` is dropped
```

## Compositor

The compositor manages overlapping widget layers and produces the final screen buffer.

### Layers

Each widget renders into a `Layer` with a position, z-index, and content:

```rust
use saorsa_core::{Compositor, Layer, Rect, ScreenBuffer, Size};

let mut compositor = Compositor::new(80, 24);

// Add a base layer
compositor.add_layer(Layer {
    widget_id: 1,
    region: Rect::new(0, 0, 80, 24),
    z_index: 0,
    lines: vec![/* segments per line */],
});

// Add a modal overlay on top
compositor.add_layer(Layer {
    widget_id: 2,
    region: Rect::new(20, 5, 40, 14),
    z_index: 10,
    lines: vec![/* modal content */],
});

// Compose all layers into the final buffer
let mut buf = ScreenBuffer::new(Size::new(80, 24));
compositor.compose(&mut buf);
```

### Composition Algorithm

1. **Cut finding** - Collects x-offsets at every layer edge to define vertical strips
2. **Chop extraction** - Extracts the segment slice from each layer for each strip
3. **Z-order selection** - For overlapping strips, the highest z-index layer wins
4. **Concatenation** - Merges selected chops into final segment lines

### Overlay System

The `ScreenStack` manages modal overlays, tooltips, and toasts:

```rust
use saorsa_core::{ScreenStack, OverlayConfig, Placement};

let mut stack = ScreenStack::new();

// Push a centered modal overlay
let id = stack.push(OverlayConfig {
    placement: Placement::Center,
    z_index: 100,
    ..Default::default()
}, content_lines);

// Pop when dismissed
stack.pop(id);
```

## Terminal Backends

### Backend Trait

All terminal I/O goes through the `Terminal` trait, making the framework backend-agnostic:

```rust
pub trait Terminal: Send {
    fn size(&self) -> Result<Size>;
    fn capabilities(&self) -> &TerminalCapabilities;
    fn enter_raw_mode(&mut self) -> Result<()>;
    fn exit_raw_mode(&mut self) -> Result<()>;
    fn write_raw(&mut self, data: &[u8]) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn enable_mouse(&mut self) -> Result<()>;
    fn disable_mouse(&mut self) -> Result<()>;
}
```

### Capability Detection

saorsa-core automatically detects the terminal emulator and its capabilities:

```rust
use saorsa_core::{detect, detect_terminal, TerminalKind, ColorSupport};

let info = detect();
println!("Terminal: {:?}", info.kind);
println!("Colors: {:?}", info.capabilities.color);
println!("Unicode: {}", info.capabilities.unicode);
println!("Synchronized output: {}", info.capabilities.synchronized_output);
```

**Detected terminals:** Alacritty, Kitty, WezTerm, iTerm2, Windows Terminal, GNOME Terminal, Konsole, Xterm, and more.

**Detected multiplexers:** tmux, screen, Zellij.

**Capabilities tracked:**

| Capability | Description |
|-----------|-------------|
| `color` | `NoColor`, `Basic16`, `Extended256`, `TrueColor` |
| `unicode` | Full Unicode grapheme support |
| `synchronized_output` | CSI ?2026 synchronized output |
| `kitty_keyboard` | Kitty keyboard protocol |
| `mouse` | Mouse event support |
| `bracketed_paste` | Bracketed paste mode |
| `focus_events` | Focus in/out notifications |
| `hyperlinks` | OSC 8 clickable hyperlinks |
| `sixel` | Sixel graphics protocol |

### Test Backend

For testing, use `TestBackend` which stores output in memory:

```rust
use saorsa_core::TestBackend;

let backend = TestBackend::new(80, 24);
// Use for snapshot testing and unit tests without a real terminal
```

## Rendering Pipeline

### Double Buffering

`ScreenBuffer` maintains a grid of `Cell` values. The renderer diffs the current buffer against the previous frame and only emits escape sequences for changed cells:

```rust
use saorsa_core::{ScreenBuffer, Size, CellChange, batch_changes, Renderer};

let prev = ScreenBuffer::new(Size::new(80, 24));
let curr = ScreenBuffer::new(Size::new(80, 24));
// ... render widgets into `curr` ...

// Compute delta
let changes: Vec<CellChange> = curr.diff(&prev);

// Batch adjacent changes for fewer cursor movements
let batches = batch_changes(&changes);

// Render to escape sequences
let renderer = Renderer::new(/* capabilities */);
let output = renderer.render_batched(&batches);
```

### SGR Optimization

The renderer minimizes escape sequence output:

- **Style diffing** - Only emits changed attributes (e.g., if bold is already on, it won't re-emit it)
- **SGR coalescing** - Combines multiple attributes into a single `\x1b[...m` sequence
- **Cursor tracking** - Skips cursor movement when the cursor is already at the target position
- **Continuation cells** - Skips zero-width continuation cells from wide characters
- **Synchronized output** - Wraps frame updates in CSI ?2026h/l to prevent tearing

## Core Types

### Segment

The fundamental rendering unit. A `Segment` is a piece of styled text:

```rust
use saorsa_core::{Segment, Style, Color};

// Plain text
let seg = Segment::new("Hello");

// Styled text
let style = Style::new().fg(Color::Rgb { r: 255, g: 100, b: 50 }).bold(true);
let seg = Segment::styled("Error:", style);

// Blank padding
let spacer = Segment::blank(10); // 10 spaces

// Control sequence (not rendered as visible text)
let ctrl = Segment::control("\x1b[?25l"); // hide cursor
```

### Cell

A single terminal cell. Stores one grapheme cluster, its style, and its display width:

```rust
use saorsa_core::Cell;

let cell = Cell::new("A", Style::default());
assert_eq!(cell.width, 1);

let wide = Cell::new("中", Style::default());
assert_eq!(wide.width, 2); // CJK character takes 2 columns
```

### Style

Builder-pattern text attributes:

```rust
use saorsa_core::{Style, Color};

let style = Style::new()
    .fg(Color::Rgb { r: 200, g: 200, b: 200 })
    .bg(Color::Indexed(236))
    .bold(true)
    .italic(true)
    .underline(true);
```

### Color

Four color modes with automatic downgrading based on terminal capabilities:

```rust
use saorsa_core::Color;
use saorsa_core::color::NamedColor;

let rgb = Color::Rgb { r: 255, g: 0, b: 128 };  // True color
let indexed = Color::Indexed(196);                 // 256-color palette
let named = Color::Named(NamedColor::BrightCyan);  // 16 ANSI colors
let reset = Color::Reset;                          // Terminal default

// Parse hex strings
let hex = Color::from_hex("#89b4fa").unwrap();
let short_hex = Color::from_hex("#f0c").unwrap();
```

## Unicode Support

saorsa-core handles the full range of Unicode correctly:

- **Grapheme clusters** - Characters with combining marks are kept together (via `unicode-segmentation`)
- **Wide characters** - CJK ideographs and some emoji occupy 2 terminal columns; continuation cells are tracked automatically
- **Emoji sequences** - ZWJ (zero-width joiner) families, flag sequences, and skin tone modifiers
- **Display width** - All width calculations use `unicode-width` for accurate column counts
- **Safe truncation** - Text truncation respects grapheme boundaries, never splitting a character
- **Tab expansion** - Configurable tab stops with proper column alignment
- **Control character filtering** - Non-printable characters are stripped or replaced

The `ScreenBuffer::set()` method automatically handles wide character edge cases: writing over a continuation cell blanks the preceding wide character, and writing a wide character at the buffer edge replaces it with a space.

## Testing

### Snapshot Testing

Widget rendering is verified with `insta` snapshot tests:

```rust
use saorsa_core::{Label, Rect, ScreenBuffer, Size, Widget, TestBackend};

#[test]
fn label_renders_correctly() {
    let mut buf = ScreenBuffer::new(Size::new(40, 1));
    let label = Label::new("Hello, world!");
    label.render(Rect::new(0, 0, 40, 1), &mut buf);

    insta::assert_snapshot!(buf.to_string());
}
```

Tests are organized by widget category in `tests/snapshot_*.rs`.

### Property-Based Testing

Layout and CSS parsing are fuzz-tested with `proptest`:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn layout_split_covers_full_area(
        width in 1u16..500,
        n_constraints in 1usize..10,
    ) {
        // Property: sum of split rects equals the input area
    }
}
```

Tests live in `tests/proptest_layout.rs` and `tests/proptest_css.rs`.

### Benchmarks

Performance-critical paths are benchmarked with `criterion`:

```bash
cargo bench -p saorsa-core
```

Benchmarks cover:
- **Rendering** - Cell diffing, SGR sequence generation, batch optimization
- **Layout** - Constraint solving, Taffy layout computation
- **CSS parsing** - Stylesheet parsing, selector matching, cascade resolution

## Error Handling

All fallible operations return `Result<T, SaorsaCoreError>`:

```rust
pub enum SaorsaCoreError {
    Io(std::io::Error),
    Terminal(String),
    Layout(String),
    Style(String),
    Render(String),
    Widget(String),
    Unicode(String),
    Reactive(String),
    Internal(String),
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `crossterm` | Terminal backend (events, raw mode, cursor) |
| `taffy` | CSS Flexbox and Grid layout engine (from Servo/Dioxus) |
| `cssparser` | CSS tokenizer and parser (from Servo) |
| `ropey` | Rope data structure for `TextArea` editing |
| `pulldown-cmark` | Markdown parsing for `MarkdownRenderer` |
| `similar` | Diff algorithm for `DiffView` |
| `fuzzy-matcher` | Fuzzy string matching for `SelectList` |
| `unicode-width` | Display width calculation |
| `unicode-segmentation` | Grapheme cluster segmentation |
| `notify` | Filesystem watcher for TCSS hot-reload |
| `tracing` | Structured logging |
| `thiserror` | Error type derivation |

## Minimum Supported Rust Version

The MSRV is **1.88** (Rust Edition 2024). This is enforced in CI.

## License

Licensed under either of:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contributing

Part of the [saorsa-tui](https://github.com/saorsa-labs/saorsa-tui) workspace. See the workspace root for contribution guidelines.
