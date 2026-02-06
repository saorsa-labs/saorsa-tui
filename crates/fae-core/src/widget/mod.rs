//! Widget traits and built-in widgets.

pub mod container;
pub mod label;
pub mod static_widget;

pub use container::{BorderStyle, Container};
pub use label::{Alignment, Label};
pub use static_widget::StaticWidget;

use crate::buffer::ScreenBuffer;
use crate::event::Event;
use crate::geometry::Rect;

/// Result of handling an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventResult {
    /// The event was consumed by this widget.
    Consumed,
    /// The event was not handled; propagate to parent.
    Ignored,
}

/// A widget that can render itself into a screen buffer region.
pub trait Widget {
    /// Render this widget into the given area of the buffer.
    fn render(&self, area: Rect, buf: &mut ScreenBuffer);
}

/// A widget with size preferences for layout.
pub trait SizedWidget: Widget {
    /// The minimum size this widget needs.
    fn min_size(&self) -> (u16, u16);
    /// The preferred size if space is available.
    fn preferred_size(&self) -> (u16, u16) {
        self.min_size()
    }
}

/// A widget that can handle input events.
pub trait InteractiveWidget: Widget {
    /// Handle an input event. Returns whether the event was consumed.
    fn handle_event(&mut self, event: &Event) -> EventResult;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::geometry::{Rect, Size};
    use crate::style::Style;

    /// A mock widget for testing the trait.
    struct MockWidget {
        text: String,
    }

    impl Widget for MockWidget {
        fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
            for (i, ch) in self.text.chars().enumerate() {
                let x = area.position.x + i as u16;
                if x < area.position.x + area.size.width {
                    buf.set(x, area.position.y, Cell::new(ch.to_string(), Style::default()));
                }
            }
        }
    }

    #[test]
    fn mock_widget_renders() {
        let w = MockWidget {
            text: "hi".into(),
        };
        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        w.render(Rect::new(0, 0, 10, 1), &mut buf);
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("h"));
        assert_eq!(buf.get(1, 0).map(|c| c.grapheme.as_str()), Some("i"));
    }

    #[test]
    fn event_result_equality() {
        assert_eq!(EventResult::Consumed, EventResult::Consumed);
        assert_ne!(EventResult::Consumed, EventResult::Ignored);
    }
}
