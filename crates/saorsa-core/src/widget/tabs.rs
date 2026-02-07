//! Tabbed content switcher widget.
//!
//! Displays a tab bar with selectable tabs and a content area showing
//! the active tab's content. Supports keyboard navigation, closable tabs,
//! and configurable tab bar positioning.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent, Modifiers};
use crate::geometry::Rect;
use crate::segment::Segment;
use crate::style::Style;
use crate::text::{string_display_width, truncate_to_display_width};
use unicode_width::UnicodeWidthStr;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// A single tab definition.
#[derive(Clone, Debug)]
pub struct Tab {
    /// Tab label text.
    pub label: String,
    /// Tab content as styled Segment lines.
    pub content: Vec<Vec<Segment>>,
    /// Whether this tab can be closed.
    pub closable: bool,
}

impl Tab {
    /// Create a new tab with the given label.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            content: Vec::new(),
            closable: false,
        }
    }

    /// Set the tab content.
    #[must_use]
    pub fn with_content(mut self, content: Vec<Vec<Segment>>) -> Self {
        self.content = content;
        self
    }

    /// Set whether this tab is closable.
    #[must_use]
    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

/// Position of the tab bar relative to content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarPosition {
    /// Tab bar appears at the top.
    Top,
    /// Tab bar appears at the bottom.
    Bottom,
}

/// Tabbed content switcher widget.
///
/// Displays a horizontal tab bar and the content of the active tab.
/// Supports keyboard navigation with Left/Right arrows, Tab/Shift+Tab,
/// and closing tabs with Ctrl+W.
pub struct Tabs {
    /// Tab definitions.
    tabs: Vec<Tab>,
    /// Active tab index.
    active_tab: usize,
    /// Style for the tab bar background.
    tab_bar_style: Style,
    /// Style for the active tab label.
    active_tab_style: Style,
    /// Style for inactive tab labels.
    inactive_tab_style: Style,
    /// Style for the content area.
    content_style: Style,
    /// Border style around the entire widget.
    border: BorderStyle,
    /// Tab bar position (top or bottom).
    tab_bar_position: TabBarPosition,
}

impl Tabs {
    /// Create a new tabbed widget with the given tabs.
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self {
            tabs,
            active_tab: 0,
            tab_bar_style: Style::default(),
            active_tab_style: Style::default().reverse(true),
            inactive_tab_style: Style::default(),
            content_style: Style::default(),
            border: BorderStyle::None,
            tab_bar_position: TabBarPosition::Top,
        }
    }

    /// Set the tab bar background style.
    #[must_use]
    pub fn with_tab_bar_style(mut self, style: Style) -> Self {
        self.tab_bar_style = style;
        self
    }

    /// Set the active tab label style.
    #[must_use]
    pub fn with_active_tab_style(mut self, style: Style) -> Self {
        self.active_tab_style = style;
        self
    }

    /// Set the inactive tab label style.
    #[must_use]
    pub fn with_inactive_tab_style(mut self, style: Style) -> Self {
        self.inactive_tab_style = style;
        self
    }

    /// Set the content area style.
    #[must_use]
    pub fn with_content_style(mut self, style: Style) -> Self {
        self.content_style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set the tab bar position.
    #[must_use]
    pub fn with_tab_bar_position(mut self, pos: TabBarPosition) -> Self {
        self.tab_bar_position = pos;
        self
    }

    /// Add a tab to the end of the tab list.
    pub fn add_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
    }

    /// Get the active tab index.
    pub fn active_tab(&self) -> usize {
        self.active_tab
    }

    /// Set the active tab index (clamped to valid range).
    pub fn set_active_tab(&mut self, idx: usize) {
        if self.tabs.is_empty() {
            self.active_tab = 0;
        } else {
            self.active_tab = idx.min(self.tabs.len().saturating_sub(1));
        }
    }

    /// Get the content of the active tab.
    pub fn active_content(&self) -> Option<&[Vec<Segment>]> {
        self.tabs.get(self.active_tab).map(|t| t.content.as_slice())
    }

    /// Close the tab at the given index if it is closable.
    ///
    /// Returns `true` if the tab was closed.
    pub fn close_tab(&mut self, idx: usize) -> bool {
        if let Some(tab) = self.tabs.get(idx) {
            if !tab.closable {
                return false;
            }
        } else {
            return false;
        }

        self.tabs.remove(idx);

        // Adjust active tab if needed
        if self.tabs.is_empty() {
            self.active_tab = 0;
        } else if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len().saturating_sub(1);
        }

        true
    }

    /// Get the number of tabs.
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Move to the next tab (wraps around).
    fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    /// Move to the previous tab (wraps around).
    fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            if self.active_tab == 0 {
                self.active_tab = self.tabs.len().saturating_sub(1);
            } else {
                self.active_tab -= 1;
            }
        }
    }

    /// Render the tab bar into the buffer at the given row.
    fn render_tab_bar(&self, area_x: u16, area_y: u16, width: u16, buf: &mut ScreenBuffer) {
        if width == 0 {
            return;
        }

        // Fill background
        for x in 0..width {
            buf.set(
                area_x + x,
                area_y,
                Cell::new(" ", self.tab_bar_style.clone()),
            );
        }

        let mut col: u16 = 0;
        let w = width as usize;

        for (i, tab) in self.tabs.iter().enumerate() {
            if col as usize >= w {
                break;
            }

            // Separator between tabs
            if i > 0 && (col as usize) < w {
                buf.set(
                    area_x + col,
                    area_y,
                    Cell::new("│", self.tab_bar_style.clone()),
                );
                col += 1;
                if col as usize >= w {
                    break;
                }
            }

            let style = if i == self.active_tab {
                self.active_tab_style.clone()
            } else {
                self.inactive_tab_style.clone()
            };

            // Build label: " label " or " label × "
            let close_suffix = if tab.closable { " ×" } else { "" };
            let label_with_padding = format!(" {}{} ", tab.label, close_suffix);

            let remaining = w.saturating_sub(col as usize);
            let truncated = truncate_to_display_width(&label_with_padding, remaining);
            let display_w = string_display_width(truncated);

            for ch in truncated.chars() {
                if col as usize >= w {
                    break;
                }
                let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
                if col as usize + char_w > w {
                    break;
                }
                buf.set(
                    area_x + col,
                    area_y,
                    Cell::new(ch.to_string(), style.clone()),
                );
                col += char_w as u16;
            }

            // Suppress unused warning — display_w used for accounting
            let _ = display_w;
        }
    }

    /// Render content lines into the buffer.
    fn render_content(
        &self,
        area_x: u16,
        area_y: u16,
        width: u16,
        height: u16,
        buf: &mut ScreenBuffer,
    ) {
        let content = match self.active_content() {
            Some(c) => c,
            None => return,
        };

        let w = width as usize;

        for (row, line) in content.iter().enumerate() {
            if row >= height as usize {
                break;
            }
            let y = area_y + row as u16;
            let mut col: u16 = 0;

            for segment in line {
                if col as usize >= w {
                    break;
                }
                let remaining = w.saturating_sub(col as usize);
                let truncated = truncate_to_display_width(&segment.text, remaining);

                for ch in truncated.chars() {
                    if col as usize >= w {
                        break;
                    }
                    let char_w = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
                    if col as usize + char_w > w {
                        break;
                    }
                    let x = area_x + col;
                    buf.set(x, y, Cell::new(ch.to_string(), segment.style.clone()));
                    col += char_w as u16;
                }
            }
        }
    }
}

impl Widget for Tabs {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        // Render border if any
        super::border::render_border(area, self.border, self.tab_bar_style.clone(), buf);

        let inner = super::border::inner_area(area, self.border);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        let w = inner.size.width;
        let h = inner.size.height;

        if h == 0 {
            return;
        }

        match self.tab_bar_position {
            TabBarPosition::Top => {
                // Tab bar at row 0
                self.render_tab_bar(inner.position.x, inner.position.y, w, buf);
                // Content below
                if h > 1 {
                    self.render_content(inner.position.x, inner.position.y + 1, w, h - 1, buf);
                }
            }
            TabBarPosition::Bottom => {
                // Content first
                if h > 1 {
                    self.render_content(inner.position.x, inner.position.y, w, h - 1, buf);
                }
                // Tab bar at last row
                let bar_y = inner.position.y + h - 1;
                self.render_tab_bar(inner.position.x, bar_y, w, buf);
            }
        }
    }
}

impl InteractiveWidget for Tabs {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event
        else {
            return EventResult::Ignored;
        };

        match code {
            KeyCode::Left => {
                self.prev_tab();
                EventResult::Consumed
            }
            KeyCode::Right => {
                self.next_tab();
                EventResult::Consumed
            }
            KeyCode::Tab if !modifiers.contains(Modifiers::SHIFT) => {
                self.next_tab();
                EventResult::Consumed
            }
            KeyCode::Tab if modifiers.contains(Modifiers::SHIFT) => {
                self.prev_tab();
                EventResult::Consumed
            }
            // Backtab often sent for Shift+Tab
            KeyCode::Char('w') if modifiers.contains(Modifiers::CTRL) => {
                self.close_tab(self.active_tab);
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
    use crate::geometry::Size;

    fn make_tab(label: &str, lines: &[&str]) -> Tab {
        Tab {
            label: label.to_string(),
            content: lines.iter().map(|l| vec![Segment::new(*l)]).collect(),
            closable: false,
        }
    }

    fn make_closable_tab(label: &str) -> Tab {
        Tab {
            label: label.to_string(),
            content: vec![vec![Segment::new("content")]],
            closable: true,
        }
    }

    #[test]
    fn create_with_multiple_tabs() {
        let tabs = Tabs::new(vec![
            make_tab("Tab1", &["line1"]),
            make_tab("Tab2", &["line2"]),
            make_tab("Tab3", &["line3"]),
        ]);
        assert_eq!(tabs.tab_count(), 3);
        assert_eq!(tabs.active_tab(), 0);
    }

    #[test]
    fn render_tab_bar_at_top() {
        let tabs = Tabs::new(vec![
            make_tab("Alpha", &["content A"]),
            make_tab("Beta", &["content B"]),
        ]);
        let mut buf = ScreenBuffer::new(Size::new(40, 5));
        tabs.render(Rect::new(0, 0, 40, 5), &mut buf);

        // Tab bar at row 0 — active tab should have "Alpha" label
        let row0: String = (0..40)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect::<String>();
        assert!(row0.contains("Alpha"));
        assert!(row0.contains("Beta"));
    }

    #[test]
    fn render_tab_bar_at_bottom() {
        let tabs = Tabs::new(vec![make_tab("X", &["content"]), make_tab("Y", &["data"])])
            .with_tab_bar_position(TabBarPosition::Bottom);

        let mut buf = ScreenBuffer::new(Size::new(30, 4));
        tabs.render(Rect::new(0, 0, 30, 4), &mut buf);

        // Tab bar at last row (row 3)
        let last_row: String = (0..30)
            .map(|x| buf.get(x, 3).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect::<String>();
        assert!(last_row.contains("X"));
        assert!(last_row.contains("Y"));
    }

    #[test]
    fn active_tab_content() {
        let tabs = Tabs::new(vec![make_tab("A", &["line A"]), make_tab("B", &["line B"])]);

        let content = tabs.active_content().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0][0].text, "line A");
    }

    #[test]
    fn switch_tabs_left_right() {
        let mut tabs = Tabs::new(vec![
            make_tab("1", &[]),
            make_tab("2", &[]),
            make_tab("3", &[]),
        ]);
        assert_eq!(tabs.active_tab(), 0);

        // Right → tab 1
        tabs.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Right)));
        assert_eq!(tabs.active_tab(), 1);

        // Right → tab 2
        tabs.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Right)));
        assert_eq!(tabs.active_tab(), 2);

        // Right wraps → tab 0
        tabs.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Right)));
        assert_eq!(tabs.active_tab(), 0);

        // Left wraps → tab 2
        tabs.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Left)));
        assert_eq!(tabs.active_tab(), 2);
    }

    #[test]
    fn tab_key_navigation() {
        let mut tabs = Tabs::new(vec![make_tab("A", &[]), make_tab("B", &[])]);

        // Tab key → next
        let result = tabs.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Tab)));
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(tabs.active_tab(), 1);

        // Shift+Tab → previous
        let result = tabs.handle_event(&Event::Key(KeyEvent::new(KeyCode::Tab, Modifiers::SHIFT)));
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(tabs.active_tab(), 0);
    }

    #[test]
    fn close_closable_tab() {
        let mut tabs = Tabs::new(vec![
            make_closable_tab("C1"),
            make_closable_tab("C2"),
            make_closable_tab("C3"),
        ]);

        tabs.set_active_tab(1);
        assert!(tabs.close_tab(1));
        assert_eq!(tabs.tab_count(), 2);
        // active_tab adjusted to 1 (was pointing at C2, now C3 is at index 1)
        assert_eq!(tabs.active_tab(), 1);
    }

    #[test]
    fn non_closable_tab_ignores_close() {
        let mut tabs = Tabs::new(vec![make_tab("Fixed", &["data"])]);
        assert!(!tabs.close_tab(0));
        assert_eq!(tabs.tab_count(), 1);
    }

    #[test]
    fn empty_tabs_list() {
        let tabs = Tabs::new(vec![]);
        assert_eq!(tabs.tab_count(), 0);
        assert_eq!(tabs.active_tab(), 0);
        assert!(tabs.active_content().is_none());

        // Render should not panic
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        tabs.render(Rect::new(0, 0, 20, 5), &mut buf);
    }

    #[test]
    fn single_tab() {
        let tabs = Tabs::new(vec![make_tab("Only", &["data"])]);
        assert_eq!(tabs.tab_count(), 1);
        assert_eq!(tabs.active_tab(), 0);

        let content = tabs.active_content().unwrap();
        assert_eq!(content[0][0].text, "data");
    }

    #[test]
    fn set_active_tab_clamping() {
        let mut tabs = Tabs::new(vec![make_tab("A", &[]), make_tab("B", &[])]);

        tabs.set_active_tab(100);
        assert_eq!(tabs.active_tab(), 1); // clamped to last

        tabs.set_active_tab(0);
        assert_eq!(tabs.active_tab(), 0);
    }

    #[test]
    fn utf8_safe_tab_labels() {
        let tabs = Tabs::new(vec![
            make_tab("日本語", &["content"]),
            make_tab("中文标签", &["data"]),
        ]);

        let mut buf = ScreenBuffer::new(Size::new(40, 3));
        tabs.render(Rect::new(0, 0, 40, 3), &mut buf);

        let row0: String = (0..40)
            .map(|x| buf.get(x, 0).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect::<String>();
        assert!(row0.contains("日本語"));
    }

    #[test]
    fn border_rendering() {
        let tabs = Tabs::new(vec![make_tab("T", &["c"])]).with_border(BorderStyle::Single);

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        tabs.render(Rect::new(0, 0, 20, 5), &mut buf);

        // Top-left corner should be single border
        assert_eq!(buf.get(0, 0).unwrap().grapheme, "┌");
        assert_eq!(buf.get(19, 0).unwrap().grapheme, "┐");
    }

    #[test]
    fn add_tab() {
        let mut tabs = Tabs::new(vec![make_tab("A", &[])]);
        tabs.add_tab(make_tab("B", &[]));
        assert_eq!(tabs.tab_count(), 2);
    }

    #[test]
    fn ctrl_w_closes_active_tab() {
        let mut tabs = Tabs::new(vec![make_closable_tab("X"), make_closable_tab("Y")]);
        tabs.set_active_tab(0);

        let result = tabs.handle_event(&Event::Key(KeyEvent::new(
            KeyCode::Char('w'),
            Modifiers::CTRL,
        )));
        assert_eq!(result, EventResult::Consumed);
        assert_eq!(tabs.tab_count(), 1);
    }

    #[test]
    fn unhandled_event_ignored() {
        let mut tabs = Tabs::new(vec![make_tab("A", &[])]);
        let result = tabs.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Char('z'))));
        assert_eq!(result, EventResult::Ignored);
    }
}
