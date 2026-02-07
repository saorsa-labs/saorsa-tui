//! Diff viewer widget with unified and side-by-side display modes.
//!
//! Uses the [`similar`] crate to compute line-by-line diffs and displays
//! them with color-coded added/removed/unchanged styling.

use similar::{ChangeTag, TextDiff};

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::style::Style;
use crate::text::truncate_to_display_width;

use super::{BorderStyle, EventResult, InteractiveWidget, Widget};

/// Display mode for the diff viewer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffMode {
    /// Unified diff: single column with +/- prefixes.
    Unified,
    /// Side-by-side diff: old on left, new on right.
    SideBySide,
}

/// A line in the computed diff.
#[derive(Clone, Debug)]
struct DiffLine {
    /// The diff tag for this line.
    tag: ChangeTag,
    /// The text content (without newline).
    text: String,
}

/// A pair of lines for side-by-side display.
#[derive(Clone, Debug)]
struct SideBySidePair {
    /// Left (old) line, if any.
    left: Option<DiffLine>,
    /// Right (new) line, if any.
    right: Option<DiffLine>,
}

/// A diff viewer widget that displays text differences.
///
/// Supports unified and side-by-side display modes with color-coded
/// added, removed, and unchanged lines.
pub struct DiffView {
    /// Original text.
    old_text: String,
    /// Modified text.
    new_text: String,
    /// Display mode.
    mode: DiffMode,
    /// Scroll offset (first visible line).
    scroll_offset: usize,
    /// Style for unchanged lines.
    unchanged_style: Style,
    /// Style for added lines.
    added_style: Style,
    /// Style for removed lines.
    removed_style: Style,
    /// Border style.
    border: BorderStyle,
    /// Cached unified diff lines.
    unified_lines: Vec<DiffLine>,
    /// Cached side-by-side pairs.
    sbs_pairs: Vec<SideBySidePair>,
}

impl DiffView {
    /// Create a new diff view comparing old and new text.
    pub fn new(old_text: &str, new_text: &str) -> Self {
        let mut view = Self {
            old_text: old_text.to_string(),
            new_text: new_text.to_string(),
            mode: DiffMode::Unified,
            scroll_offset: 0,
            unchanged_style: Style::default(),
            added_style: Style::default()
                .bg(crate::color::Color::Named(crate::color::NamedColor::Green)),
            removed_style: Style::default()
                .bg(crate::color::Color::Named(crate::color::NamedColor::Red)),
            border: BorderStyle::None,
            unified_lines: Vec::new(),
            sbs_pairs: Vec::new(),
        };
        view.compute_diff();
        view
    }

    /// Set the display mode.
    #[must_use]
    pub fn with_mode(mut self, mode: DiffMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the style for unchanged lines.
    #[must_use]
    pub fn with_unchanged_style(mut self, style: Style) -> Self {
        self.unchanged_style = style;
        self
    }

    /// Set the style for added lines.
    #[must_use]
    pub fn with_added_style(mut self, style: Style) -> Self {
        self.added_style = style;
        self
    }

    /// Set the style for removed lines.
    #[must_use]
    pub fn with_removed_style(mut self, style: Style) -> Self {
        self.removed_style = style;
        self
    }

    /// Set the border style.
    #[must_use]
    pub fn with_border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    /// Set new texts and recompute the diff.
    pub fn set_texts(&mut self, old_text: &str, new_text: &str) {
        self.old_text = old_text.to_string();
        self.new_text = new_text.to_string();
        self.scroll_offset = 0;
        self.compute_diff();
    }

    /// Switch display mode.
    pub fn set_mode(&mut self, mode: DiffMode) {
        self.mode = mode;
        self.scroll_offset = 0;
    }

    /// Get the current display mode.
    pub fn mode(&self) -> DiffMode {
        self.mode
    }

    /// Get the total number of display lines for the current mode.
    pub fn line_count(&self) -> usize {
        match self.mode {
            DiffMode::Unified => self.unified_lines.len(),
            DiffMode::SideBySide => self.sbs_pairs.len(),
        }
    }

    /// Get the scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Compute the diff between old and new text.
    fn compute_diff(&mut self) {
        let diff = TextDiff::from_lines(&self.old_text, &self.new_text);

        // Build unified lines
        self.unified_lines.clear();
        for change in diff.iter_all_changes() {
            let text = change.to_string_lossy().trim_end_matches('\n').to_string();
            self.unified_lines.push(DiffLine {
                tag: change.tag(),
                text,
            });
        }

        // Build side-by-side pairs
        self.sbs_pairs.clear();
        let mut old_lines: Vec<DiffLine> = Vec::new();
        let mut new_lines: Vec<DiffLine> = Vec::new();

        for change in diff.iter_all_changes() {
            let text = change.to_string_lossy().trim_end_matches('\n').to_string();
            match change.tag() {
                ChangeTag::Equal => {
                    flush_sbs_pairs(&mut self.sbs_pairs, &mut old_lines, &mut new_lines);
                    self.sbs_pairs.push(SideBySidePair {
                        left: Some(DiffLine {
                            tag: ChangeTag::Equal,
                            text: text.clone(),
                        }),
                        right: Some(DiffLine {
                            tag: ChangeTag::Equal,
                            text,
                        }),
                    });
                }
                ChangeTag::Delete => {
                    old_lines.push(DiffLine {
                        tag: ChangeTag::Delete,
                        text,
                    });
                }
                ChangeTag::Insert => {
                    new_lines.push(DiffLine {
                        tag: ChangeTag::Insert,
                        text,
                    });
                }
            }
        }
        flush_sbs_pairs(&mut self.sbs_pairs, &mut old_lines, &mut new_lines);
    }

    /// Get the style for a given change tag.
    fn style_for_tag(&self, tag: ChangeTag) -> &Style {
        match tag {
            ChangeTag::Equal => &self.unchanged_style,
            ChangeTag::Insert => &self.added_style,
            ChangeTag::Delete => &self.removed_style,
        }
    }

    /// Get the prefix character for a given change tag.
    fn prefix_for_tag(tag: ChangeTag) -> &'static str {
        match tag {
            ChangeTag::Equal => " ",
            ChangeTag::Insert => "+",
            ChangeTag::Delete => "-",
        }
    }

    /// Render a single line of text into the buffer at the given position.
    fn render_line(
        &self,
        text: &str,
        style: &Style,
        x: u16,
        y: u16,
        max_width: usize,
        buf: &mut ScreenBuffer,
    ) {
        let truncated = truncate_to_display_width(text, max_width);
        let mut col: u16 = 0;
        for ch in truncated.chars() {
            let char_w = unicode_width::UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]) as &str);
            if col as usize + char_w > max_width {
                break;
            }
            buf.set(x + col, y, Cell::new(ch.to_string(), style.clone()));
            col += char_w as u16;
        }
    }

    /// Render in unified mode.
    fn render_unified(&self, inner: Rect, buf: &mut ScreenBuffer) {
        let height = inner.size.height as usize;
        let width = inner.size.width as usize;
        let count = self.unified_lines.len();
        let max_offset = count.saturating_sub(height.max(1));
        let scroll = self.scroll_offset.min(max_offset);
        let end = (scroll + height).min(count);

        for (row, line_idx) in (scroll..end).enumerate() {
            let y = inner.position.y + row as u16;
            if let Some(line) = self.unified_lines.get(line_idx) {
                let style = self.style_for_tag(line.tag);
                let prefix = Self::prefix_for_tag(line.tag);

                // Fill the full row with the style
                for col in 0..inner.size.width {
                    buf.set(inner.position.x + col, y, Cell::new(" ", style.clone()));
                }

                // Render prefix
                if width > 0 {
                    buf.set(inner.position.x, y, Cell::new(prefix, style.clone()));
                }

                // Render text content (after prefix)
                if width > 1 {
                    self.render_line(
                        &line.text,
                        style,
                        inner.position.x + 1,
                        y,
                        width.saturating_sub(1),
                        buf,
                    );
                }
            }
        }
    }

    /// Render in side-by-side mode.
    fn render_side_by_side(&self, inner: Rect, buf: &mut ScreenBuffer) {
        let height = inner.size.height as usize;
        let total_width = inner.size.width as usize;
        let count = self.sbs_pairs.len();
        let max_offset = count.saturating_sub(height.max(1));
        let scroll = self.scroll_offset.min(max_offset);
        let end = (scroll + height).min(count);

        // Split width: left half | separator | right half
        if total_width < 3 {
            return;
        }
        let separator_col = total_width / 2;
        let left_width = separator_col;
        let right_width = total_width.saturating_sub(separator_col + 1);

        // Draw separator
        for row in 0..inner.size.height {
            buf.set(
                inner.position.x + separator_col as u16,
                inner.position.y + row,
                Cell::new("\u{2502}", self.unchanged_style.clone()), // │
            );
        }

        for (row, pair_idx) in (scroll..end).enumerate() {
            let y = inner.position.y + row as u16;
            if let Some(pair) = self.sbs_pairs.get(pair_idx) {
                // Left side
                if let Some(ref left) = pair.left {
                    let style = self.style_for_tag(left.tag);
                    // Fill left side with style
                    for col in 0..left_width {
                        buf.set(
                            inner.position.x + col as u16,
                            y,
                            Cell::new(" ", style.clone()),
                        );
                    }
                    self.render_line(&left.text, style, inner.position.x, y, left_width, buf);
                }

                // Right side
                if let Some(ref right) = pair.right {
                    let style = self.style_for_tag(right.tag);
                    let right_x = inner.position.x + separator_col as u16 + 1;
                    // Fill right side with style
                    for col in 0..right_width {
                        buf.set(right_x + col as u16, y, Cell::new(" ", style.clone()));
                    }
                    self.render_line(&right.text, style, right_x, y, right_width, buf);
                }
            }
        }
    }
}

impl Widget for DiffView {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        super::border::render_border(area, self.border, self.unchanged_style.clone(), buf);

        let inner = super::border::inner_area(area, self.border);
        if inner.size.width == 0 || inner.size.height == 0 {
            return;
        }

        match self.mode {
            DiffMode::Unified => self.render_unified(inner, buf),
            DiffMode::SideBySide => self.render_side_by_side(inner, buf),
        }
    }
}

impl InteractiveWidget for DiffView {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        let count = self.line_count();

        match code {
            KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                EventResult::Consumed
            }
            KeyCode::Down => {
                if count > 0 && self.scroll_offset < count.saturating_sub(1) {
                    self.scroll_offset += 1;
                }
                EventResult::Consumed
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
                EventResult::Consumed
            }
            KeyCode::PageDown => {
                if count > 0 {
                    self.scroll_offset = (self.scroll_offset + 20).min(count.saturating_sub(1));
                }
                EventResult::Consumed
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
                EventResult::Consumed
            }
            KeyCode::End => {
                if count > 0 {
                    self.scroll_offset = count.saturating_sub(1);
                }
                EventResult::Consumed
            }
            KeyCode::Char('m') => {
                self.mode = match self.mode {
                    DiffMode::Unified => DiffMode::SideBySide,
                    DiffMode::SideBySide => DiffMode::Unified,
                };
                self.scroll_offset = 0;
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

/// Flush accumulated old/new lines into side-by-side pairs.
fn flush_sbs_pairs(
    pairs: &mut Vec<SideBySidePair>,
    old_lines: &mut Vec<DiffLine>,
    new_lines: &mut Vec<DiffLine>,
) {
    let max_len = old_lines.len().max(new_lines.len());
    for i in 0..max_len {
        pairs.push(SideBySidePair {
            left: old_lines.get(i).cloned(),
            right: new_lines.get(i).cloned(),
        });
    }
    old_lines.clear();
    new_lines.clear();
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    #[test]
    fn create_diff_view() {
        let dv = DiffView::new("hello\nworld\n", "hello\nrust\n");
        assert_eq!(dv.mode(), DiffMode::Unified);
        assert!(dv.line_count() > 0);
    }

    #[test]
    fn unified_prefixes() {
        let dv = DiffView::new("aaa\nbbb\n", "aaa\nccc\n");

        // Should have: " aaa", "-bbb", "+ccc"
        assert_eq!(dv.unified_lines.len(), 3);
        assert_eq!(dv.unified_lines[0].tag, ChangeTag::Equal);
        assert_eq!(dv.unified_lines[0].text, "aaa");
        assert_eq!(dv.unified_lines[1].tag, ChangeTag::Delete);
        assert_eq!(dv.unified_lines[1].text, "bbb");
        assert_eq!(dv.unified_lines[2].tag, ChangeTag::Insert);
        assert_eq!(dv.unified_lines[2].text, "ccc");
    }

    #[test]
    fn side_by_side_pairs() {
        let dv = DiffView::new("aaa\nbbb\n", "aaa\nccc\n").with_mode(DiffMode::SideBySide);

        assert_eq!(dv.mode(), DiffMode::SideBySide);
        // Pair 1: aaa | aaa (equal)
        // Pair 2: bbb | ccc (delete | insert)
        assert_eq!(dv.sbs_pairs.len(), 2);

        assert!(dv.sbs_pairs[0].left.is_some());
        assert!(dv.sbs_pairs[0].right.is_some());
        assert_eq!(
            dv.sbs_pairs[0].left.as_ref().map(|l| l.tag),
            Some(ChangeTag::Equal)
        );

        assert!(dv.sbs_pairs[1].left.is_some());
        assert!(dv.sbs_pairs[1].right.is_some());
        assert_eq!(
            dv.sbs_pairs[1].left.as_ref().map(|l| l.tag),
            Some(ChangeTag::Delete)
        );
        assert_eq!(
            dv.sbs_pairs[1].right.as_ref().map(|l| l.tag),
            Some(ChangeTag::Insert)
        );
    }

    #[test]
    fn scroll_up_down() {
        let mut dv = DiffView::new("a\nb\nc\nd\ne\nf\n", "a\nb\nc\nd\ne\nf\n");

        let down = Event::Key(KeyEvent {
            code: KeyCode::Down,
            modifiers: crate::event::Modifiers::NONE,
        });
        let up = Event::Key(KeyEvent {
            code: KeyCode::Up,
            modifiers: crate::event::Modifiers::NONE,
        });

        assert_eq!(dv.scroll_offset(), 0);
        dv.handle_event(&down);
        assert_eq!(dv.scroll_offset(), 1);
        dv.handle_event(&up);
        assert_eq!(dv.scroll_offset(), 0);
        // Up at 0 stays 0
        dv.handle_event(&up);
        assert_eq!(dv.scroll_offset(), 0);
    }

    #[test]
    fn page_up_down() {
        let mut dv = DiffView::new(
            "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11\nline12\nline13\nline14\nline15\nline16\nline17\nline18\nline19\nline20\nline21\nline22\nline23\nline24\nline25\n",
            "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11\nline12\nline13\nline14\nline15\nline16\nline17\nline18\nline19\nline20\nline21\nline22\nline23\nline24\nline25\n",
        );

        let pgdn = Event::Key(KeyEvent {
            code: KeyCode::PageDown,
            modifiers: crate::event::Modifiers::NONE,
        });
        let pgup = Event::Key(KeyEvent {
            code: KeyCode::PageUp,
            modifiers: crate::event::Modifiers::NONE,
        });

        dv.handle_event(&pgdn);
        assert_eq!(dv.scroll_offset(), 20);
        dv.handle_event(&pgup);
        assert_eq!(dv.scroll_offset(), 0);
    }

    #[test]
    fn home_end() {
        let mut dv = DiffView::new("a\nb\nc\nd\ne\n", "a\nb\nc\nd\ne\n");

        let end_key = Event::Key(KeyEvent {
            code: KeyCode::End,
            modifiers: crate::event::Modifiers::NONE,
        });
        let home_key = Event::Key(KeyEvent {
            code: KeyCode::Home,
            modifiers: crate::event::Modifiers::NONE,
        });

        dv.handle_event(&end_key);
        assert_eq!(dv.scroll_offset(), dv.line_count().saturating_sub(1));
        dv.handle_event(&home_key);
        assert_eq!(dv.scroll_offset(), 0);
    }

    #[test]
    fn toggle_mode_with_m() {
        let mut dv = DiffView::new("a\n", "b\n");
        assert_eq!(dv.mode(), DiffMode::Unified);

        let m = Event::Key(KeyEvent {
            code: KeyCode::Char('m'),
            modifiers: crate::event::Modifiers::NONE,
        });

        dv.handle_event(&m);
        assert_eq!(dv.mode(), DiffMode::SideBySide);
        dv.handle_event(&m);
        assert_eq!(dv.mode(), DiffMode::Unified);
    }

    #[test]
    fn empty_diff_identical_texts() {
        let dv = DiffView::new("same\n", "same\n");
        assert_eq!(dv.unified_lines.len(), 1);
        assert_eq!(dv.unified_lines[0].tag, ChangeTag::Equal);
    }

    #[test]
    fn all_added_old_empty() {
        let dv = DiffView::new("", "new1\nnew2\n");
        for line in &dv.unified_lines {
            assert_eq!(line.tag, ChangeTag::Insert);
        }
    }

    #[test]
    fn all_removed_new_empty() {
        let dv = DiffView::new("old1\nold2\n", "");
        for line in &dv.unified_lines {
            assert_eq!(line.tag, ChangeTag::Delete);
        }
    }

    #[test]
    fn mixed_changes() {
        let dv = DiffView::new("a\nb\nc\n", "a\nB\nc\nd\n");
        // a = equal, b = delete, B = insert, c = equal, d = insert
        let tags: Vec<ChangeTag> = dv.unified_lines.iter().map(|l| l.tag).collect();
        assert!(tags.contains(&ChangeTag::Equal));
        assert!(tags.contains(&ChangeTag::Delete));
        assert!(tags.contains(&ChangeTag::Insert));
    }

    #[test]
    fn render_unified_mode() {
        let dv = DiffView::new("old\n", "new\n");
        let mut buf = ScreenBuffer::new(Size::new(30, 5));
        dv.render(Rect::new(0, 0, 30, 5), &mut buf);

        // First line should have "-" prefix (delete)
        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("-"));
        // Second line should have "+" prefix (insert)
        assert_eq!(buf.get(0, 1).map(|c| c.grapheme.as_str()), Some("+"));
    }

    #[test]
    fn render_side_by_side_mode() {
        let dv = DiffView::new("old\n", "new\n").with_mode(DiffMode::SideBySide);
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        dv.render(Rect::new(0, 0, 20, 5), &mut buf);

        // Separator at column 10 (width/2)
        assert_eq!(
            buf.get(10, 0).map(|c| c.grapheme.as_str()),
            Some("\u{2502}")
        );
    }

    #[test]
    fn set_texts_recomputes() {
        let mut dv = DiffView::new("a\n", "b\n");
        let initial_count = dv.line_count();

        dv.set_texts("x\ny\nz\n", "x\nw\nz\n");
        // Should have recomputed
        assert!(dv.line_count() > 0);
        // Scroll should be reset
        assert_eq!(dv.scroll_offset(), 0);
        // Count may differ
        let _ = initial_count;
    }

    #[test]
    fn border_rendering() {
        let dv = DiffView::new("a\n", "b\n").with_border(BorderStyle::Single);
        let mut buf = ScreenBuffer::new(Size::new(30, 10));
        dv.render(Rect::new(0, 0, 30, 10), &mut buf);

        assert_eq!(buf.get(0, 0).map(|c| c.grapheme.as_str()), Some("\u{250c}"));
    }

    #[test]
    fn utf8_safe_diff() {
        let dv = DiffView::new("你好\n", "世界\n");
        assert_eq!(dv.line_count(), 2); // one delete, one insert

        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        dv.render(Rect::new(0, 0, 20, 5), &mut buf);

        // Should render without panic
        assert!(buf.get(0, 0).is_some());
    }
}
