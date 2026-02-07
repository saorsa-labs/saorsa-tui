//! Widget traits and built-in widgets.

pub mod border;
pub mod collapsible;
pub mod container;
pub mod data_table;
pub mod diff_view;
pub mod directory_tree;
pub mod form_controls;
pub mod label;
pub mod loading_indicator;
pub mod markdown;
pub mod modal;
pub mod option_list;
pub mod progress_bar;
pub mod rich_log;
pub mod select_list;
pub mod sparkline;
pub mod static_widget;
pub mod tabs;
pub mod text_area;
pub mod toast;
pub mod tooltip;
pub mod tree;

pub use collapsible::Collapsible;
pub use container::{BorderStyle, Container};
pub use data_table::{Column, DataTable};
pub use diff_view::{DiffMode, DiffView};
pub use directory_tree::DirectoryTree;
pub use form_controls::{Checkbox, RadioButton, Switch};
pub use label::{Alignment, Label};
pub use loading_indicator::{IndicatorStyle, LoadingIndicator};
pub use markdown::MarkdownRenderer;
pub use modal::Modal;
pub use option_list::OptionList;
pub use progress_bar::{ProgressBar, ProgressMode};
pub use rich_log::RichLog;
pub use select_list::SelectList;
pub use sparkline::Sparkline;
pub use static_widget::StaticWidget;
pub use tabs::{Tab, TabBarPosition, Tabs};
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::cell::Cell;
    use crate::event::{Event, KeyCode, KeyEvent};
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

    // --- Phase 4.3 integration tests ---

    #[test]
    fn tabs_with_progress_bar_content() {
        use crate::segment::Segment;

        let bar = ProgressBar::new(0.7);
        let mut bar_buf = ScreenBuffer::new(Size::new(20, 1));
        bar.render(Rect::new(0, 0, 20, 1), &mut bar_buf);

        // Tabs can hold arbitrary content
        let tabs = Tabs::new(vec![
            Tab::new("Status").with_content(vec![vec![Segment::new("Progress: 70%")]]),
            Tab::new("Details").with_content(vec![vec![Segment::new("All good")]]),
        ]);
        assert_eq!(tabs.tab_count(), 2);
        assert_eq!(tabs.active_tab(), 0);

        let mut buf = ScreenBuffer::new(Size::new(40, 5));
        tabs.render(Rect::new(0, 0, 40, 5), &mut buf);

        let row1: String = (0..40)
            .map(|x| buf.get(x, 1).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row1.contains("Progress: 70%"));
    }

    #[test]
    fn form_controls_group_radio_selection() {
        let mut radios = vec![
            RadioButton::new("Option A").with_selected(true),
            RadioButton::new("Option B"),
            RadioButton::new("Option C"),
        ];

        assert!(radios[0].is_selected());
        assert!(!radios[1].is_selected());

        // Simulate selecting option B (deselect all, select new)
        for r in &mut radios {
            r.deselect();
        }
        radios[1].select();

        assert!(!radios[0].is_selected());
        assert!(radios[1].is_selected());
        assert!(!radios[2].is_selected());
    }

    #[test]
    fn animated_widgets_tick() {
        let mut bar = ProgressBar::indeterminate();
        let mut loader = LoadingIndicator::new();

        bar.tick();
        loader.tick();

        assert!(matches!(
            bar.mode(),
            ProgressMode::Indeterminate { phase: 1 }
        ));
        assert_eq!(loader.frame(), 1);

        // Render after ticking
        let mut buf = ScreenBuffer::new(Size::new(20, 2));
        bar.render(Rect::new(0, 0, 20, 1), &mut buf);
        loader.render(Rect::new(0, 1, 20, 1), &mut buf);

        // Both should have rendered non-space chars
        assert_ne!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some(" "));
        assert_ne!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some(" "));
    }

    #[test]
    fn collapsible_with_option_list() {
        let mut collapsible = Collapsible::new("Settings")
            .with_content(vec![
                vec![crate::segment::Segment::new("Dark Mode")],
                vec![crate::segment::Segment::new("Sound")],
            ])
            .with_expanded(true);

        let ol = OptionList::new(vec!["Theme".to_string(), "Language".to_string()]);

        // Render both
        let mut buf = ScreenBuffer::new(Size::new(30, 10));
        collapsible.render(Rect::new(0, 0, 30, 5), &mut buf);
        ol.render(Rect::new(0, 5, 30, 5), &mut buf);

        let row0: String = (0..30)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect();
        assert!(row0.contains("Settings"));

        // Collapse it
        collapsible.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Enter)));
        assert!(!collapsible.is_expanded());
    }

    #[test]
    fn sparkline_live_data_push() {
        let mut spark = Sparkline::new(vec![]).with_max_width(5);
        for i in 0..10 {
            spark.push(i as f32);
        }
        // Should only have last 5 data points
        assert_eq!(spark.data().len(), 5);
        assert_eq!(spark.data()[0], 5.0);
        assert_eq!(spark.data()[4], 9.0);

        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        spark.render(Rect::new(0, 0, 10, 1), &mut buf);
        // First 5 positions should have bar chars
        assert_ne!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some(" "));
    }

    #[test]
    fn empty_widgets_render_safely() {
        let tabs = Tabs::new(vec![]);
        let ol = OptionList::new(vec![]);
        let spark = Sparkline::new(vec![]);
        let collapsible = Collapsible::new("Empty");

        let mut buf = ScreenBuffer::new(Size::new(20, 10));
        tabs.render(Rect::new(0, 0, 20, 2), &mut buf);
        ol.render(Rect::new(0, 2, 20, 2), &mut buf);
        spark.render(Rect::new(0, 4, 20, 2), &mut buf);
        collapsible.render(Rect::new(0, 6, 20, 2), &mut buf);
        // No panic
    }

    #[test]
    fn utf8_across_all_widgets() {
        use crate::segment::Segment;

        let tabs = Tabs::new(vec![
            Tab::new("日本語").with_content(vec![vec![Segment::new("コンテンツ")]]),
        ]);
        let switch = Switch::new("暗いモード");
        let checkbox = Checkbox::new("同意する");
        let ol = OptionList::new(vec!["選択肢A".to_string(), "選択肢B".to_string()]);
        let spark = Sparkline::new(vec![1.0, 2.0, 3.0]);
        let collapsible = Collapsible::new("セクション").with_expanded(true);

        let mut buf = ScreenBuffer::new(Size::new(40, 20));
        tabs.render(Rect::new(0, 0, 40, 3), &mut buf);
        switch.render(Rect::new(0, 3, 40, 1), &mut buf);
        checkbox.render(Rect::new(0, 4, 40, 1), &mut buf);
        ol.render(Rect::new(0, 5, 40, 3), &mut buf);
        spark.render(Rect::new(0, 8, 40, 1), &mut buf);
        collapsible.render(Rect::new(0, 9, 40, 3), &mut buf);
        // No panic, no truncation errors
    }

    #[test]
    fn event_consumption_correctness() {
        let mut tabs = Tabs::new(vec![Tab::new("A"), Tab::new("B")]);
        let mut switch = Switch::new("S");
        let mut checkbox = Checkbox::new("C");
        let mut radio = RadioButton::new("R");
        let mut collapsible = Collapsible::new("X");
        let mut ol = OptionList::new(vec!["1".to_string()]);

        // Space/Enter consumed by interactive widgets
        assert_eq!(
            switch.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Char(' ')))),
            EventResult::Consumed
        );
        assert_eq!(
            checkbox.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Enter))),
            EventResult::Consumed
        );
        assert_eq!(
            radio.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Enter))),
            EventResult::Consumed
        );
        assert_eq!(
            collapsible.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Enter))),
            EventResult::Consumed
        );
        assert_eq!(
            ol.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Down))),
            EventResult::Consumed
        );
        assert_eq!(
            tabs.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Right))),
            EventResult::Consumed
        );

        // Unhandled events ignored
        assert_eq!(
            switch.handle_event(&Event::Key(KeyEvent::plain(KeyCode::F(1)))),
            EventResult::Ignored
        );
    }

    #[test]
    fn zero_size_area_no_panic() {
        let tabs = Tabs::new(vec![Tab::new("A")]);
        let bar = ProgressBar::new(0.5);
        let loader = LoadingIndicator::new();
        let collapsible = Collapsible::new("C");
        let switch = Switch::new("S");
        let ol = OptionList::new(vec!["X".to_string()]);
        let spark = Sparkline::new(vec![1.0]);

        let mut buf = ScreenBuffer::new(Size::new(1, 1));
        let zero = Rect::new(0, 0, 0, 0);

        tabs.render(zero, &mut buf);
        bar.render(zero, &mut buf);
        loader.render(zero, &mut buf);
        collapsible.render(zero, &mut buf);
        switch.render(zero, &mut buf);
        ol.render(zero, &mut buf);
        spark.render(zero, &mut buf);
        // No panic
    }

    #[test]
    fn style_propagation() {
        let style = Style::default().bold(true);
        let switch = Switch::new("Bold")
            .with_on_style(style.clone())
            .with_state(true);
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        switch.render(Rect::new(0, 0, 20, 1), &mut buf);

        assert!(buf.get(0, 0).map(|c| c.style.bold).unwrap_or(false));
    }

    #[test]
    fn border_consistency() {
        let widgets_with_borders: Vec<Box<dyn Widget>> = vec![
            Box::new(Tabs::new(vec![Tab::new("A")]).with_border(BorderStyle::Single)),
            Box::new(ProgressBar::new(0.5).with_border(BorderStyle::Single)),
            Box::new(Collapsible::new("C").with_border(BorderStyle::Single)),
            Box::new(OptionList::new(vec!["O".to_string()]).with_border(BorderStyle::Single)),
        ];

        for widget in &widgets_with_borders {
            let mut buf = ScreenBuffer::new(Size::new(20, 5));
            widget.render(Rect::new(0, 0, 20, 5), &mut buf);
            // All should have top-left corner border char
            assert_eq!(
                buf.get(0, 0).map(|c| c.grapheme.as_str()),
                Some("┌"),
                "Widget should render single border"
            );
        }
    }
}
