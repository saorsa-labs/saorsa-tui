//! Simple option list widget for static selections.
//!
//! A simplified alternative to [`SelectList`] for static option sets
//! without fuzzy filtering. Useful for settings screens and forms.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use unicode_width::UnicodeWidthStr;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// A simple option list widget.
///
/// Displays a list of string options with keyboard navigation
/// and selection highlighting. Unlike [`SelectList`], this widget
/// does not support filtering or generic item types.
pub struct OptionList {
    /// Option labels.
    options: Vec<String>,
    /// Index of the currently selected option.
    selected: usize,
    /// Scroll offset (first visible option index).
    scroll_offset: usize,
    /// Style for unselected options.
    option_style: Style,
    /// Style for the selected option.
    selected_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Optional prefix for each option (e.g., "> ", "• ").
    prefix: Option<String>,
}

impl OptionList {
    /// Create a new option list with the given options.
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected: 0,
            scroll_offset: 0,
            option_style: Style::default(),
            selected_style: Style::default().reverse(true),
            border: BorderStyle::None,
            prefix: None,
        }
    }

    /// Set the style for the selected option.
    #[must_use]
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set the style for unselected options.
    #[must_use]
    pub fn with_option_style(mut self, style: Style) -> Self {
        self.option_style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set a prefix string for each option.
    #[must_use]
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = Some(prefix.to_string());
        self
    }

    /// Get the options slice.
    pub fn options(&self) -> &[String] {
        &self.options
    }

    /// Set new options (resets selection to 0).
    pub fn set_options(&mut self, options: Vec<String>) {
        self.options = options;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Get the selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected index (clamped to valid range).
    pub fn set_selected(&mut self, idx: usize) {
        if self.options.is_empty() {
            self.selected = 0;
        } else {
            self.selected = idx.min(self.options.len().saturating_sub(1));
        }
    }

    /// Get the selected option text.
    pub fn selected_option(&self) -> Option<&str> {
        self.options.get(self.selected).map(|s| s.as_str())
    }

    /// Ensure the selected item is visible by adjusting scroll offset.
    fn ensure_visible(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + height {
            self.scroll_offset = self.selected.saturating_sub(height.saturating_sub(1));
        }
    }
}

impl Widget for OptionList {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        super::border::render_border(area, self.border, self.option_style.clone(), buf);
        let inner = super::border::inner_area(area, self.border);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let w = inner.size.width as usize;
        let h = inner.size.height as usize;
        let x0 = inner.position.x;

        let visible_end = (self.scroll_offset + h).min(self.options.len());

        for (row, opt_idx) in (self.scroll_offset..visible_end).enumerate() {
            let y = inner.position.y + row as u16;

            if let Some(option) = self.options.get(opt_idx) {
                let is_selected = opt_idx == self.selected;
                let style = if is_selected {
                    &self.selected_style
                } else {
                    &self.option_style
                };

                let text = match &self.prefix {
                    Some(pfx) => format!("{pfx}{option}"),
                    None => option.clone(),
                };

                let truncated = truncate_to_display_width(&text, w);
                let mut col: u16 = 0;

                for ch in truncated.chars() {
                    if col as usize >= w {
                        break;
                    }
                    let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
                    if col as usize + char_w > w {
                        break;
                    }
                    buf.set(x0 + col, y, Cell::new(ch.to_string(), style.clone()));
                    col += char_w as u16;
                }

                // Fill remaining width for selected item highlight
                if is_selected {
                    while (col as usize) < w {
                        buf.set(x0 + col, y, Cell::new(" ", style.clone()));
                        col += 1;
                    }
                }
            }
        }
    }
}

impl InteractiveWidget for OptionList {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        if self.options.is_empty() {
            return EventResult::Ignored;
        }

        match code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                self.ensure_visible(20);
                EventResult::Consumed
            }
            KeyCode::Down => {
                if self.selected < self.options.len().saturating_sub(1) {
                    self.selected += 1;
                }
                self.ensure_visible(20);
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.selected = 0;
                self.scroll_offset = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                self.selected = self.options.len().saturating_sub(1);
                self.ensure_visible(20);
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(20);
                self.ensure_visible(20);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                self.selected = (self.selected + 20).min(self.options.len().saturating_sub(1));
                self.ensure_visible(20);
                EventResult::Consumed
            }
            KeyCode::Enter => EventResult::Consumed,
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    fn opts(labels: &[&str]) -> Vec<String> {
        labels.iter().map(|s| s.to_string()).collect()
    }

    fn row_text(buf: &ScreenBuffer, y: u16, w: u16) -> String {
        (0..w)
            .map(|x| buf.get(x, y).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect()
    }

    #[test]
    fn create_with_options() {
        let ol = OptionList::new(opts(&["A", "B", "C"]));
        assert_eq!(ol.options().len(), 3);
        assert_eq!(ol.selected(), 0);
    }

    #[test]
    fn render_highlights_selected() {
        let ol = OptionList::new(opts(&["Alpha", "Beta", "Gamma"]));
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        ol.render(Rect::new(0, 0, 20, 5), &mut buf);

        let row0 = row_text(&buf, 0, 20);
        assert!(row0.contains("Alpha"));
    }

    #[test]
    fn navigate_up_down() {
        let mut ol = OptionList::new(opts(&["A", "B", "C"]));
        assert_eq!(ol.selected(), 0);

        ol.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Down)));
        assert_eq!(ol.selected(), 1);

        ol.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Down)));
        assert_eq!(ol.selected(), 2);

        // Can't go past end
        ol.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Down)));
        assert_eq!(ol.selected(), 2);

        ol.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Up)));
        assert_eq!(ol.selected(), 1);
    }

    #[test]
    fn home_end_navigation() {
        let mut ol = OptionList::new(opts(&["A", "B", "C", "D"]));
        ol.handle_event(&Event::Key(KeyEvent::plain(KeyCode::End)));
        assert_eq!(ol.selected(), 3);

        ol.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Home)));
        assert_eq!(ol.selected(), 0);
    }

    #[test]
    fn set_selected_clamped() {
        let mut ol = OptionList::new(opts(&["A", "B"]));
        ol.set_selected(100);
        assert_eq!(ol.selected(), 1);
    }

    #[test]
    fn selected_option_text() {
        let ol = OptionList::new(opts(&["Foo", "Bar"]));
        assert_eq!(ol.selected_option(), Some("Foo"));
    }

    #[test]
    fn empty_list() {
        let ol = OptionList::new(vec![]);
        assert_eq!(ol.selected(), 0);
        assert!(ol.selected_option().is_none());

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        ol.render(Rect::new(0, 0, 20, 5), &mut buf);
        // Should not panic
    }

    #[test]
    fn single_option() {
        let ol = OptionList::new(opts(&["Only"]));
        assert_eq!(ol.selected_option(), Some("Only"));
    }

    #[test]
    fn custom_prefix() {
        let ol = OptionList::new(opts(&["Item"])).with_prefix("> ");
        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        ol.render(Rect::new(0, 0, 20, 3), &mut buf);
        let row = row_text(&buf, 0, 20);
        assert!(row.contains("> Item"));
    }

    #[test]
    fn set_options_resets_selection() {
        let mut ol = OptionList::new(opts(&["A", "B", "C"]));
        ol.set_selected(2);
        ol.set_options(opts(&["X", "Y"]));
        assert_eq!(ol.selected(), 0);
    }

    #[test]
    fn border_rendering() {
        let ol = OptionList::new(opts(&["T"])).with_border(BorderStyle::Single);
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        ol.render(Rect::new(0, 0, 20, 5), &mut buf);
        assert_eq!(buf.get(0, 0).unwrap().grapheme, "┌");
    }

    #[test]
    fn enter_consumed() {
        let mut ol = OptionList::new(opts(&["A"]));
        let result = ol.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Enter)));
        assert_eq!(result, EventResult::Consumed);
    }
}
