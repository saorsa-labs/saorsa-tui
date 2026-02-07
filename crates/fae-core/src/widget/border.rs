//! Border rendering utilities for widgets.
//!
//! Provides shared implementations for border rendering, inner area calculation,
//! and border character selection.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::geometry::Rect;
use crate::style::Style;

use super::BorderStyle;

/// Border character set: (top-left, top-right, bottom-left, bottom-right, horizontal, vertical)
pub type BorderChars = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
);

impl BorderStyle {
    /// Get the Unicode box-drawing characters for this border style.
    ///
    /// Returns `None` for `BorderStyle::None`.
    pub fn chars(self) -> Option<BorderChars> {
        match self {
            BorderStyle::None => None,
            BorderStyle::Single => Some((
                "\u{250c}", "\u{2510}", "\u{2514}", "\u{2518}", "\u{2500}", "\u{2502}",
            )),
            BorderStyle::Double => Some((
                "\u{2554}", "\u{2557}", "\u{255a}", "\u{255d}", "\u{2550}", "\u{2551}",
            )),
            BorderStyle::Rounded => Some((
                "\u{256d}", "\u{256e}", "\u{2570}", "\u{256f}", "\u{2500}", "\u{2502}",
            )),
            BorderStyle::Heavy => Some((
                "\u{250f}", "\u{2513}", "\u{2517}", "\u{251b}", "\u{2501}", "\u{2503}",
            )),
        }
    }
}

/// Render a border into the screen buffer.
///
/// If `border_style` is `BorderStyle::None`, this function does nothing.
pub fn render_border(
    area: Rect,
    border_style: BorderStyle,
    cell_style: Style,
    buf: &mut ScreenBuffer,
) {
    let Some((tl, tr, bl, br, h, v)) = border_style.chars() else {
        return;
    };

    let x1 = area.position.x;
    let y1 = area.position.y;
    let w = area.size.width;
    let h_val = area.size.height;

    if w == 0 || h_val == 0 {
        return;
    }

    let x2 = x1.saturating_add(w.saturating_sub(1));
    let y2 = y1.saturating_add(h_val.saturating_sub(1));

    // Corners
    buf.set(x1, y1, Cell::new(tl, cell_style.clone()));
    buf.set(x2, y1, Cell::new(tr, cell_style.clone()));
    buf.set(x1, y2, Cell::new(bl, cell_style.clone()));
    buf.set(x2, y2, Cell::new(br, cell_style.clone()));

    // Top and bottom edges
    for x in (x1 + 1)..x2 {
        buf.set(x, y1, Cell::new(h, cell_style.clone()));
        buf.set(x, y2, Cell::new(h, cell_style.clone()));
    }

    // Left and right edges
    for y in (y1 + 1)..y2 {
        buf.set(x1, y, Cell::new(v, cell_style.clone()));
        buf.set(x2, y, Cell::new(v, cell_style.clone()));
    }
}

/// Calculate the inner area after accounting for a border.
///
/// If `border_style` is `BorderStyle::None`, returns the original area.
/// Otherwise, shrinks the area by 1 cell on each side.
///
/// Returns a zero-sized rectangle if the area is too small for a border.
pub fn inner_area(area: Rect, border_style: BorderStyle) -> Rect {
    match border_style {
        BorderStyle::None => area,
        _ => {
            if area.size.width < 2 || area.size.height < 2 {
                return Rect::new(area.position.x, area.position.y, 0, 0);
            }
            Rect::new(
                area.position.x + 1,
                area.position.y + 1,
                area.size.width.saturating_sub(2),
                area.size.height.saturating_sub(2),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    #[test]
    fn border_style_chars_single() {
        let chars = BorderStyle::Single.chars();
        assert!(chars.is_some());
        match chars {
            Some((tl, tr, bl, br, h, v)) => {
                assert_eq!(tl, "\u{250c}");
                assert_eq!(tr, "\u{2510}");
                assert_eq!(bl, "\u{2514}");
                assert_eq!(br, "\u{2518}");
                assert_eq!(h, "\u{2500}");
                assert_eq!(v, "\u{2502}");
            }
            None => unreachable!("should have border chars"),
        }
    }

    #[test]
    fn border_style_chars_none() {
        assert!(BorderStyle::None.chars().is_none());
    }

    #[test]
    fn inner_area_no_border() {
        let area = Rect::new(5, 5, 20, 10);
        let inner = inner_area(area, BorderStyle::None);
        assert_eq!(inner, area);
    }

    #[test]
    fn inner_area_with_border() {
        let area = Rect::new(5, 5, 20, 10);
        let inner = inner_area(area, BorderStyle::Single);
        assert_eq!(inner.position.x, 6);
        assert_eq!(inner.position.y, 6);
        assert_eq!(inner.size.width, 18);
        assert_eq!(inner.size.height, 8);
    }

    #[test]
    fn inner_area_too_small() {
        let area = Rect::new(0, 0, 1, 1);
        let inner = inner_area(area, BorderStyle::Single);
        assert_eq!(inner.size.width, 0);
        assert_eq!(inner.size.height, 0);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn render_border_single() {
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        let area = Rect::new(0, 0, 10, 5);
        let style = Style::default();

        render_border(area, BorderStyle::Single, style, &mut buf);

        // Check corners
        assert_eq!(buf.get(0, 0).unwrap().grapheme, "\u{250c}");
        assert_eq!(buf.get(9, 0).unwrap().grapheme, "\u{2510}");
        assert_eq!(buf.get(0, 4).unwrap().grapheme, "\u{2514}");
        assert_eq!(buf.get(9, 4).unwrap().grapheme, "\u{2518}");

        // Check edges
        assert_eq!(buf.get(1, 0).unwrap().grapheme, "\u{2500}");
        assert_eq!(buf.get(0, 1).unwrap().grapheme, "\u{2502}");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn render_border_none_does_nothing() {
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        let area = Rect::new(0, 0, 10, 5);
        let style = Style::default();

        render_border(area, BorderStyle::None, style, &mut buf);

        // All cells should remain blank (default)
        assert_eq!(buf.get(0, 0).unwrap().grapheme, " ");
    }
}
