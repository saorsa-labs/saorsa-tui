//! Keyboard-navigable list widget with selection highlighting and fuzzy filtering.
//!
//! Displays a list of items and highlights the currently selected one.
//! Supports vertical scrolling, keyboard navigation, selection
//! confirmation via Enter key, and fuzzy text filtering.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use unicode_width::UnicodeWidthStr;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// Type alias for the item render function.
type RenderFn<T> = Box<dyn Fn(&T) -> Vec<Segment>>;

/// Type alias for the selection callback.
type OnSelectFn<T> = Option<Box<dyn FnMut(&T)>>;

/// Type alias for the search text extraction function.
type SearchFn<T> = Option<Box<dyn Fn(&T) -> String>>;

/// A keyboard-navigable list widget that displays items with selection.
///
/// Each item of type `T` is rendered via a customizable render function
/// into a row of [`Segment`]s. The selected item is highlighted with a
/// distinct style. Supports optional fuzzy filtering.
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
    /// Function to extract search text from an item.
    search_fn: SearchFn<T>,
    /// Current filter query.
    filter_query: String,
    /// Filtered indices (maps display index -> items index).
    filtered_indices: Vec<usize>,
    /// Whether filtering is active.
    filter_active: bool,
}

impl<T> SelectList<T> {
    /// Create a new select list with the given items.
    ///
    /// By default, items render using a placeholder. Set a render function
    /// with [`with_render_fn`](Self::with_render_fn).
    pub fn new(items: Vec<T>) -> Self {
        let len = items.len();
        Self {
            items,
            render_fn: Box::new(|_| vec![Segment::new("???")]),
            selected: 0,
            scroll_offset: 0,
            item_style: Style::default(),
            selected_style: Style::default().reverse(true),
            border: BorderStyle::None,
            on_select: None,
            search_fn: None,
            filter_query: String::new(),
            filtered_indices: (0..len).collect(),
            filter_active: false,
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

    /// Set a function to extract searchable text from an item.
    ///
    /// This enables fuzzy filtering. Without a search function,
    /// filtering has no effect.
    #[must_use]
    pub fn with_search_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> String + 'static,
    {
        self.search_fn = Some(Box::new(f));
        self
    }

    /// Get a reference to the items.
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Replace all items, resetting the selection to 0.
    pub fn set_items(&mut self, items: Vec<T>) {
        let len = items.len();
        self.items = items;
        self.selected = 0;
        self.scroll_offset = 0;
        self.filtered_indices = (0..len).collect();
        self.filter_query.clear();
        self.filter_active = false;
    }

    /// Get the currently selected index.
    ///
    /// When filtering is active, this is the index into the filtered list.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Set the selected index (clamped to valid range).
    pub fn set_selected(&mut self, idx: usize) {
        let count = self.visible_count();
        if count == 0 {
            self.selected = 0;
        } else {
            self.selected = idx.min(count.saturating_sub(1));
        }
    }

    /// Get a reference to the currently selected item.
    ///
    /// When filtering is active, returns the selected item from the filtered list.
    pub fn selected_item(&self) -> Option<&T> {
        if self.filter_active {
            self.filtered_indices
                .get(self.selected)
                .and_then(|&idx| self.items.get(idx))
        } else {
            self.items.get(self.selected)
        }
    }

    /// Move the selection by a delta (positive = down, negative = up).
    pub fn move_selection(&mut self, delta: isize) {
        let count = self.visible_count();
        if count == 0 {
            return;
        }
        let max_idx = count.saturating_sub(1);
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

    /// Get the number of items (total, not filtered).
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the list is empty (total, not filtered).
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    // --- Filtering API ---

    /// Enable filter mode.
    pub fn enable_filter(&mut self) {
        self.filter_active = true;
        self.update_filter();
    }

    /// Disable filter mode and restore the full list.
    pub fn disable_filter(&mut self) {
        self.filter_active = false;
        self.filter_query.clear();
        self.filtered_indices = (0..self.items.len()).collect();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Get the current filter query.
    pub fn filter_query(&self) -> &str {
        &self.filter_query
    }

    /// Set the filter query and update the filtered list.
    pub fn set_filter_query(&mut self, query: &str) {
        self.filter_query = query.to_string();
        if self.filter_active {
            self.update_filter();
        }
    }

    /// Clear the filter query (shows all items if filter is active).
    pub fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.filtered_indices = (0..self.items.len()).collect();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Check if filtering is currently active.
    pub fn is_filter_active(&self) -> bool {
        self.filter_active
    }

    /// Get references to all items that match the current filter.
    pub fn filtered_items(&self) -> Vec<&T> {
        if self.filter_active {
            self.filtered_indices
                .iter()
                .filter_map(|&idx| self.items.get(idx))
                .collect()
        } else {
            self.items.iter().collect()
        }
    }

    /// Number of currently visible items (filtered or total).
    fn visible_count(&self) -> usize {
        if self.filter_active {
            self.filtered_indices.len()
        } else {
            self.items.len()
        }
    }

    /// Get the real item index for a display row.
    fn real_index(&self, display_idx: usize) -> Option<usize> {
        if self.filter_active {
            self.filtered_indices.get(display_idx).copied()
        } else if display_idx < self.items.len() {
            Some(display_idx)
        } else {
            None
        }
    }

    /// Update filtered_indices based on current query.
    fn update_filter(&mut self) {
        if self.filter_query.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else if let Some(ref search_fn) = self.search_fn {
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<(usize, i64)> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let text = search_fn(item);
                    matcher
                        .fuzzy_match(&text, &self.filter_query)
                        .map(|score| (idx, score))
                })
                .collect();
            // Sort by score descending (best match first)
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_indices = scored.into_iter().map(|(idx, _)| idx).collect();
        } else {
            // No search function: show all items
            self.filtered_indices = (0..self.items.len()).collect();
        }

        // Reset selection
        self.selected = 0;
        self.scroll_offset = 0;
    }

    /// Ensure the selected item is visible by adjusting scroll_offset.
    fn ensure_selected_visible(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
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

        super::border::render_border(area, self.border, self.item_style.clone(), buf);

        let inner = super::border::inner_area(area, self.border);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let height = inner.size.height as usize;
        let width = inner.size.width as usize;
        let count = self.visible_count();

        // Clamp scroll offset
        let max_offset = count.saturating_sub(height.max(1));
        let scroll = self.scroll_offset.min(max_offset);

        let visible_end = (scroll + height).min(count);

        for (row, display_idx) in (scroll..visible_end).enumerate() {
            let y = inner.position.y + row as u16;
            if let Some(real_idx) = self.real_index(display_idx)
                && let Some(item) = self.items.get(real_idx)
            {
                let segments = (self.render_fn)(item);
                let is_selected = display_idx == self.selected;
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

        let count = self.visible_count();

        match code {
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if count > 0 && self.selected < count.saturating_sub(1) {
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
                if count > 0 {
                    self.selected = (self.selected + page).min(count.saturating_sub(1));
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
                if count > 0 {
                    self.selected = count.saturating_sub(1);
                    self.ensure_selected_visible(20);
                }
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(real_idx) = self.real_index(self.selected)
                    && let Some(item) = self.items.get(real_idx)
                    && let Some(ref mut callback) = self.on_select
                {
                    callback(item);
                }
                EventResult::Consumed
            }
            KeyCode::Char(ch) if self.filter_active => {
                self.filter_query.push(*ch);
                self.update_filter();
                EventResult::Consumed
            }
            KeyCode::Backspace if self.filter_active => {
                self.filter_query.pop();
                self.update_filter();
                EventResult::Consumed
            }
            KeyCode::Escape if self.filter_active => {
                self.disable_filter();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
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

    fn make_searchable_list(items: Vec<&str>) -> SelectList<String> {
        make_string_list(items).with_search_fn(|s| s.clone())
    }

    // --- Core SelectList tests (Task 2) ---

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
        assert_eq!(list.selected(), 1);
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
        list.move_selection(-100);
        assert_eq!(list.selected(), 0);
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
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some(" "));
    }

    #[test]
    fn render_with_items() {
        let list = make_string_list(vec!["Hello", "World"]);
        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        list.render(Rect::new(0, 0, 10, 5), &mut buf);
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("H"));
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some("o"));
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

        let cell_a = buf.get(0, 0);
        assert!(cell_a.is_some());
        assert!(!cell_a.map(|c| c.style.bold).unwrap_or(true));

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
        assert_eq!(list.handle_event(&down), EventResult::Consumed);
        assert_eq!(list.selected(), 2);
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

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("\u{250c}"));
        assert_eq!(buf.get(1, 1).map(|c| c.grapheme.as_str()), Some("H"));
    }

    #[test]
    fn utf8_wide_chars_in_items() {
        let list =
            SelectList::new(vec!["你好世界".to_string()]).with_render_fn(|s| vec![Segment::new(s)]);

        let mut buf = ScreenBuffer::new(Size::new(6, 1));
        list.render(Rect::new(0, 0, 6, 1), &mut buf);

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

        list.set_selected(25);
        list.ensure_selected_visible(5);
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

        let cell = buf.get(0, 0);
        assert!(cell.is_some());
        match cell.map(|c| &c.style.fg) {
            Some(Some(Color::Named(crate::color::NamedColor::Red))) => {}
            other => panic!("Expected red fg, got {other:?}"),
        }
    }

    // --- Fuzzy Filtering tests (Task 3) ---

    #[test]
    fn enable_disable_filter() {
        let mut list = make_searchable_list(vec!["Alpha", "Beta", "Gamma"]);
        assert!(!list.is_filter_active());
        list.enable_filter();
        assert!(list.is_filter_active());
        list.disable_filter();
        assert!(!list.is_filter_active());
    }

    #[test]
    fn set_filter_query_updates_indices() {
        let mut list = make_searchable_list(vec!["Apple", "Banana", "Apricot", "Cherry"]);
        list.enable_filter();
        list.set_filter_query("ap");

        let filtered = list.filtered_items();
        // "Apple" and "Apricot" should match "ap"
        assert!(filtered.len() >= 2);
        assert!(filtered.iter().any(|s| s.as_str() == "Apple"));
        assert!(filtered.iter().any(|s| s.as_str() == "Apricot"));
    }

    #[test]
    fn fuzzy_matching_works() {
        let mut list = make_searchable_list(vec!["a_b_c", "axbxc", "abc", "xyz"]);
        list.enable_filter();
        list.set_filter_query("abc");

        let filtered = list.filtered_items();
        // All of "a_b_c", "axbxc", "abc" should fuzzy-match "abc"
        assert!(filtered.len() >= 3);
        assert!(filtered.iter().any(|s| s.as_str() == "abc"));
        assert!(filtered.iter().any(|s| s.as_str() == "a_b_c"));
        assert!(filtered.iter().any(|s| s.as_str() == "axbxc"));
        // "xyz" should not match
        assert!(!filtered.iter().any(|s| s.as_str() == "xyz"));
    }

    #[test]
    fn render_filtered_list_shows_only_matches() {
        let mut list = make_searchable_list(vec!["Apple", "Banana", "Apricot"]);
        list.enable_filter();
        list.set_filter_query("ap");

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        list.render(Rect::new(0, 0, 20, 5), &mut buf);

        // Row 0 should be a matching item (Apple or Apricot)
        let c0 = buf.get(0, 0).map(|c| c.grapheme.as_str());
        assert!(c0 == Some("A")); // Both start with 'A'
    }

    #[test]
    fn selected_index_operates_on_filtered_list() {
        let mut list = make_searchable_list(vec!["Apple", "Banana", "Apricot"]);
        list.enable_filter();
        list.set_filter_query("ap");

        // Selected item should be from filtered list
        let item = list.selected_item();
        assert!(item.is_some());
        let text = item.map(|s| s.as_str()).unwrap_or("");
        assert!(text == "Apple" || text == "Apricot");
    }

    #[test]
    fn navigation_on_filtered_list() {
        let mut list = make_searchable_list(vec!["Apple", "Banana", "Apricot", "Cherry"]);
        list.enable_filter();
        list.set_filter_query("ap");

        let down = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });

        // Move down within filtered items
        list.handle_event(&down);
        assert_eq!(list.selected(), 1);
        let item = list.selected_item();
        assert!(item.is_some());
    }

    #[test]
    fn clear_filter_restores_full_list() {
        let mut list = make_searchable_list(vec!["Apple", "Banana", "Cherry"]);
        list.enable_filter();
        list.set_filter_query("ap");

        let filtered_count = list.filtered_items().len();
        assert!(filtered_count < 3);

        list.clear_filter();
        assert_eq!(list.filtered_items().len(), 3);
    }

    #[test]
    fn backspace_removes_filter_chars() {
        let mut list = make_searchable_list(vec!["Apple", "Banana", "Cherry"]);
        list.enable_filter();
        list.set_filter_query("xyz");
        assert!(list.filtered_items().is_empty());

        // Backspace via event
        let backspace = Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: crate::event::Modifiers::NONE,
        });

        list.handle_event(&backspace); // remove 'z'
        list.handle_event(&backspace); // remove 'y'
        list.handle_event(&backspace); // remove 'x'
        assert_eq!(list.filter_query(), "");
        // Empty query shows all
        assert_eq!(list.filtered_items().len(), 3);
    }

    #[test]
    fn esc_clears_and_disables_filter() {
        let mut list = make_searchable_list(vec!["Apple", "Banana"]);
        list.enable_filter();
        list.set_filter_query("ap");
        assert!(list.is_filter_active());

        let esc = Event::Key(KeyEvent {
            code: KeyCode::Escape,
            modifiers: crate::event::Modifiers::NONE,
        });

        list.handle_event(&esc);
        assert!(!list.is_filter_active());
        assert_eq!(list.filter_query(), "");
    }

    #[test]
    fn empty_query_shows_all_items() {
        let mut list = make_searchable_list(vec!["A", "B", "C"]);
        list.enable_filter();
        list.set_filter_query("");

        assert_eq!(list.filtered_items().len(), 3);
    }

    #[test]
    fn no_matches_empty_filtered_list() {
        let mut list = make_searchable_list(vec!["Apple", "Banana"]);
        list.enable_filter();
        list.set_filter_query("zzzzz");

        assert!(list.filtered_items().is_empty());
    }

    #[test]
    fn filter_with_custom_search_fn() {
        #[derive(Clone)]
        struct Item {
            name: String,
            tag: String,
        }

        let items = vec![
            Item {
                name: "Apple".into(),
                tag: "fruit".into(),
            },
            Item {
                name: "Carrot".into(),
                tag: "veggie".into(),
            },
            Item {
                name: "Banana".into(),
                tag: "fruit".into(),
            },
        ];

        let mut list = SelectList::new(items)
            .with_render_fn(|item| vec![Segment::new(&item.name)])
            .with_search_fn(|item| item.tag.clone());

        list.enable_filter();
        list.set_filter_query("fruit");

        let filtered = list.filtered_items();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|item| item.tag == "fruit"));
    }

    #[test]
    fn char_input_triggers_filter_update() {
        let mut list = make_searchable_list(vec!["Apple", "Banana", "Apricot"]);
        list.enable_filter();

        let char_a = Event::Key(KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: crate::event::Modifiers::NONE,
        });
        let char_p = Event::Key(KeyEvent {
            code: KeyCode::Char('p'),
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(list.handle_event(&char_a), EventResult::Consumed);
        assert_eq!(list.filter_query(), "a");

        assert_eq!(list.handle_event(&char_p), EventResult::Consumed);
        assert_eq!(list.filter_query(), "ap");

        // Should have filtered to items containing "ap"
        let filtered = list.filtered_items();
        assert!(filtered.len() >= 2);
    }

    #[test]
    fn utf8_safe_query_input() {
        let mut list = make_searchable_list(vec!["你好", "世界", "你世"]);
        list.enable_filter();
        list.set_filter_query("你");

        let filtered = list.filtered_items();
        assert!(filtered.len() >= 2);
        assert!(filtered.iter().any(|s| s.as_str() == "你好"));
        assert!(filtered.iter().any(|s| s.as_str() == "你世"));
    }

    #[test]
    fn enter_on_filtered_list_selects_correct_item() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let selected_value = Rc::new(RefCell::new(String::new()));
        let captured = Rc::clone(&selected_value);

        let mut list = make_searchable_list(vec!["Apple", "Banana", "Apricot"]).with_on_select(
            move |item: &String| {
                *captured.borrow_mut() = item.clone();
            },
        );

        list.enable_filter();
        list.set_filter_query("ap");

        // Navigate to second filtered item
        let down = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });
        list.handle_event(&down);

        let enter = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: crate::event::Modifiers::NONE,
        });
        list.handle_event(&enter);

        // Should have selected one of the "ap" matching items
        let val = selected_value.borrow().clone();
        assert!(val == "Apple" || val == "Apricot");
    }
}
