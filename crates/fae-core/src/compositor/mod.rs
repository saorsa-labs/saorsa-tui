//! Compositor — resolves overlapping widget layers into a flat cell grid.
//!
//! The compositor collects styled segment output from each widget,
//! finds cut boundaries where widget edges meet, selects the topmost
//! visible widget for each region, and writes the result to a screen buffer.

pub mod layer;

pub use layer::{CompositorError, CompositorRegion, Layer};
