//! Basic snapshot tests for widget rendering.

use saorsa_core::buffer::ScreenBuffer;
use saorsa_core::geometry::{Rect, Size};
use saorsa_core::segment::Segment;
use saorsa_core::widget::{Container, Label, StaticWidget, Widget};

/// Render a widget to a text grid for snapshot testing.
fn render_widget_to_text(widget: &dyn Widget, width: u16, height: u16) -> String {
    let mut buf = ScreenBuffer::new(Size::new(width, height));
    widget.render(Rect::new(0, 0, width, height), &mut buf);

    let mut result = String::new();
    for y in 0..height {
        for x in 0..width {
            match buf.get(x, y) {
                Some(cell) => {
                    if cell.is_blank() {
                        result.push(' ');
                    } else {
                        result.push_str(&cell.grapheme);
                    }
                }
                None => result.push(' '),
            }
        }
        result.push('\n');
    }

    result
}

#[test]
fn snapshot_label_empty() {
    let label = Label::new("");
    let rendered = render_widget_to_text(&label, 20, 1);
    insta::assert_snapshot!("label_empty", rendered);
}

#[test]
fn snapshot_label_short() {
    let label = Label::new("Hello");
    let rendered = render_widget_to_text(&label, 20, 1);
    insta::assert_snapshot!("label_short", rendered);
}

#[test]
fn snapshot_label_long() {
    let label = Label::new("This is a very long label text");
    let rendered = render_widget_to_text(&label, 15, 1);
    insta::assert_snapshot!("label_long", rendered);
}

#[test]
fn snapshot_static_widget() {
    let segments = vec![
        Segment::new("Line 1"),
        Segment::new("Line 2"),
        Segment::new("Line 3"),
    ];
    let widget = StaticWidget::new(segments);
    let rendered = render_widget_to_text(&widget, 30, 3);
    insta::assert_snapshot!("static_widget", rendered);
}

#[test]
fn snapshot_container_empty() {
    let container = Container::new();
    let rendered = render_widget_to_text(&container, 20, 5);
    insta::assert_snapshot!("container_empty", rendered);
}
