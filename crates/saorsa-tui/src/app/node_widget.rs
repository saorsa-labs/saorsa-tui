//! Trait-object wrappers for widgets stored in the DOM.
//!
//! We avoid specialization by using explicit wrapper types:
//! - [`Leaf`][]: render-only
//! - [`Interactive`][]: render + event handling
//! - [`StyledLeaf`][] / [`StyledInteractive`][]: plus TCSS computed-style application

use std::any::Any;

use crate::buffer::ScreenBuffer;
use crate::event::Event;
use crate::geometry::Rect;
use crate::tcss::ComputedStyle;
use crate::widget::EventResult;

/// Dynamic widget interface stored in the [`Dom`](super::Dom).
pub trait NodeWidget {
    /// Render this widget into `area`.
    fn render(&mut self, area: Rect, buf: &mut ScreenBuffer);

    /// Lifecycle: called when the node is attached to the DOM.
    fn on_mount(&mut self) {}

    /// Lifecycle: called right before the node is removed from the DOM.
    fn on_unmount(&mut self) {}

    /// Handle an event. Default is ignored.
    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }

    /// Apply TCSS computed style. Default is no-op.
    fn apply_computed_style(&mut self, _computed: &ComputedStyle) {}

    /// Downcast support.
    fn as_any(&self) -> &dyn Any;

    /// Downcast support (mutable).
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Render-only wrapper.
pub struct Leaf<T>(pub T);

impl<T> Leaf<T> {
    /// Wrap a render-only widget.
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> NodeWidget for Leaf<T>
where
    T: crate::widget::Widget + Any,
{
    fn render(&mut self, area: Rect, buf: &mut ScreenBuffer) {
        self.0.render(area, buf);
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}

/// Render + event handling wrapper.
pub struct Interactive<T>(pub T);

impl<T> Interactive<T> {
    /// Wrap an interactive widget.
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> NodeWidget for Interactive<T>
where
    T: crate::widget::Widget + crate::widget::InteractiveWidget + Any,
{
    fn render(&mut self, area: Rect, buf: &mut ScreenBuffer) {
        self.0.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        self.0.handle_event(event)
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}

/// Render-only + computed-style application wrapper.
pub struct StyledLeaf<T>(pub T);

impl<T> StyledLeaf<T> {
    /// Wrap a stylable widget.
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> NodeWidget for StyledLeaf<T>
where
    T: crate::widget::Widget + crate::tcss::ApplyComputedStyle + Any,
{
    fn render(&mut self, area: Rect, buf: &mut ScreenBuffer) {
        self.0.render(area, buf);
    }

    fn apply_computed_style(&mut self, computed: &ComputedStyle) {
        self.0.apply_computed_style(computed);
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}

/// Interactive + computed-style application wrapper.
pub struct StyledInteractive<T>(pub T);

impl<T> StyledInteractive<T> {
    /// Wrap a stylable interactive widget.
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> NodeWidget for StyledInteractive<T>
where
    T: crate::widget::Widget
        + crate::widget::InteractiveWidget
        + crate::tcss::ApplyComputedStyle
        + Any,
{
    fn render(&mut self, area: Rect, buf: &mut ScreenBuffer) {
        self.0.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        self.0.handle_event(event)
    }

    fn apply_computed_style(&mut self, computed: &ComputedStyle) {
        self.0.apply_computed_style(computed);
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut self.0
    }
}
