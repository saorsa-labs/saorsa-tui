//! fae-core: A retained-mode, CSS-styled terminal UI framework.
//!
//! This crate provides the core rendering pipeline, layout engine,
//! CSS styling system, and widget infrastructure for building
//! rich terminal user interfaces.

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

pub use buffer::{CellChange, ScreenBuffer};
pub use cell::Cell;
pub use color::Color;
pub use compositor::{Compositor, CompositorError, CompositorRegion, Layer};
pub use cursor::{CursorPosition, CursorState, Selection};
pub use error::{FaeCoreError, Result};
pub use event::{Event, KeyCode, KeyEvent, Modifiers, MouseEvent};
pub use focus::{FocusManager, FocusState, WidgetId};
pub use geometry::{Position, Rect, Size};
pub use highlight::{HighlightSpan, Highlighter, NoHighlighter, SimpleKeywordHighlighter};
pub use layout::{
    Constraint, Direction, Dock, Layout, LayoutEngine, LayoutError, LayoutRect, OverflowBehavior,
    ScrollManager, ScrollState,
};
pub use overlay::{OverlayConfig, OverlayId, OverlayPosition, Placement, ScreenStack};
pub use render_context::RenderContext;
pub use renderer::{DeltaBatch, Renderer, batch_changes, build_sgr_sequence};
pub use segment::Segment;
pub use style::Style;
pub use terminal::{CrosstermBackend, Terminal, TestBackend};
pub use text::{
    TextConfig, expand_tabs, filter_control_chars, preprocess, string_display_width,
    truncate_to_char_boundary, truncate_to_display_width,
};
pub use text_buffer::TextBuffer;
pub use undo::{EditOperation, UndoStack};
pub use viewport::Viewport;
pub use widget::{
    Alignment, BorderStyle, Column, Container, DataTable, DiffMode, DiffView, DirectoryTree,
    EventResult, IndicatorStyle, Label, LoadingIndicator, MarkdownRenderer, Modal, ProgressBar,
    ProgressMode, RichLog, SelectList, StaticWidget, Tab, TabBarPosition, Tabs, TextArea, Toast,
    ToastPosition, Tooltip, Tree, TreeNode, Widget,
};
pub use wrap::{WrapLine, WrapResult, line_number_width, wrap_line, wrap_lines};
