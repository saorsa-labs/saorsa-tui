//! fae-core: A retained-mode, CSS-styled terminal UI framework.
//!
//! This crate provides the core rendering pipeline, layout engine,
//! CSS styling system, and widget infrastructure for building
//! rich terminal user interfaces.

pub mod buffer;
pub mod cell;
pub mod color;
pub mod error;
pub mod geometry;
pub mod render_context;
pub mod renderer;
pub mod segment;
pub mod style;
pub mod terminal;

pub use buffer::{CellChange, ScreenBuffer};
pub use cell::Cell;
pub use color::Color;
pub use error::{FaeCoreError, Result};
pub use geometry::{Position, Rect, Size};
pub use render_context::RenderContext;
pub use renderer::Renderer;
pub use segment::Segment;
pub use style::Style;
pub use terminal::{CrosstermBackend, Terminal, TestBackend};
