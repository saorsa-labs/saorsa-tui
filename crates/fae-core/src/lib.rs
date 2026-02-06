//! fae-core: A retained-mode, CSS-styled terminal UI framework.
//!
//! This crate provides the core rendering pipeline, layout engine,
//! CSS styling system, and widget infrastructure for building
//! rich terminal user interfaces.

pub mod cell;
pub mod color;
pub mod error;
pub mod geometry;
pub mod segment;
pub mod style;
pub mod terminal;

pub use cell::Cell;
pub use color::Color;
pub use error::{FaeCoreError, Result};
pub use geometry::{Position, Rect, Size};
pub use segment::Segment;
pub use style::Style;
pub use terminal::{CrosstermBackend, Terminal, TestBackend};
