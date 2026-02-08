//! Form control widgets: Switch, RadioButton, and Checkbox.
//!
//! Simple interactive toggle/selection widgets for building forms
//! and settings screens.

use crate::buffer::ScreenBuffer;
use crate::cell::Cell;
use crate::event::{Event, KeyCode, KeyEvent};
use crate::geometry::Rect;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use unicode_width::UnicodeWidthStr;

use super::{EventResult, InteractiveWidget, Widget};

// ---------------------------------------------------------------------------
// Switch
// ---------------------------------------------------------------------------

/// A toggle switch widget (on/off).
///
/// Renders as `[ON ] label` or `[OFF] label`.
pub struct Switch {
    /// Current toggle state.
    state: bool,
    /// Label text.
    label: String,
    /// Style when on.
    on_style: Style,
    /// Style when off.
    off_style: Style,
    /// Text shown when on (e.g., "[ON ]").
    on_indicator: String,
    /// Text shown when off (e.g., "[OFF]").
    off_indicator: String,
}

impl Switch {
    /// Create a new switch with the given label (off by default).
    pub fn new(label: &str) -> Self {
        Self {
            state: false,
            label: label.to_string(),
            on_style: Style::default().bold(true),
            off_style: Style::default(),
            on_indicator: "[ON ]".to_string(),
            off_indicator: "[OFF]".to_string(),
        }
    }

    /// Set the initial state.
    #[must_use]
    pub fn with_state(mut self, state: bool) -> Self {
        self.state = state;
        self
    }

    /// Set the style when on.
    #[must_use]
    pub fn with_on_style(mut self, style: Style) -> Self {
        self.on_style = style;
        self
    }

    /// Set the style when off.
    #[must_use]
    pub fn with_off_style(mut self, style: Style) -> Self {
        self.off_style = style;
        self
    }

    /// Set custom indicator text for on/off.
    #[must_use]
    pub fn with_indicators(mut self, on: &str, off: &str) -> Self {
        self.on_indicator = on.to_string();
        self.off_indicator = off.to_string();
        self
    }

    /// Toggle the switch state.
    pub fn toggle(&mut self) {
        self.state = !self.state;
    }

    /// Set the switch state.
    pub fn set_state(&mut self, state: bool) {
        self.state = state;
    }

    /// Get the current state.
    pub fn state(&self) -> bool {
        self.state
    }
}

impl Widget for Switch {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        let (indicator, style) = if self.state {
            (&self.on_indicator, &self.on_style)
        } else {
            (&self.off_indicator, &self.off_style)
        };

        let text = format!("{indicator} {}", self.label);
        render_single_line(&text, style, area, buf);
    }
}

impl InteractiveWidget for Switch {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        match code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

// ---------------------------------------------------------------------------
// RadioButton
// ---------------------------------------------------------------------------

/// A radio button widget (single selection in a group).
///
/// Renders as `(●) label` (selected) or `( ) label` (unselected).
pub struct RadioButton {
    /// Label text.
    label: String,
    /// Whether selected.
    selected: bool,
    /// Style when selected.
    selected_style: Style,
    /// Style when unselected.
    unselected_style: Style,
}

impl RadioButton {
    /// Create a new radio button (unselected by default).
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            selected: false,
            selected_style: Style::default().bold(true),
            unselected_style: Style::default(),
        }
    }

    /// Set the initial selected state.
    #[must_use]
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set the style when selected.
    #[must_use]
    pub fn with_selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }

    /// Set the style when unselected.
    #[must_use]
    pub fn with_unselected_style(mut self, style: Style) -> Self {
        self.unselected_style = style;
        self
    }

    /// Select this radio button.
    pub fn select(&mut self) {
        self.selected = true;
    }

    /// Deselect this radio button.
    pub fn deselect(&mut self) {
        self.selected = false;
    }

    /// Check if selected.
    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

impl Widget for RadioButton {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        let (indicator, style) = if self.selected {
            ("(\u{25cf})", &self.selected_style) // (●)
        } else {
            ("( )", &self.unselected_style)
        };

        let text = format!("{indicator} {}", self.label);
        render_single_line(&text, style, area, buf);
    }
}

impl InteractiveWidget for RadioButton {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        match code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.select();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

/// A checkbox widget (boolean toggle with label).
///
/// Renders as `[✓] label` (checked) or `[ ] label` (unchecked).
pub struct Checkbox {
    /// Label text.
    label: String,
    /// Whether checked.
    checked: bool,
    /// Style when checked.
    checked_style: Style,
    /// Style when unchecked.
    unchecked_style: Style,
}

impl Checkbox {
    /// Create a new checkbox (unchecked by default).
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            checked: false,
            checked_style: Style::default().bold(true),
            unchecked_style: Style::default(),
        }
    }

    /// Set the initial checked state.
    #[must_use]
    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the style when checked.
    #[must_use]
    pub fn with_checked_style(mut self, style: Style) -> Self {
        self.checked_style = style;
        self
    }

    /// Set the style when unchecked.
    #[must_use]
    pub fn with_unchecked_style(mut self, style: Style) -> Self {
        self.unchecked_style = style;
        self
    }

    /// Toggle the checked state.
    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    /// Set the checked state.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    /// Check if checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }
}

impl Widget for Checkbox {
    fn render(&self, area: Rect, buf: &mut ScreenBuffer) {
        if area.size.width == 0 || area.size.height == 0 {
            return;
        }

        let (indicator, style) = if self.checked {
            ("[\u{2713}]", &self.checked_style) // [✓]
        } else {
            ("[ ]", &self.unchecked_style)
        };

        let text = format!("{indicator} {}", self.label);
        render_single_line(&text, style, area, buf);
    }
}

impl InteractiveWidget for Checkbox {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return EventResult::Ignored;
        };

        match code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

// ---------------------------------------------------------------------------
// Shared rendering helper
// ---------------------------------------------------------------------------

/// Render a single line of text into the buffer at the given area.
fn render_single_line(text: &str, style: &Style, area: Rect, buf: &mut ScreenBuffer) {
    let w = area.size.width as usize;
    let x0 = area.position.x;
    let y = area.position.y;

    let truncated = truncate_to_display_width(text, w);
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::geometry::Size;

    fn row_text(buf: &ScreenBuffer, y: u16, w: u16) -> String {
        (0..w)
            .map(|x| buf.get(x, y).map(|c| c.grapheme.as_str()).unwrap_or(" "))
            .collect()
    }

    // --- Switch tests ---

    #[test]
    fn switch_default_off() {
        let s = Switch::new("Dark Mode");
        assert!(!s.state());
    }

    #[test]
    fn switch_toggle() {
        let mut s = Switch::new("Test");
        s.toggle();
        assert!(s.state());
        s.toggle();
        assert!(!s.state());
    }

    #[test]
    fn switch_set_state() {
        let mut s = Switch::new("Test");
        s.set_state(true);
        assert!(s.state());
        s.set_state(false);
        assert!(!s.state());
    }

    #[test]
    fn switch_render_on() {
        let s = Switch::new("Opt").with_state(true);
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        s.render(Rect::new(0, 0, 20, 1), &mut buf);
        let text = row_text(&buf, 0, 20);
        assert!(text.contains("[ON ]"));
        assert!(text.contains("Opt"));
    }

    #[test]
    fn switch_render_off() {
        let s = Switch::new("Opt").with_state(false);
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        s.render(Rect::new(0, 0, 20, 1), &mut buf);
        let text = row_text(&buf, 0, 20);
        assert!(text.contains("[OFF]"));
    }

    #[test]
    fn switch_custom_indicators() {
        let s = Switch::new("X")
            .with_state(true)
            .with_indicators("YES", "NO");
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        s.render(Rect::new(0, 0, 20, 1), &mut buf);
        let text = row_text(&buf, 0, 20);
        assert!(text.contains("YES"));
    }

    #[test]
    fn switch_space_toggles() {
        let mut s = Switch::new("T");
        let result = s.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Char(' '))));
        assert_eq!(result, EventResult::Consumed);
        assert!(s.state());
    }

    // --- RadioButton tests ---

    #[test]
    fn radio_default_unselected() {
        let r = RadioButton::new("Option A");
        assert!(!r.is_selected());
    }

    #[test]
    fn radio_select_deselect() {
        let mut r = RadioButton::new("A");
        r.select();
        assert!(r.is_selected());
        r.deselect();
        assert!(!r.is_selected());
    }

    #[test]
    fn radio_render_selected() {
        let r = RadioButton::new("Choice").with_selected(true);
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        r.render(Rect::new(0, 0, 20, 1), &mut buf);
        let text = row_text(&buf, 0, 20);
        assert!(text.contains("(\u{25cf})"));
        assert!(text.contains("Choice"));
    }

    #[test]
    fn radio_render_unselected() {
        let r = RadioButton::new("Choice");
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        r.render(Rect::new(0, 0, 20, 1), &mut buf);
        let text = row_text(&buf, 0, 20);
        assert!(text.contains("( )"));
    }

    #[test]
    fn radio_enter_selects() {
        let mut r = RadioButton::new("A");
        let result = r.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Enter)));
        assert_eq!(result, EventResult::Consumed);
        assert!(r.is_selected());
    }

    // --- Checkbox tests ---

    #[test]
    fn checkbox_default_unchecked() {
        let c = Checkbox::new("Enable");
        assert!(!c.is_checked());
    }

    #[test]
    fn checkbox_toggle() {
        let mut c = Checkbox::new("Enable");
        c.toggle();
        assert!(c.is_checked());
        c.toggle();
        assert!(!c.is_checked());
    }

    #[test]
    fn checkbox_set_checked() {
        let mut c = Checkbox::new("Enable");
        c.set_checked(true);
        assert!(c.is_checked());
    }

    #[test]
    fn checkbox_render_checked() {
        let c = Checkbox::new("Agree").with_checked(true);
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        c.render(Rect::new(0, 0, 20, 1), &mut buf);
        let text = row_text(&buf, 0, 20);
        assert!(text.contains("[\u{2713}]"));
        assert!(text.contains("Agree"));
    }

    #[test]
    fn checkbox_render_unchecked() {
        let c = Checkbox::new("Agree");
        let mut buf = ScreenBuffer::new(Size::new(20, 1));
        c.render(Rect::new(0, 0, 20, 1), &mut buf);
        let text = row_text(&buf, 0, 20);
        assert!(text.contains("[ ]"));
    }

    #[test]
    fn checkbox_space_toggles() {
        let mut c = Checkbox::new("X");
        let result = c.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Char(' '))));
        assert_eq!(result, EventResult::Consumed);
        assert!(c.is_checked());
    }

    // --- Cross-widget tests ---

    #[test]
    fn all_single_line_rendering() {
        let s = Switch::new("S");
        let r = RadioButton::new("R");
        let c = Checkbox::new("C");

        let mut buf = ScreenBuffer::new(Size::new(20, 3));
        s.render(Rect::new(0, 0, 20, 1), &mut buf);
        r.render(Rect::new(0, 1, 20, 1), &mut buf);
        c.render(Rect::new(0, 2, 20, 1), &mut buf);

        // All three should have rendered
        assert!(row_text(&buf, 0, 20).contains("[OFF]"));
        assert!(row_text(&buf, 1, 20).contains("( )"));
        assert!(row_text(&buf, 2, 20).contains("[ ]"));
    }

    #[test]
    fn unhandled_event_ignored() {
        let mut s = Switch::new("T");
        let result = s.handle_event(&Event::Key(KeyEvent::plain(KeyCode::Char('z'))));
        assert_eq!(result, EventResult::Ignored);
    }
}
