//! fae-core: A retained-mode, CSS-styled terminal UI framework.
//!
//! This crate provides the core rendering pipeline, layout engine,
//! CSS styling system, and widget infrastructure for building
//! rich terminal user interfaces.

pub mod buffer;
pub mod cell;
pub mod color;
pub mod error;
pub mod event;
pub mod focus;
pub mod geometry;
pub mod layout;
pub mod render_context;
pub mod renderer;
pub mod segment;
pub mod style;
pub mod terminal;
pub mod widget;

pub use buffer::{CellChange, ScreenBuffer};
pub use cell::Cell;
pub use color::Color;
pub use error::{FaeCoreError, Result};
pub use event::{Event, KeyCode, KeyEvent, Modifiers, MouseEvent};
pub use focus::{FocusManager, FocusState, WidgetId};
pub use geometry::{Position, Rect, Size};
pub use layout::{Constraint, Direction, Dock, Layout};
pub use render_context::RenderContext;
pub use renderer::Renderer;
pub use segment::Segment;
pub use style::Style;
pub use terminal::{CrosstermBackend, Terminal, TestBackend};
pub use widget::{Alignment, BorderStyle, Container, EventResult, Label, StaticWidget, Widget};
