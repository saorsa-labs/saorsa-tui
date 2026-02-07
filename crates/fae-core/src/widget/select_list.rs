//! Keyboard-navigable list widget with selection highlighting.
//!
//! Displays a list of items and highlights the currently selected one.
//! Supports vertical scrolling, keyboard navigation, and selection
//! confirmation via Enter key.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use unicode_width::UnicodeWidthStr;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// Type alias for the item render function.
type RenderFn<T> = Box<dyn Fn(&T) -> Vec<Segment>>;

/// Type alias for the selection callback.
type OnSelectFn<T> = Option<Box<dyn FnMut(&T)>>;

/// A keyboard-navigable list widget that displays items with selection.
///
/// Each item of type `T` is rendered via a customizable render function
/// into a row of [`Segment`]s. The selected item is highlighted with a
/// distinct style.
pub struct SelectList<T> {
    /// List items.
    items: Vec<T>,
    /// Function to render an item as Segments.
    render_fn: RenderFn<T>,
    /// Index of the currently selected item.
    selected: usize,
    /// Scroll offset (first visible item index).
    scroll_offset: usize,
    /// Style for unselected items.
    item_style: Style,
    /// Style for the selected item.
    selected_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Callback on selection confirmation (Enter pressed).
    on_select: OnSelectFn<T>,
}

impl<T> SelectList<T> {
    /// Create a new select list with the given items.
    ///
    /// By default, items render using their `Display` implementation
    /// if no custom render function is provided.
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            render_fn: Box::new(|_| vec![Segment::new("???")]),
            selected: 0,
            scroll_offset: 0,
            item_style: Style::default(),
            selected_style: Style::default().reverse(true),
            border: BorderStyle::None,
            on_select: None,
        }
    }

    /// Set a custom render function for items.
    #[must_use]
    pub fn with_render_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> Vec<Segment> + 'static,
    {
        self.render_fn = Box::new(f);
        self
    }

    /// Set the style for the selected item.
    #[must_use]
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set the style for unselected items.
    #[must_use]
    pub fn with_item_style(mut self, style: Style) -> Self {
        self.item_style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set a callback invoked when Enter is pressed on the selected item.
    #[must_use]
    pub fn with_on_select<F>(mut self, f: F) -> Self
    where
        F: FnMut(&T) + 'static,
    {
        self.on_select = Some(Box::new(f));
        self
    }

    /// Get a reference to the items.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Replace all items, resetting the selection to 0.
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Get the currently selected index.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected index (clamped to valid range).
    pub fn set_selected(&mut self, idx: usize) {
        if self.items.is_empty() {
            self.selected = 0;
        } else {
            self.selected = idx.min(self.items.len().saturating_sub(1));
        }
    }

    /// Get a reference to the currently selected item.
    pub fn selected_item(&self) -> Option<&T> {
        self.items.get(self.selected)
    }

    /// Move the selection by a delta (positive = down, negative = up).
    pub fn move_selection(&mut self, delta: isize) {
        if self.items.is_empty() {
            return;
        }
        let max_idx = self.items.len().saturating_sub(1);
        if delta < 0 {
            let abs_delta = delta.unsigned_abs();
            self.selected = self.selected.saturating_sub(abs_delta);
        } else {
            self.selected = (self.selected + delta as usize).min(max_idx);
        }
    }

    /// Get the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get the number of items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Calculate the inner area after accounting for borders.
    fn inner_area(&self, area: Rect) -> Rect {
        match self.border {
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

    /// Render the border into the buffer.
    fn render_border(&self, area: Rect, buf: &mut ScreenBuffer) {
        let chars = border_chars(self.border);
        let (tl, tr, bl, br, h, v) = match chars {
            Some(c) => c,
            None => return,
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
        buf.set(x1, y1, Cell::new(tl, self.item_style.clone()));
        buf.set(x2, y1, Cell::new(tr, self.item_style.clone()));
        buf.set(x1, y2, Cell::new(bl, self.item_style.clone()));
        buf.set(x2, y2, Cell::new(br, self.item_style.clone()));

        // Top and bottom edges
        for x in (x1 + 1)..x2 {
            buf.set(x, y1, Cell::new(h, self.item_style.clone()));
            buf.set(x, y2, Cell::new(h, self.item_style.clone()));
        }

        // Left and right edges
        for y in (y1 + 1)..y2 {
            buf.set(x1, y, Cell::new(v, self.item_style.clone()));
            buf.set(x2, y, Cell::new(v, self.item_style.clone()));
        }
    }

    /// Ensure the selected item is visible by adjusting scroll_offset.
    fn ensure_selected_visible(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        // If selected is above the visible window, scroll up
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
        // If selected is below the visible window, scroll down
        if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self
                .selected
                .saturating_sub(visible_height.saturating_sub(1));
        }
    }
}

impl<T> Widget for SelectList<T> {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        self.render_border(area, buf);

        let inner = self.inner_area(area);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let height = inner.size.height as usize;
        let width = inner.size.width as usize;

        // Clamp scroll offset
        let max_offset = self.items.len().saturating_sub(height.max(1));
        let scroll = self.scroll_offset.min(max_offset);

        let visible_end = (scroll + height).min(self.items.len());

        for (row, item_idx) in (scroll..visible_end).enumerate() {
            let y = inner.position.y + row as u16;
            if let Some(item) = self.items.get(item_idx) {
                let segments = (self.render_fn)(item);
                let is_selected = item_idx == self.selected;
                let style = if is_selected {
                    &self.selected_style
                } else {
                    &self.item_style
                };

                // If selected, fill the entire row with the selected style first
                if is_selected {
                    for col in 0..inner.size.width {
                        buf.set(inner.position.x + col, y, Cell::new(" ", style.clone()));
                    }
                }

                let mut col: u16 = 0;
                for segment in &segments {
                    if col as usize >= width {
                        break;
                    }
                    let remaining = width.saturating_sub(col as usize);
                    let truncated = truncate_to_display_width(&segment.text, remaining);
                    for ch in truncated.chars() {
                        let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
                        if col as usize + char_w > width {
                            break;
                        }
                        let x = inner.position.x + col;
                        // Use the item/selected style, not the segment's own style
                        buf.set(x, y, Cell::new(ch.to_string(), style.clone()));
                        col += char_w as u16;
                    }
                }
            }
        }
    }
}

impl<T> InteractiveWidget for SelectList<T> {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        match code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    // We'll adjust scroll in render, but also update here
                    // for correct state between renders
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if !self.items.is_empty() && self.selected < self.items.len().saturating_sub(1) {
                    self.selected += 1;
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                let page = 20;
                self.selected = self.selected.saturating_sub(page);
                self.ensure_selected_visible(20);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                let page = 20;
                if !self.items.is_empty() {
                    self.selected = (self.selected + page).min(self.items.len().saturating_sub(1));
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.selected = 0;
                self.scroll_offset = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                if !self.items.is_empty() {
                    self.selected = self.items.len().saturating_sub(1);
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(item) = self.items.get(self.selected)
                    && let Some(ref mut callback) = self.on_select
                {
                    callback(item);
                }
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

/// Return border characters for the given style, or None for `BorderStyle::None`.
fn border_chars(
    style: BorderStyle,
) -> Option<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
)> {
    match style {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::geometry::Size;

    fn make_string_list(items: Vec<&str>) -> SelectList<String> {
        let string_items: Vec<String> = items.into_iter().map(String::from).collect();
        SelectList::new(string_items).with_render_fn(|s| vec![Segment::new(s)])
    }

    #[test]
    fn new_list_with_items() {
        let list = make_string_list(vec!["Alpha", "Beta", "Gamma"]);
        assert_eq!(list.len(), 3);
        assert!(!list.is_empty());
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn empty_list() {
        let list: SelectList<String> = SelectList::new(vec![]);
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(list.selected_item().is_none());
    }

    #[test]
    fn selected_item_access() {
        let list = make_string_list(vec!["Alpha", "Beta", "Gamma"]);
        match list.selected_item() {
            Some(s) => assert_eq!(s, "Alpha"),
            None => unreachable!("should have selected item"),
        }
    }

    #[test]
    fn set_selected_clamps() {
        let mut list = make_string_list(vec!["Alpha", "Beta"]);
        list.set_selected(100);
        assert_eq!(list.selected(), 1); // clamped to last
        list.set_selected(0);
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn move_selection_positive_and_negative() {
        let mut list = make_string_list(vec!["A", "B", "C", "D"]);
        list.move_selection(2);
        assert_eq!(list.selected(), 2);
        list.move_selection(-1);
        assert_eq!(list.selected(), 1);
        // Clamp at 0
        list.move_selection(-100);
        assert_eq!(list.selected(), 0);
        // Clamp at end
        list.move_selection(100);
        assert_eq!(list.selected(), 3);
    }

    #[test]
    fn set_items_resets_selection() {
        let mut list = make_string_list(vec!["A", "B", "C"]);
        list.set_selected(2);
        assert_eq!(list.selected(), 2);
        list.set_items(vec!["X".into(), "Y".into()]);
        assert_eq!(list.selected(), 0);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn render_empty_list() {
        let list: SelectList<String> =
            SelectList::new(vec![]).with_render_fn(|s| vec![Segment::new(s)]);
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        list.render(Rect::new(0, 0, 20, 5), &mut buf);
        // Should not crash; buffer should be empty
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some(" "));
    }

    #[test]
    fn render_with_items() {
        let list = make_string_list(vec!["Hello", "World"]);
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        list.render(Rect::new(0, 0, 10, 5), &mut buf);
        // First row should show "Hello" (and it's selected, so full row filled)
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("o"));
        // Second row should show "World"
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("W"));
    }

    #[test]
    fn render_selected_item_highlighted() {
        let selected_style = Style::default().bold(true);
        let item_style = Style::default();
        let mut list = make_string_list(vec!["A", "B", "C"]);
        list.selected_style = selected_style.clone();
        list.item_style = item_style.clone();
        list.set_selected(1);

        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        list.render(Rect::new(0, 0, 10, 5), &mut buf);

        // Row 0 = "A" (not selected)
        let cell_a = buf.get(0, 0);
        assert!(cell_a.is_some());
        assert!(!cell_a.map(|c| c.style.bold).unwrap_or(true));

        // Row 1 = "B" (selected, bold)
        let cell_b = buf.get(0, 1);
        assert!(cell_b.is_some());
        assert!(cell_b.map(|c| c.style.bold).unwrap_or(false));
    }

    #[test]
    fn keyboard_navigation_up_down() {
        let mut list = make_string_list(vec!["A", "B", "C"]);

        let down = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });
        let up = Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(list.handle_event(&down), EventResult::Consumed);
        assert_eq!(list.selected(), 1);
        assert_eq!(list.handle_event(&down), EventResult::Consumed);
        assert_eq!(list.selected(), 2);
        // At end, stay at 2
        assert_eq!(list.handle_event(&down), EventResult::Consumed);
        assert_eq!(list.selected(), 2);
        // Up
        assert_eq!(list.handle_event(&up), EventResult::Consumed);
        assert_eq!(list.selected(), 1);
    }

    #[test]
    fn keyboard_home_end() {
        let mut list = make_string_list(vec!["A", "B", "C", "D", "E"]);

        let end = Event::Key(KeyEvent {
            code: KeyCode::End,
            modifiers: crate::event::Modifiers::NONE,
        });
        let home = Event::Key(KeyEvent {
            code: KeyCode::Home,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(list.handle_event(&end), EventResult::Consumed);
        assert_eq!(list.selected(), 4);
        assert_eq!(list.handle_event(&home), EventResult::Consumed);
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn keyboard_page_up_down() {
        let items: Vec<String> = (0..50).map(|i| format!("Item {i}")).collect();
        let mut list = SelectList::new(items).with_render_fn(|s| vec![Segment::new(s)]);

        let page_down = Event::Key(KeyEvent {
            code: KeyCode::PageDown,
            modifiers: crate::event::Modifiers::NONE,
        });
        let page_up = Event::Key(KeyEvent {
            code: KeyCode::PageUp,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(list.handle_event(&page_down), EventResult::Consumed);
        assert_eq!(list.selected(), 20);
        assert_eq!(list.handle_event(&page_up), EventResult::Consumed);
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn enter_triggers_on_select() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let selected_value = Rc::new(RefCell::new(String::new()));
        let captured = Rc::clone(&selected_value);

        let mut list =
            make_string_list(vec!["Alpha", "Beta"]).with_on_select(move |item: &String| {
                *captured.borrow_mut() = item.clone();
            });

        list.set_selected(1);

        let enter = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(list.handle_event(&enter), EventResult::Consumed);
        assert_eq!(*selected_value.borrow(), "Beta");
    }

    #[test]
    fn custom_render_fn() {
        let list = SelectList::new(vec![42, 99]).with_render_fn(|n: &i32| {
            vec![Segment::styled(
                format!("Number: {n}"),
                Style::default().italic(true),
            )]
        });

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        list.render(Rect::new(0, 0, 20, 5), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("N"));
        assert_eq!(buf.get(8, 0).map(|c| c.grapheme.as_str()), Some("4"));
    }

    #[test]
    fn render_with_border() {
        let list = make_string_list(vec!["Hello"]).with_border(BorderStyle::Single);
        let mut buf = ScreenBuffer::new(Size::new(12, 5));
        list.render(Rect::new(0, 0, 12, 5), &mut buf);

        // Top-left corner
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("\u{250c}"));
        // Content inside border
        assert_eq!(buf.get(1, 1).map(|c| c.grapheme.as_str()), Some("H"));
    }

    #[test]
    fn utf8_wide_chars_in_items() {
        let list =
            SelectList::new(vec!["你好世界".to_string()]).with_render_fn(|s| vec![Segment::new(s)]);

        let mut buf = ScreenBuffer::new(Size::new(6, 1));
        list.render(Rect::new(0, 0, 6, 1), &mut buf);

        // "你" = 2 width, "好" = 2 width, "世" = 2 width => fits exactly in 6
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("你"));
        assert_eq!(buf.get(2, 0).map(|c| c.grapheme.as_str()), Some("好"));
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("世"));
    }

    #[test]
    fn empty_list_handles_events_gracefully() {
        let mut list: SelectList<String> =
            SelectList::new(vec![]).with_render_fn(|s| vec![Segment::new(s)]);

        let down = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });
        let enter = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: crate::event::Modifiers::NONE,
        });

        // Should not crash
        assert_eq!(list.handle_event(&down), EventResult::Consumed);
        assert_eq!(list.handle_event(&enter), EventResult::Consumed);
        assert_eq!(list.selected(), 0);
    }

    #[test]
    fn unhandled_event_returns_ignored() {
        let mut list = make_string_list(vec!["A"]);

        let tab = Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(list.handle_event(&tab), EventResult::Ignored);
    }

    #[test]
    fn scroll_offset_adjusted_when_selection_out_of_view() {
        let items: Vec<String> = (0..30).map(|i| format!("Item {i}")).collect();
        let mut list = SelectList::new(items).with_render_fn(|s| vec![Segment::new(s)]);

        // Simulate scrolling down past visible area
        // Set selected to item 25
        list.set_selected(25);
        list.ensure_selected_visible(5); // visible height of 5
        // scroll_offset should be at least 21 (25 - 5 + 1)
        assert!(list.scroll_offset() >= 21);
    }

    #[test]
    fn builder_pattern_chaining() {
        let list = SelectList::new(vec!["A".to_string(), "B".to_string()])
            .with_render_fn(|s| vec![Segment::new(s)])
            .with_item_style(Style::default().dim(true))
            .with_selected_style(Style::default().bold(true))
            .with_border(BorderStyle::Rounded);

        assert_eq!(list.len(), 2);
        assert!(list.selected_style.bold);
        assert!(list.item_style.dim);
    }

    #[test]
    fn items_accessor() {
        let list = make_string_list(vec!["X", "Y", "Z"]);
        let items = list.items();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "X");
        assert_eq!(items[2], "Z");
    }

    #[test]
    fn render_with_selected_style_applies_color() {
        let selected_style = Style::default().fg(Color::Named(crate::color::NamedColor::Red));
        let list = make_string_list(vec!["First", "Second"]).with_selected_style(selected_style);

        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        list.render(Rect::new(0, 0, 10, 5), &mut buf);

        // First item is selected, should have red fg
        let cell = buf.get(0, 0);
        assert!(cell.is_some());
        match cell.map(|c| &c.style.fg) {
            Some(Some(Color::Named(crate::color::NamedColor::Red))) => {}
            other => panic!("Expected red fg, got {other:?}"),
        }
    }
}
