//! Snapshot tests for TextArea and MarkdownRenderer widgets.

#[path = "snapshot_helpers.rs"]
mod snapshot_helpers;

use fae_core::buffer::ScreenBuffer;
use fae_core::geometry::{Rect, Size};
use fae_core::widget::{MarkdownRenderer, TextArea, Widget};
use snapshot_helpers::render_to_text;

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

// --- TextArea Snapshots ---

#[test]
fn snapshot_textarea_empty() {
    let ta = TextArea::new();
    let rendered = render_widget_to_text(&ta, 40, 5);
    insta::assert_snapshot!("textarea_empty", rendered);
}

#[test]
fn snapshot_textarea_single_line() {
    let ta = TextArea::from_text("Hello, World!");
    let rendered = render_widget_to_text(&ta, 40, 3);
    insta::assert_snapshot!("textarea_single_line", rendered);
}

#[test]
fn snapshot_textarea_multiline() {
    let ta = TextArea::from_text("Line 1\nLine 2\nLine 3\nLine 4");
    let rendered = render_widget_to_text(&ta, 40, 6);
    insta::assert_snapshot!("textarea_multiline", rendered);
}

#[test]
fn snapshot_textarea_word_wrap() {
    let ta = TextArea::from_text(
        "This is a very long line of text that should wrap around when rendered in a narrow width area",
    );
    let rendered = render_widget_to_text(&ta, 20, 8);
    insta::assert_snapshot!("textarea_word_wrap", rendered);
}

#[test]
fn snapshot_textarea_line_numbers() {
    let ta =
        TextArea::from_text("fn main() {\n    println!(\"Hello\");\n}\n").with_line_numbers(true);
    let rendered = render_widget_to_text(&ta, 50, 5);
    insta::assert_snapshot!("textarea_line_numbers", rendered);
}

#[test]
fn snapshot_textarea_cursor() {
    let mut ta = TextArea::from_text("Hello\nWorld");
    ta.cursor.position.line = 1;
    ta.cursor.position.col = 2;
    let rendered = render_widget_to_text(&ta, 30, 4);
    insta::assert_snapshot!("textarea_cursor", rendered);
}

// --- MarkdownRenderer Snapshots ---

#[test]
fn snapshot_markdown_heading() {
    let mut md = MarkdownRenderer::new();
    md.push_str("# Title");
    let lines = md.render_to_lines(40);
    let rendered = render_to_text(&lines, 40, 3);
    insta::assert_snapshot!("markdown_heading", rendered);
}

#[test]
fn snapshot_markdown_bold_italic() {
    let mut md = MarkdownRenderer::new();
    md.push_str("**bold** and *italic*");
    let lines = md.render_to_lines(40);
    let rendered = render_to_text(&lines, 40, 3);
    insta::assert_snapshot!("markdown_bold_italic", rendered);
}

#[test]
fn snapshot_markdown_code_block() {
    let mut md = MarkdownRenderer::new();
    md.push_str("```rust\nfn main() {\n    println!(\"Hello\");\n}\n```");
    let lines = md.render_to_lines(40);
    let rendered = render_to_text(&lines, 40, 8);
    insta::assert_snapshot!("markdown_code_block", rendered);
}

#[test]
fn snapshot_markdown_list() {
    let mut md = MarkdownRenderer::new();
    md.push_str("- First item\n- Second item\n- Third item");
    let lines = md.render_to_lines(40);
    let rendered = render_to_text(&lines, 40, 6);
    insta::assert_snapshot!("markdown_list", rendered);
}

#[test]
fn snapshot_markdown_mixed() {
    let mut md = MarkdownRenderer::new();
    md.push_str("# Main Title\n\nSome **bold** and *italic* text.\n\n## Subsection\n\nA paragraph with `code`.\n\n```\ncode block\n```\n\n- list item");
    let lines = md.render_to_lines(50);
    let rendered = render_to_text(&lines, 50, 15);
    insta::assert_snapshot!("markdown_mixed", rendered);
}

#[test]
fn snapshot_markdown_link() {
    let mut md = MarkdownRenderer::new();
    md.push_str("[link text](https://example.com)");
    let lines = md.render_to_lines(40);
    let rendered = render_to_text(&lines, 40, 3);
    insta::assert_snapshot!("markdown_link", rendered);
}
