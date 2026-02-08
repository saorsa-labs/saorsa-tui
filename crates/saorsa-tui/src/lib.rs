//! saorsa-core: A retained-mode, CSS-styled terminal UI framework.
//!
//! This crate provides the core rendering pipeline, layout engine,
//! CSS styling system, and widget infrastructure for building
//! rich terminal user interfaces.
//!
//! # Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Application Layer                        │
//! │  (Widget tree, CSS styles, reactive signals & bindings)     │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Layout Engine (Taffy)                      │
//! │  TCSS → ComputedStyle → taffy::Style → computed rects       │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │               Widget Rendering System                       │
//! │  Widget::render() → Vec<Segment> → Lines of styled text     │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │          Compositor (Layers, Z-ordering, Clipping)          │
//! │  Base layer + overlays → CompositorRegion → final buffer    │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │         Renderer (Differential, SGR optimization)           │
//! │  ScreenBuffer → DeltaBatch → optimized escape sequences     │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │               Terminal Backend (Crossterm)                  │
//! │  Raw mode, cursor control, alternate screen, events         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Core Subsystems
//!
//! - **TCSS Parser**: CSS-like styling with variables, pseudo-classes, and themes
//! - **Layout Engine**: Flexbox and grid layout via Taffy, scroll management
//! - **Reactive System**: Signal-based state with computed values, effects, and bindings
//! - **Compositor**: Layer-based rendering with z-ordering, clipping, and overlays
//! - **Widget Library**: Rich set of data, text, and UI widgets (tables, trees, markdown, etc.)
//! - **Renderer**: Double-buffered differential rendering with SGR optimization
//!
//! ## Key Types
//!
//! - `Segment`: Fundamental rendering unit (styled text + control flags)
//! - `Cell`: Single terminal cell (grapheme cluster + style + display width)
//! - `ScreenBuffer`: Double-buffered grid of cells with delta tracking
//! - `Widget`: Trait for renderable UI components
//! - `Signal<T>`: Reactive state container with automatic dependency tracking
//! - `Compositor`: Manages layers and composition into final screen buffer

pub mod app;
pub mod buffer;
pub mod cell;
pub mod color;
pub mod compositor;
pub mod cursor;
pub mod error;
pub mod event;
pub mod focus;
pub mod geometry;
pub mod highlight;
pub mod layout;
pub mod overlay;
pub mod reactive;
pub mod render_context;
pub mod renderer;
pub mod segment;
pub mod style;
pub mod tcss;
pub mod terminal;
pub mod text;
pub mod text_buffer;
pub mod undo;
pub mod viewport;
pub mod widget;
pub mod wrap;

#[cfg(test)]
mod test_env;

pub use buffer::{CellChange, ScreenBuffer};
pub use cell::Cell;
pub use color::Color;
pub use compositor::{Compositor, CompositorError, CompositorRegion, Layer};
pub use cursor::{CursorPosition, CursorState, Selection};
pub use error::{Result, SaorsaTuiError};
pub use event::{Event, KeyCode, KeyEvent, Modifiers, MouseEvent};
pub use focus::{FocusManager, FocusState, WidgetId};
pub use geometry::{Position, Rect, Size};
pub use highlight::{HighlightSpan, Highlighter, NoHighlighter, SimpleKeywordHighlighter};
pub use layout::{
    Constraint, Direction, Dock, Layout, LayoutEngine, LayoutError, LayoutRect, OverflowBehavior,
    ScrollManager, ScrollState,
};
pub use overlay::{OverlayConfig, OverlayId, OverlayPosition, Placement, ScreenStack};
pub use reactive::{
    Binding, BindingDirection, BindingExpression, BindingId, BindingScope, Computed, Effect,
    OneWayBinding, PropertySink, ReactiveScope, Signal, TwoWayBinding, batch,
};
pub use render_context::RenderContext;
pub use renderer::{DeltaBatch, Renderer, batch_changes, build_sgr_sequence};
pub use segment::Segment;
pub use style::Style;
pub use terminal::{
    CrosstermBackend, MultiplexerKind, Terminal, TerminalCapabilities, TerminalInfo, TerminalKind,
    TestBackend, detect, detect_multiplexer, detect_terminal, merge_multiplexer_limits,
    profile_for,
};
pub use text::{
    TextConfig, expand_tabs, filter_control_chars, preprocess, string_display_width,
    truncate_to_char_boundary, truncate_to_display_width,
};
pub use text_buffer::TextBuffer;
pub use undo::{EditOperation, UndoStack};
pub use viewport::Viewport;
pub use widget::{
    Alignment, BorderStyle, Checkbox, Collapsible, Column, Container, DataTable, DiffMode,
    DiffView, DirectoryTree, EventResult, IndicatorStyle, Label, LoadingIndicator,
    MarkdownRenderer, Modal, OptionList, ProgressBar, ProgressMode, RadioButton, RichLog,
    SelectList, Sparkline, StaticWidget, Switch, Tab, TabBarPosition, Tabs, TextArea, Toast,
    ToastPosition, Tooltip, Tree, TreeNode, Widget,
};
pub use wrap::{WrapLine, WrapResult, line_number_width, wrap_line, wrap_lines};
