//! Scrollable log widget that displays styled entries.
//!
//! Each entry is a line of [`Segment`]s. The log supports vertical
//! scrolling via keyboard and optional auto-scrolling to the bottom
//! when new entries are added.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use unicode_width::UnicodeWidthStr;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// A scrollable log widget that displays styled entries.
///
/// Each entry is a vector of [`Segment`]s representing one line.
/// Supports vertical scrolling and optional auto-scroll to bottom.
#[derive(Clone, Debug)]
pub struct RichLog {
    /// Log entries: each entry is a line of segments.
    entries: Vec<Vec<Segment>>,
    /// Index of the first visible entry.
    scroll_offset: usize,
    /// Base style for the log area.
    style: Style,
    /// Whether to auto-scroll to bottom when entries are added.
    auto_scroll: bool,
    /// Border style (optional).
    border: BorderStyle,
}

impl RichLog {
    /// Create a new empty log.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            scroll_offset: 0,
            style: Style::default(),
            auto_scroll: true,
            border: BorderStyle::None,
        }
    }

    /// Set the base style for the log area.
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Enable or disable auto-scrolling to the bottom on new entries.
    #[must_use]
    pub fn with_auto_scroll(mut self, enabled: bool) -> Self {
        self.auto_scroll = enabled;
        self
    }

    /// Add a log entry (single line of segments).
    pub fn push(&mut self, entry: Vec<Segment>) {
        self.entries.push(entry);
        if self.auto_scroll {
            // Will be applied on next render based on visible height;
            // for now, set offset to show last entry.
            // We use saturating_sub to handle the case where we don't know
            // the visible height yet - scroll_to_bottom() can be called
            // explicitly or it adjusts in render.
            self.scroll_offset = self.entries.len().saturating_sub(1);
        }
    }

    /// Add a plain text entry (convenience method).
    pub fn push_text(&mut self, text: &str) {
        self.entries.push(vec![Segment::new(text)]);
        if self.auto_scroll {
            self.scroll_offset = self.entries.len().saturating_sub(1);
        }
    }

    /// Clear all entries and reset scroll.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
    }

    /// Get total entry count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the log has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Scroll to the bottom (last entry visible).
    pub fn scroll_to_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.scroll_offset = self.entries.len().saturating_sub(1);
        }
    }

    /// Scroll to the top (first entry visible).
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Get the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
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
        if chars.is_none() {
            return;
        }
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
        buf.set(x1, y1, Cell::new(tl, self.style.clone()));
        buf.set(x2, y1, Cell::new(tr, self.style.clone()));
        buf.set(x1, y2, Cell::new(bl, self.style.clone()));
        buf.set(x2, y2, Cell::new(br, self.style.clone()));

        // Top and bottom edges
        for x in (x1 + 1)..x2 {
            buf.set(x, y1, Cell::new(h, self.style.clone()));
            buf.set(x, y2, Cell::new(h, self.style.clone()));
        }

        // Left and right edges
        for y in (y1 + 1)..y2 {
            buf.set(x1, y, Cell::new(v, self.style.clone()));
            buf.set(x2, y, Cell::new(v, self.style.clone()));
        }
    }
}

impl Default for RichLog {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for RichLog {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        // Render border if any
        self.render_border(area, buf);

        let inner = self.inner_area(area);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let height = inner.size.height as usize;
        let width = inner.size.width as usize;

        // Clamp scroll offset (use a local copy since render takes &self)
        let max_offset = self.entries.len().saturating_sub(height.max(1));
        let scroll = self.scroll_offset.min(max_offset);

        let visible_end = (scroll + height).min(self.entries.len());

        for (row, entry_idx) in (scroll..visible_end).enumerate() {
            let y = inner.position.y + row as u16;
            if let Some(entry) = self.entries.get(entry_idx) {
                let mut col: u16 = 0;
                for segment in entry {
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
                        buf.set(x, y, Cell::new(ch.to_string(), segment.style.clone()));
                        col += char_w as u16;
                    }
                }
            }
        }
    }
}

impl InteractiveWidget for RichLog {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        match code {
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                    self.auto_scroll = false;
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if !self.entries.is_empty()
                    && self.scroll_offset < self.entries.len().saturating_sub(1)
                {
                    self.scroll_offset += 1;
                    self.auto_scroll = false;
                }
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                // Scroll by a page (assume ~20 lines if we don't know height)
                let page = 20;
                self.scroll_offset = self.scroll_offset.saturating_sub(page);
                self.auto_scroll = false;
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                let page = 20;
                if !self.entries.is_empty() {
                    self.scroll_offset =
                        (self.scroll_offset + page).min(self.entries.len().saturating_sub(1));
                    self.auto_scroll = false;
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.scroll_to_top();
                self.auto_scroll = false;
                EventResult::Consumed
            }
            KeyCode::End => {
                self.scroll_to_bottom();
                // Scrolling to end re-enables auto_scroll behavior
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

/// Return border characters for the given style, or None for BorderStyle::None.
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
    use crate::geometry::Size;
    use crate::style::Style;

    fn make_segment(text: &str) -> Segment {
        Segment::new(text)
    }

    fn styled_segment(text: &str, style: Style) -> Segment {
        Segment::styled(text, style)
    }

    #[test]
    fn new_log_is_empty() {
        let log = RichLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
        assert_eq!(log.scroll_offset(), 0);
    }

    #[test]
    fn default_matches_new() {
        let log: RichLog = Default::default();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);
    }

    #[test]
    fn push_adds_entries() {
        let mut log = RichLog::new();
        log.push(vec![make_segment("line 1")]);
        log.push(vec![make_segment("line 2")]);
        assert_eq!(log.len(), 2);
        assert!(!log.is_empty());
    }

    #[test]
    fn push_text_convenience() {
        let mut log = RichLog::new();
        log.push_text("hello");
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn clear_resets() {
        let mut log = RichLog::new();
        log.push_text("a");
        log.push_text("b");
        log.clear();
        assert!(log.is_empty());
        assert_eq!(log.scroll_offset(), 0);
    }

    #[test]
    fn render_empty_log() {
        let log = RichLog::new();
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        log.render(Rect::new(0, 0, 20, 5), &mut buf);
        // Should not panic; area remains blank
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some(" "));
    }

    #[test]
    fn render_with_entries() {
        let mut log = RichLog::new().with_auto_scroll(false);
        log.push_text("hello");
        log.push_text("world");

        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        log.render(Rect::new(0, 0, 10, 5), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("h"));
        assert_eq!(buf.get(1, 0).map(|c| c.grapheme.as_str()), Some("e"));
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("w"));
    }

    #[test]
    fn render_with_multi_segment_entries() {
        let mut log = RichLog::new().with_auto_scroll(false);
        let bold = Style::new().bold(true);
        log.push(vec![styled_segment("bold", bold), make_segment(" normal")]);

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        log.render(Rect::new(0, 0, 20, 5), &mut buf);

        // 'b' should be bold
        let cell_b = buf.get(0, 0);
        assert!(cell_b.is_some());
        assert_eq!(cell_b.map(|c| c.grapheme.as_str()), Some("b"));
        assert!(cell_b.map(|c| c.style.bold).unwrap_or(false));

        // ' ' after "bold" should be normal
        let cell_space = buf.get(4, 0);
        assert_eq!(cell_space.map(|c| c.grapheme.as_str()), Some(" "));
    }

    #[test]
    fn render_with_border() {
        let mut log = RichLog::new()
            .with_border(BorderStyle::Single)
            .with_auto_scroll(false);
        log.push_text("hi");

        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        log.render(Rect::new(0, 0, 10, 5), &mut buf);

        // Top-left corner should be box drawing char
        let corner = buf.get(0, 0).map(|c| c.grapheme.as_str());
        assert_eq!(corner, Some("\u{250c}"));

        // Content at (1, 1) inside border
        assert_eq!(buf.get(1, 1).map(|c| c.grapheme.as_str()), Some("h"));
    }

    #[test]
    fn scroll_operations() {
        let mut log = RichLog::new().with_auto_scroll(false);
        for i in 0..20 {
            log.push_text(&format!("line {i}"));
        }

        log.scroll_to_bottom();
        assert_eq!(log.scroll_offset(), 19);

        log.scroll_to_top();
        assert_eq!(log.scroll_offset(), 0);
    }

    #[test]
    fn auto_scroll_on_push() {
        let mut log = RichLog::new().with_auto_scroll(true);
        log.push_text("a");
        assert_eq!(log.scroll_offset(), 0);
        log.push_text("b");
        assert_eq!(log.scroll_offset(), 1);
        log.push_text("c");
        assert_eq!(log.scroll_offset(), 2);
    }

    #[test]
    fn manual_scroll_disables_auto_scroll() {
        let mut log = RichLog::new().with_auto_scroll(true);
        for _ in 0..10 {
            log.push_text("line");
        }

        // Scroll up manually
        let event = Event::Key(KeyEvent::plain(KeyCode::Up));
        let result = log.handle_event(&event);
        assert_eq!(result, EventResult::Consumed);

        // Auto-scroll should now be disabled
        let prev_offset = log.scroll_offset();
        log.push_text("new line");
        // Offset should NOT auto-scroll because auto_scroll is disabled
        assert_eq!(log.scroll_offset(), prev_offset);
    }

    #[test]
    fn keyboard_navigation() {
        let mut log = RichLog::new().with_auto_scroll(false);
        for i in 0..30 {
            log.push_text(&format!("line {i}"));
        }

        // Down key
        let down = Event::Key(KeyEvent::plain(KeyCode::Down));
        log.handle_event(&down);
        assert_eq!(log.scroll_offset(), 1);

        // Up key
        let up = Event::Key(KeyEvent::plain(KeyCode::Up));
        log.handle_event(&up);
        assert_eq!(log.scroll_offset(), 0);

        // Up at top stays at 0
        log.handle_event(&up);
        assert_eq!(log.scroll_offset(), 0);

        // Page down
        let pgdn = Event::Key(KeyEvent::plain(KeyCode::PageDown));
        log.handle_event(&pgdn);
        assert_eq!(log.scroll_offset(), 20);

        // Page up
        let pgup = Event::Key(KeyEvent::plain(KeyCode::PageUp));
        log.handle_event(&pgup);
        assert_eq!(log.scroll_offset(), 0);

        // End key
        let end = Event::Key(KeyEvent::plain(KeyCode::End));
        log.handle_event(&end);
        assert_eq!(log.scroll_offset(), 29);

        // Home key
        let home = Event::Key(KeyEvent::plain(KeyCode::Home));
        log.handle_event(&home);
        assert_eq!(log.scroll_offset(), 0);
    }

    #[test]
    fn empty_log_keyboard_events_graceful() {
        let mut log = RichLog::new();
        let down = Event::Key(KeyEvent::plain(KeyCode::Down));
        let result = log.handle_event(&down);
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(log.scroll_offset(), 0);
    }

    #[test]
    fn utf8_safety_wide_chars() {
        let mut log = RichLog::new().with_auto_scroll(false);
        log.push_text("日本語テスト");
        log.push_text("Hello 🎉 World");

        let mut buf = ScreenBuffer::new(Size::new(10, 5));
        log.render(Rect::new(0, 0, 10, 5), &mut buf);

        // Should not panic, and content should be truncated to width
        let first_cell = buf.get(0, 0).map(|c| c.grapheme.as_str());
        assert_eq!(first_cell, Some("日"));
    }

    #[test]
    fn overflow_truncation() {
        let mut log = RichLog::new().with_auto_scroll(false);
        log.push_text("This is a very long line that should be truncated to fit");

        let mut buf = ScreenBuffer::new(Size::new(10, 1));
        log.render(Rect::new(0, 0, 10, 1), &mut buf);

        // Only first 10 chars should appear: "This is a "
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("T"));
        assert_eq!(buf.get(4, 0).map(|c| c.grapheme.as_str()), Some(" "));
        assert_eq!(buf.get(5, 0).map(|c| c.grapheme.as_str()), Some("i"));
    }

    #[test]
    fn unhandled_event_returns_ignored() {
        let mut log = RichLog::new();
        let event = Event::Key(KeyEvent::plain(KeyCode::Char('a')));
        assert_eq!(log.handle_event(&event), EventResult::Ignored);
    }

    #[test]
    fn builder_pattern() {
        let log = RichLog::new()
            .with_style(Style::new().bold(true))
            .with_border(BorderStyle::Rounded)
            .with_auto_scroll(false);

        assert!(!log.auto_scroll);
        assert!(matches!(log.border, BorderStyle::Rounded));
    }
}
