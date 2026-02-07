//! Widget traits and built-in widgets.

pub mod border;
pub mod container;
pub mod data_table;
pub mod diff_view;
pub mod directory_tree;
pub mod label;
pub mod markdown;
pub mod modal;
pub mod rich_log;
pub mod select_list;
pub mod static_widget;
pub mod text_area;
pub mod toast;
pub mod tooltip;
pub mod tree;

pub use container::{BorderStyle, Container};
pub use data_table::{Column, DataTable};
pub use diff_view::{DiffMode, DiffView};
pub use directory_tree::DirectoryTree;
pub use label::{Alignment, Label};
pub use markdown::MarkdownRenderer;
pub use modal::Modal;
pub use rich_log::RichLog;
pub use select_list::SelectList;
pub use static_widget::StaticWidget;
pub use text_area::TextArea;
pub use toast::{Toast, ToastPosition};
pub use tooltip::Tooltip;
pub use tree::{Tree, TreeNode};

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
                    buf.set(
                        x,
                        area.position.y,
                        Cell::new(ch.to_string(), Style::default()),
                    );
                }
            }
        }
    }

    #[test]
    fn mock_widget_renders() {
        let w = MockWidget { text: "hi".into() };
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

    // --- Widget module integration tests ---

    #[test]
    fn modal_create_and_render() {
        let modal = Modal::new("Test Modal", 30, 10);
        let lines = modal.render_to_lines();
        assert!(lines.len() == 10);
        assert!(!lines[0].is_empty());
    }

    #[test]
    fn toast_create_and_render() {
        let toast = Toast::new("Notification");
        let lines = toast.render_to_lines();
        assert!(lines.len() == 1);
        let text: String = lines[0].iter().map(|s| &*s.text).collect();
        assert!(text.contains("Notification"));
    }

    #[test]
    fn tooltip_create_and_render() {
        let tooltip = Tooltip::new("Help text", Rect::new(10, 10, 5, 2));
        let lines = tooltip.render_to_lines();
        assert!(lines.len() == 1);
        assert!(lines[0][0].text == "Help text");
    }

    #[test]
    fn modal_pushed_to_screen_stack() {
        use crate::overlay::ScreenStack;

        let modal = Modal::new("M", 20, 5);
        let lines = modal.render_to_lines();
        let config = modal.to_overlay_config();

        let mut stack = ScreenStack::new();
        let id = stack.push(config, lines);
        assert!(id > 0);
        assert!(stack.len() == 1);
    }

    #[test]
    fn toast_pushed_to_screen_stack() {
        use crate::overlay::ScreenStack;

        let toast = Toast::new("T").with_width(10);
        let screen = Size::new(80, 24);
        let lines = toast.render_to_lines();
        let config = toast.to_overlay_config(screen);

        let mut stack = ScreenStack::new();
        stack.push(config, lines);
        assert!(stack.len() == 1);
    }

    #[test]
    fn multiple_overlay_types_in_stack() {
        use crate::overlay::{Placement, ScreenStack};

        let mut stack = ScreenStack::new();
        let screen = Size::new(80, 24);

        // Add modal
        let modal = Modal::new("M", 20, 5);
        stack.push(modal.to_overlay_config(), modal.render_to_lines());

        // Add toast
        let toast = Toast::new("T").with_width(10);
        stack.push(toast.to_overlay_config(screen), toast.render_to_lines());

        // Add tooltip
        let tooltip = Tooltip::new("tip", Rect::new(10, 10, 5, 2)).with_placement(Placement::Below);
        stack.push(tooltip.to_overlay_config(screen), tooltip.render_to_lines());

        assert!(stack.len() == 3);
    }
}
