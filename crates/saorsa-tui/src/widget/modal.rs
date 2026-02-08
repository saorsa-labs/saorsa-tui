//! Modal dialog widget — centered overlay with title, body, and border.

use crate::overlay::{OverlayConfig, OverlayPosition};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{string_display_width, truncate_to_display_width};
use crate::widget::container::BorderStyle;

/// A modal dialog widget with title, body, border, and optional dim background.
///
/// Renders as a bordered box with a title in the top border and body content
/// inside. Use [`Modal::to_overlay_config`] to create an overlay config for
/// use with [`crate::overlay::ScreenStack`].
#[derive(Clone, Debug)]
pub struct Modal {
    title: String,
    body_lines: Vec<Vec<Segment>>,
    style: Style,
    border_style: BorderStyle,
    width: u16,
    height: u16,
}

impl Modal {
    /// Create a new modal with the given title and dimensions.
    pub fn new(title: impl Into<String>, width: u16, height: u16) -> Self {
        Self {
            title: title.into(),
            body_lines: Vec::new(),
            style: Style::default(),
            border_style: BorderStyle::Single,
            width,
            height,
        }
    }

    /// Set the body content lines.
    #[must_use]
    pub fn with_body(mut self, lines: Vec<Vec<Segment>>) -> Self {
        self.body_lines = lines;
        self
    }

    /// Set the style for the modal border and text.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border_style = border;
        self
    }

    /// Render the modal to lines ready for the compositor.
    ///
    /// Produces a bordered box with the title in the top border line and
    /// body content inside. Lines are padded to the modal's width.
    pub fn render_to_lines(&self) -> Vec<Vec<Segment>> {
        let chars = border_chars(self.border_style);
        let w = self.width as usize;
        let h = self.height as usize;

        if w < 2 || h < 2 {
            return Vec::new();
        }

        let mut lines: Vec<Vec<Segment>> = Vec::with_capacity(h);
        let inner_w = w.saturating_sub(2);

        // Top border with title
        let mut top = String::new();
        top.push_str(chars.top_left);
        if !self.title.is_empty() && inner_w > 0 {
            let truncated_title = truncate_to_display_width(&self.title, inner_w);
            top.push_str(truncated_title);
            let title_w = string_display_width(truncated_title) as usize;
            for _ in title_w..inner_w {
                top.push_str(chars.horizontal);
            }
        } else {
            for _ in 0..inner_w {
                top.push_str(chars.horizontal);
            }
        }
        top.push_str(chars.top_right);
        lines.push(vec![Segment::styled(top, self.style.clone())]);

        // Body rows
        let body_rows = h.saturating_sub(2);
        for row_idx in 0..body_rows {
            let mut row = String::new();
            row.push_str(chars.vertical);

            if row_idx < self.body_lines.len() {
                // We need to flatten the body line segments into text, then pad
                let body_text: String = self.body_lines[row_idx].iter().map(|s| &*s.text).collect();
                let truncated_body = truncate_to_display_width(&body_text, inner_w);
                let body_w = string_display_width(truncated_body) as usize;
                row.push_str(truncated_body);
                for _ in body_w..inner_w {
                    row.push(' ');
                }
            } else {
                for _ in 0..inner_w {
                    row.push(' ');
                }
            }

            row.push_str(chars.vertical);
            lines.push(vec![Segment::styled(row, self.style.clone())]);
        }

        // Bottom border
        let mut bottom = String::new();
        bottom.push_str(chars.bottom_left);
        for _ in 0..inner_w {
            bottom.push_str(chars.horizontal);
        }
        bottom.push_str(chars.bottom_right);
        lines.push(vec![Segment::styled(bottom, self.style.clone())]);

        lines
    }

    /// Create an overlay config for this modal (centered, with dim background).
    pub fn to_overlay_config(&self) -> OverlayConfig {
        OverlayConfig {
            position: OverlayPosition::Center,
            size: crate::geometry::Size::new(self.width, self.height),
            z_offset: 0,
            dim_background: true,
        }
    }
}

/// Border character set for rendering.
struct BorderCharSet {
    top_left: &'static str,
    top_right: &'static str,
    bottom_left: &'static str,
    bottom_right: &'static str,
    horizontal: &'static str,
    vertical: &'static str,
}

/// Get the border characters for a border style.
fn border_chars(style: BorderStyle) -> BorderCharSet {
    match style {
        BorderStyle::None => BorderCharSet {
            top_left: " ",
            top_right: " ",
            bottom_left: " ",
            bottom_right: " ",
            horizontal: " ",
            vertical: " ",
        },
        BorderStyle::Single => BorderCharSet {
            top_left: "┌",
            top_right: "┐",
            bottom_left: "└",
            bottom_right: "┘",
            horizontal: "─",
            vertical: "│",
        },
        BorderStyle::Double => BorderCharSet {
            top_left: "╔",
            top_right: "╗",
            bottom_left: "╚",
            bottom_right: "╝",
            horizontal: "═",
            vertical: "║",
        },
        BorderStyle::Rounded => BorderCharSet {
            top_left: "╭",
            top_right: "╮",
            bottom_left: "╰",
            bottom_right: "╯",
            horizontal: "─",
            vertical: "│",
        },
        BorderStyle::Heavy => BorderCharSet {
            top_left: "┏",
            top_right: "┓",
            bottom_left: "┗",
            bottom_right: "┛",
            horizontal: "━",
            vertical: "┃",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    #[test]
    fn new_modal_defaults() {
        let m = Modal::new("Test", 20, 10);
        assert!(m.title == "Test");
        assert!(m.width == 20);
        assert!(m.height == 10);
        assert!(m.body_lines.is_empty());
    }

    #[test]
    fn render_to_lines_correct_count() {
        let m = Modal::new("Title", 20, 5);
        let lines = m.render_to_lines();
        assert!(lines.len() == 5); // top + 3 body + bottom
    }

    #[test]
    fn title_in_top_border() {
        let m = Modal::new("Hello", 20, 5);
        let lines = m.render_to_lines();
        assert!(!lines.is_empty());
        let top_text: String = lines[0].iter().map(|s| &*s.text).collect();
        assert!(top_text.contains("Hello"));
        assert!(top_text.starts_with('┌'));
        assert!(top_text.ends_with('┐'));
    }

    #[test]
    fn body_content_inside_border() {
        let body = vec![vec![Segment::new("content")]];
        let m = Modal::new("T", 20, 5).with_body(body);
        let lines = m.render_to_lines();
        // Row 1 is the first body row
        let row_text: String = lines[1].iter().map(|s| &*s.text).collect();
        assert!(row_text.contains("content"));
        assert!(row_text.starts_with('│'));
        assert!(row_text.ends_with('│'));
    }

    #[test]
    fn empty_body_border_only() {
        let m = Modal::new("", 10, 3);
        let lines = m.render_to_lines();
        assert!(lines.len() == 3);
        // Top, one body row (empty), bottom
        let body_text: String = lines[1].iter().map(|s| &*s.text).collect();
        assert!(body_text.starts_with('│'));
        assert!(body_text.ends_with('│'));
    }

    #[test]
    fn style_applied() {
        let style = Style::new().bold(true);
        let m = Modal::new("S", 10, 3).with_style(style);
        let lines = m.render_to_lines();
        assert!(!lines.is_empty());
        assert!(lines[0][0].style.bold);
    }

    #[test]
    fn overlay_config_centered_with_dim() {
        let m = Modal::new("M", 30, 10);
        let config = m.to_overlay_config();
        assert!(config.position == OverlayPosition::Center);
        assert!(config.size == Size::new(30, 10));
        assert!(config.dim_background);
    }

    #[test]
    fn custom_border_style() {
        let m = Modal::new("D", 10, 3).with_border(BorderStyle::Double);
        let lines = m.render_to_lines();
        let top_text: String = lines[0].iter().map(|s| &*s.text).collect();
        assert!(top_text.starts_with('╔'));
        assert!(top_text.ends_with('╗'));
    }

    #[test]
    fn too_small_modal_returns_empty() {
        let m = Modal::new("X", 1, 1);
        let lines = m.render_to_lines();
        assert!(lines.is_empty());
    }

    #[test]
    fn bottom_border_correct() {
        let m = Modal::new("B", 10, 3);
        let lines = m.render_to_lines();
        let bottom_text: String = lines[2].iter().map(|s| &*s.text).collect();
        assert!(bottom_text.starts_with('└'));
        assert!(bottom_text.ends_with('┘'));
    }
}
