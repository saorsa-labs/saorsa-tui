//! Toast notification widget — corner-positioned ephemeral message.

use crate::geometry::Size;
use crate::overlay::{OverlayConfig, OverlayPosition};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{string_display_width, truncate_to_display_width};

/// Corner position for a toast notification.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastPosition {
    /// Top-left corner of the screen.
    TopLeft,
    /// Top-right corner of the screen.
    #[default]
    TopRight,
    /// Bottom-left corner of the screen.
    BottomLeft,
    /// Bottom-right corner of the screen.
    BottomRight,
}

/// A toast notification widget that appears in a screen corner.
///
/// Toasts are typically short-lived, non-modal messages. Use
/// [`Toast::to_overlay_config`] with a screen size to produce an
/// [`OverlayConfig`] for use with [`crate::overlay::ScreenStack`].
#[derive(Clone, Debug)]
pub struct Toast {
    message: String,
    position: ToastPosition,
    style: Style,
    width: u16,
}

impl Toast {
    /// Create a new toast with a message (defaults to top-right, width 30).
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            position: ToastPosition::TopRight,
            style: Style::default(),
            width: 30,
        }
    }

    /// Set the toast position.
    #[must_use]
    pub fn with_position(mut self, pos: ToastPosition) -> Self {
        self.position = pos;
        self
    }

    /// Set the toast style.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the toast width.
    #[must_use]
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Render the toast to lines ready for the compositor.
    ///
    /// Produces a single-line notification padded to the toast width.
    pub fn render_to_lines(&self) -> Vec<Vec<Segment>> {
        let w = self.width as usize;
        let truncated = truncate_to_display_width(&self.message, w);
        let display_w = string_display_width(truncated) as usize;
        let mut padded = String::with_capacity(w);
        padded.push_str(truncated);
        for _ in display_w..w {
            padded.push(' ');
        }
        vec![vec![Segment::styled(padded, self.style.clone())]]
    }

    /// Create an overlay config for this toast at the appropriate screen corner.
    pub fn to_overlay_config(&self, screen: Size) -> OverlayConfig {
        let height = 1u16;
        let pos = match self.position {
            ToastPosition::TopLeft => crate::geometry::Position::new(0, 0),
            ToastPosition::TopRight => {
                let x = screen.width.saturating_sub(self.width);
                crate::geometry::Position::new(x, 0)
            }
            ToastPosition::BottomLeft => {
                let y = screen.height.saturating_sub(height);
                crate::geometry::Position::new(0, y)
            }
            ToastPosition::BottomRight => {
                let x = screen.width.saturating_sub(self.width);
                let y = screen.height.saturating_sub(height);
                crate::geometry::Position::new(x, y)
            }
        };
        OverlayConfig {
            position: OverlayPosition::At(pos),
            size: Size::new(self.width, height),
            z_offset: 0,
            dim_background: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_toast_defaults() {
        let t = Toast::new("Hello");
        assert!(t.message == "Hello");
        assert!(t.position == ToastPosition::TopRight);
        assert!(t.width == 30);
    }

    #[test]
    fn render_to_lines_produces_content() {
        let t = Toast::new("Test message");
        let lines = t.render_to_lines();
        assert!(lines.len() == 1);
        let text: String = lines[0].iter().map(|s| &*s.text).collect();
        assert!(text.contains("Test message"));
    }

    #[test]
    fn top_right_position() {
        let t = Toast::new("Hi").with_width(10);
        let config = t.to_overlay_config(Size::new(80, 24));
        match &config.position {
            OverlayPosition::At(p) => {
                assert!(p.x == 70); // 80 - 10
                assert!(p.y == 0);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn bottom_left_position() {
        let t = Toast::new("Hi")
            .with_position(ToastPosition::BottomLeft)
            .with_width(10);
        let config = t.to_overlay_config(Size::new(80, 24));
        match &config.position {
            OverlayPosition::At(p) => {
                assert!(p.x == 0);
                assert!(p.y == 23); // 24 - 1
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn top_left_position() {
        let t = Toast::new("Hi")
            .with_position(ToastPosition::TopLeft)
            .with_width(10);
        let config = t.to_overlay_config(Size::new(80, 24));
        match &config.position {
            OverlayPosition::At(p) => {
                assert!(p.x == 0);
                assert!(p.y == 0);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn bottom_right_position() {
        let t = Toast::new("Hi")
            .with_position(ToastPosition::BottomRight)
            .with_width(10);
        let config = t.to_overlay_config(Size::new(80, 24));
        match &config.position {
            OverlayPosition::At(p) => {
                assert!(p.x == 70); // 80 - 10
                assert!(p.y == 23); // 24 - 1
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn toast_style_applied() {
        let style = Style::new().bold(true);
        let t = Toast::new("Styled").with_style(style);
        let lines = t.render_to_lines();
        assert!(lines[0][0].style.bold);
    }

    #[test]
    fn custom_width_respected() {
        let t = Toast::new("AB").with_width(5);
        let lines = t.render_to_lines();
        let text: String = lines[0].iter().map(|s| &*s.text).collect();
        assert!(text.len() == 5);
        assert!(text.starts_with("AB"));
    }

    #[test]
    fn no_dim_background() {
        let t = Toast::new("msg");
        let config = t.to_overlay_config(Size::new(80, 24));
        assert!(!config.dim_background);
    }
}
