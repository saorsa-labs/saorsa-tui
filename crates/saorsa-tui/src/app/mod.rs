//! Framework runtime: DOM tree + TCSS + layout + rendering + event dispatch.
//!
//! This module is the start of a "Textual-equivalent" retained-mode runtime.
//! It owns the widget tree, computes styles (TCSS), runs layout (Taffy),
//! dispatches input events, and renders frames via [`RenderContext`].

mod dom;
mod node_widget;
mod runtime;

pub use dom::{Dom, NodeId, NodeRef};
pub use node_widget::{Interactive, Leaf, NodeWidget, StyledInteractive, StyledLeaf};
pub use runtime::App;
