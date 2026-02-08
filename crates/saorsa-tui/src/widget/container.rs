//! Container widget — a box with optional border and title.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::geometry::Rect;
use crate::style::Style;

use super::Widget;

/// Border style for a container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BorderStyle {
    /// No border.
    #[default]
    None,
    /// Single line border: ┌┐└┘─│
    Single,
    /// Double line border: ╔╗╚╝═║
    Double,
    /// Rounded corners: ╭╮╰╯─│
    Rounded,
    /// Heavy/thick border: ┏┓┗┛━┃
    Heavy,
}

/// A container widget with optional border, title, and padding.
#[derive(Clone, Debug)]
pub struct Container {
    border: BorderStyle,
    border_style: Style,
    title: Option<String>,
    title_style: Style,
    padding: u16,
    fill_style: Style,
}

impl Container {
    /// Create a new container with no border.
    pub fn new() -> Self {
        Self {
            border: BorderStyle::None,
            border_style: Style::default(),
            title: None,
            title_style: Style::default(),
            padding: 0,
            fill_style: Style::default(),
        }
    }

    /// Set the border style.
    #[must_use]
    pub fn border(mut self, style: BorderStyle) -> Self {
        self.border = style;
        self
    }

    /// Set the border color/style.
    #[must_use]
    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    /// Set the title displayed in the top border.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the title style.
    #[must_use]
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Set (or clear) the title displayed in the top border.
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Convenience: set the title text.
    pub fn set_title_text(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Set inner padding (cells on each side).
    #[must_use]
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Get the current border style kind.
    pub fn border_style_kind(&self) -> BorderStyle {
        self.border
    }

    /// Set the border style kind.
    pub fn set_border(&mut self, style: BorderStyle) {
        self.border = style;
    }

    /// Set the border color/style.
    pub fn set_border_style(&mut self, style: Style) {
        self.border_style = style;
    }

    /// Set the title style.
    pub fn set_title_style(&mut self, style: Style) {
        self.title_style = style;
    }

    /// Set inner padding (cells on each side).
    pub fn set_padding(&mut self, padding: u16) {
        self.padding = padding;
    }

    /// Set the fill style for the container background.
    pub fn set_fill_style(&mut self, style: Style) {
        self.fill_style = style;
    }

    /// Calculate the inner area (after border and padding).
    pub fn inner_area(&self, area: Rect) -> Rect {
        let border_offset = if self.border != BorderStyle::None {
            1
        } else {
            0
        };
        let total_offset = border_offset + self.padding;

        if area.size.width <= total_offset * 2 || area.size.height <= total_offset * 2 {
            return Rect::new(
                area.position.x + total_offset,
                area.position.y + total_offset,
                0,
                0,
            );
        }

        Rect::new(
            area.position.x + total_offset,
            area.position.y + total_offset,
            area.size.width - total_offset * 2,
            area.size.height - total_offset * 2,
        )
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Container {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width < 2 || area.size.height < 2 {
            return;
        }

        // Background fill.
        if self.fill_style.bg.is_some() || self.fill_style.fg.is_some() {
            for y in 0..area.size.height {
                for x in 0..area.size.width {
                    buf.set(
                        area.position.x + x,
                        area.position.y + y,
                        Cell::new(" ", self.fill_style.clone()),
                    );
                }
            }
        }

        let Some((tl, tr, bl, br, h, v)) = self.border.chars() else {
            return; // No border to draw
        };

        let right = area.position.x + area.size.width - 1;
        let bottom = area.position.y + area.size.height - 1;

        // Corners
        buf.set(
            area.position.x,
            area.position.y,
            Cell::new(tl, self.border_style.clone()),
        );
        buf.set(
            right,
            area.position.y,
            Cell::new(tr, self.border_style.clone()),
        );
        buf.set(
            area.position.x,
            bottom,
            Cell::new(bl, self.border_style.clone()),
        );
        buf.set(right, bottom, Cell::new(br, self.border_style.clone()));

        // Top and bottom edges
        for x in (area.position.x + 1)..right {
            buf.set(x, area.position.y, Cell::new(h, self.border_style.clone()));
            buf.set(x, bottom, Cell::new(h, self.border_style.clone()));
        }

        // Left and right edges
        for y in (area.position.y + 1)..bottom {
            buf.set(area.position.x, y, Cell::new(v, self.border_style.clone()));
            buf.set(right, y, Cell::new(v, self.border_style.clone()));
        }

        // Title (rendered in the top border)
        if let Some(ref title) = self.title {
            let max_title_width = (area.size.width.saturating_sub(4)) as usize; // leave space for corners + padding
            let display_title = if title.len() > max_title_width {
                &title[..max_title_width]
            } else {
                title.as_str()
            };

            let start_x = area.position.x + 2; // after corner + space
            for (i, ch) in display_title.chars().enumerate() {
                let x = start_x + i as u16;
                if x < right {
                    buf.set(
                        x,
                        area.position.y,
                        Cell::new(ch.to_string(), self.title_style.clone()),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{Color, NamedColor};
    use crate::geometry::Size;

    #[test]
    fn single_border_corners() {
        let container = Container::new().border(BorderStyle::Single);
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        container.render(Rect::new(0, 0, 10, 5), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("┌"));
        assert_eq!(buf.get(9, 0).map(|c| c.grapheme.as_str()), Some("┐"));
        assert_eq!(buf.get(0, 4).map(|c| c.grapheme.as_str()), Some("└"));
        assert_eq!(buf.get(9, 4).map(|c| c.grapheme.as_str()), Some("┘"));
    }

    #[test]
    fn single_border_edges() {
        let container = Container::new().border(BorderStyle::Single);
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        container.render(Rect::new(0, 0, 10, 5), &mut buf);

        // Top edge
        assert_eq!(buf.get(1, 0).map(|c| c.grapheme.as_str()), Some("─"));
        // Bottom edge
        assert_eq!(buf.get(1, 4).map(|c| c.grapheme.as_str()), Some("─"));
        // Left edge
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("│"));
        // Right edge
        assert_eq!(buf.get(9, 1).map(|c| c.grapheme.as_str()), Some("│"));
    }

    #[test]
    fn double_border() {
        let container = Container::new().border(BorderStyle::Double);
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        container.render(Rect::new(0, 0, 10, 5), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("╔"));
        assert_eq!(buf.get(9, 0).map(|c| c.grapheme.as_str()), Some("╗"));
    }

    #[test]
    fn rounded_border() {
        let container = Container::new().border(BorderStyle::Rounded);
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        container.render(Rect::new(0, 0, 10, 5), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("╭"));
        assert_eq!(buf.get(9, 0).map(|c| c.grapheme.as_str()), Some("╮"));
    }

    #[test]
    fn heavy_border() {
        let container = Container::new().border(BorderStyle::Heavy);
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        container.render(Rect::new(0, 0, 10, 5), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("┏"));
        assert_eq!(buf.get(1, 0).map(|c| c.grapheme.as_str()), Some("━"));
    }

    #[test]
    fn border_with_title() {
        let container = Container::new().border(BorderStyle::Single).title("Test");
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        container.render(Rect::new(0, 0, 20, 5), &mut buf);

        assert_eq!(buf.get(2, 0).map(|c| c.grapheme.as_str()), Some("T"));
        assert_eq!(buf.get(3, 0).map(|c| c.grapheme.as_str()), Some("e"));
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("s"));
        assert_eq!(buf.get(5, 0).map(|c| c.grapheme.as_str()), Some("t"));
    }

    #[test]
    fn border_with_style() {
        let style = Style::new().fg(Color::Named(NamedColor::Cyan));
        let container = Container::new()
            .border(BorderStyle::Single)
            .border_style(style.clone());
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        container.render(Rect::new(0, 0, 10, 5), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| &c.style), Some(&style));
    }

    #[test]
    fn inner_area_with_border() {
        let container = Container::new().border(BorderStyle::Single);
        let inner = container.inner_area(Rect::new(0, 0, 20, 10));
        assert_eq!(inner, Rect::new(1, 1, 18, 8));
    }

    #[test]
    fn inner_area_with_border_and_padding() {
        let container = Container::new().border(BorderStyle::Single).padding(1);
        let inner = container.inner_area(Rect::new(0, 0, 20, 10));
        assert_eq!(inner, Rect::new(2, 2, 16, 6));
    }

    #[test]
    fn inner_area_no_border() {
        let container = Container::new();
        let inner = container.inner_area(Rect::new(0, 0, 20, 10));
        assert_eq!(inner, Rect::new(0, 0, 20, 10));
    }

    #[test]
    fn no_border_renders_nothing() {
        let container = Container::new();
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        container.render(Rect::new(0, 0, 10, 5), &mut buf);
        // All cells should still be blank
        assert!(buf.get(0, 0).is_some_and(|c| c.is_blank()));
    }
}
