//! Event types for terminal input handling.

use std::fmt;

/// A terminal event.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Event {
    /// A key was pressed.
    Key(KeyEvent),
    /// A mouse event occurred.
    Mouse(MouseEvent),
    /// The terminal was resized.
    Resize(u16, u16),
    /// Text was pasted (bracketed paste mode).
    Paste(String),
}

/// A keyboard event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key code.
    pub code: KeyCode,
    /// Active modifiers.
    pub modifiers: Modifiers,
}

impl KeyEvent {
    /// Create a new key event.
    pub fn new(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }

    /// Create a plain key event with no modifiers.
    pub fn plain(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: Modifiers::NONE,
        }
    }

    /// Check if Ctrl is held.
    pub fn ctrl(&self) -> bool {
        self.modifiers.contains(Modifiers::CTRL)
    }

    /// Check if Alt is held.
    pub fn alt(&self) -> bool {
        self.modifiers.contains(Modifiers::ALT)
    }

    /// Check if Shift is held.
    pub fn shift(&self) -> bool {
        self.modifiers.contains(Modifiers::SHIFT)
    }
}

/// A key code.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum KeyCode {
    /// A character key.
    Char(char),
    /// Enter / Return.
    Enter,
    /// Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Escape.
    Escape,
    /// Arrow up.
    Up,
    /// Arrow down.
    Down,
    /// Arrow left.
    Left,
    /// Arrow right.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// Insert.
    Insert,
    /// Function key (F1-F12).
    F(u8),
}

/// Keyboard modifier flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Modifiers(u8);

impl Modifiers {
    /// No modifiers.
    pub const NONE: Self = Self(0);
    /// Shift modifier.
    pub const SHIFT: Self = Self(1);
    /// Ctrl modifier.
    pub const CTRL: Self = Self(2);
    /// Alt/Option modifier.
    pub const ALT: Self = Self(4);
    /// Super/Command modifier.
    pub const SUPER: Self = Self(8);

    /// Check if this modifier set contains the given modifier.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0 && other.0 != 0
    }

    /// Combine two modifier sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// The kind of mouse event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MouseEventKind {
    /// A button was pressed.
    Press,
    /// A button was released.
    Release,
    /// The mouse was moved (while a button is held).
    Drag,
    /// The mouse was moved (no button held).
    Move,
    /// Scroll up.
    ScrollUp,
    /// Scroll down.
    ScrollDown,
}

/// A mouse event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    /// The kind of mouse event.
    pub kind: MouseEventKind,
    /// Column position (0-based).
    pub x: u16,
    /// Row position (0-based).
    pub y: u16,
    /// Active modifiers.
    pub modifiers: Modifiers,
}

// Crossterm conversions

impl From<crossterm::event::Event> for Event {
    fn from(ct: crossterm::event::Event) -> Self {
        match ct {
            crossterm::event::Event::Key(key) => Event::Key(key.into()),
            crossterm::event::Event::Mouse(mouse) => Event::Mouse(mouse.into()),
            crossterm::event::Event::Resize(w, h) => Event::Resize(w, h),
            crossterm::event::Event::Paste(text) => Event::Paste(text),
            _ => Event::Key(KeyEvent::plain(KeyCode::Escape)), // fallback for FocusGained/Lost
        }
    }
}

impl From<crossterm::event::KeyEvent> for KeyEvent {
    fn from(ct: crossterm::event::KeyEvent) -> Self {
        Self {
            code: ct.code.into(),
            modifiers: ct.modifiers.into(),
        }
    }
}

impl From<crossterm::event::KeyCode> for KeyCode {
    fn from(ct: crossterm::event::KeyCode) -> Self {
        match ct {
            crossterm::event::KeyCode::Char(c) => KeyCode::Char(c),
            crossterm::event::KeyCode::Enter => KeyCode::Enter,
            crossterm::event::KeyCode::Tab => KeyCode::Tab,
            crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
            crossterm::event::KeyCode::Delete => KeyCode::Delete,
            crossterm::event::KeyCode::Esc => KeyCode::Escape,
            crossterm::event::KeyCode::Up => KeyCode::Up,
            crossterm::event::KeyCode::Down => KeyCode::Down,
            crossterm::event::KeyCode::Left => KeyCode::Left,
            crossterm::event::KeyCode::Right => KeyCode::Right,
            crossterm::event::KeyCode::Home => KeyCode::Home,
            crossterm::event::KeyCode::End => KeyCode::End,
            crossterm::event::KeyCode::PageUp => KeyCode::PageUp,
            crossterm::event::KeyCode::PageDown => KeyCode::PageDown,
            crossterm::event::KeyCode::Insert => KeyCode::Insert,
            crossterm::event::KeyCode::F(n) => KeyCode::F(n),
            _ => KeyCode::Escape, // fallback
        }
    }
}

impl From<crossterm::event::KeyModifiers> for Modifiers {
    fn from(ct: crossterm::event::KeyModifiers) -> Self {
        let mut m = Modifiers::NONE;
        if ct.contains(crossterm::event::KeyModifiers::SHIFT) {
            m = m | Modifiers::SHIFT;
        }
        if ct.contains(crossterm::event::KeyModifiers::CONTROL) {
            m = m | Modifiers::CTRL;
        }
        if ct.contains(crossterm::event::KeyModifiers::ALT) {
            m = m | Modifiers::ALT;
        }
        if ct.contains(crossterm::event::KeyModifiers::SUPER) {
            m = m | Modifiers::SUPER;
        }
        m
    }
}

impl From<crossterm::event::MouseEvent> for MouseEvent {
    fn from(ct: crossterm::event::MouseEvent) -> Self {
        Self {
            kind: ct.kind.into(),
            x: ct.column,
            y: ct.row,
            modifiers: ct.modifiers.into(),
        }
    }
}

impl From<crossterm::event::MouseEventKind> for MouseEventKind {
    fn from(ct: crossterm::event::MouseEventKind) -> Self {
        match ct {
            crossterm::event::MouseEventKind::Down(_) => MouseEventKind::Press,
            crossterm::event::MouseEventKind::Up(_) => MouseEventKind::Release,
            crossterm::event::MouseEventKind::Drag(_) => MouseEventKind::Drag,
            crossterm::event::MouseEventKind::Moved => MouseEventKind::Move,
            crossterm::event::MouseEventKind::ScrollUp => MouseEventKind::ScrollUp,
            crossterm::event::MouseEventKind::ScrollDown => MouseEventKind::ScrollDown,
            _ => MouseEventKind::Move, // fallback for ScrollLeft/ScrollRight
        }
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyCode::Char(c) => write!(f, "{c}"),
            KeyCode::Enter => write!(f, "Enter"),
            KeyCode::Tab => write!(f, "Tab"),
            KeyCode::Backspace => write!(f, "Backspace"),
            KeyCode::Delete => write!(f, "Delete"),
            KeyCode::Escape => write!(f, "Escape"),
            KeyCode::Up => write!(f, "Up"),
            KeyCode::Down => write!(f, "Down"),
            KeyCode::Left => write!(f, "Left"),
            KeyCode::Right => write!(f, "Right"),
            KeyCode::Home => write!(f, "Home"),
            KeyCode::End => write!(f, "End"),
            KeyCode::PageUp => write!(f, "PageUp"),
            KeyCode::PageDown => write!(f, "PageDown"),
            KeyCode::Insert => write!(f, "Insert"),
            KeyCode::F(n) => write!(f, "F{n}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_event_plain() {
        let k = KeyEvent::plain(KeyCode::Char('a'));
        assert!(!k.ctrl());
        assert!(!k.alt());
        assert!(!k.shift());
    }

    #[test]
    fn key_event_with_modifiers() {
        let k = KeyEvent::new(KeyCode::Char('c'), Modifiers::CTRL);
        assert!(k.ctrl());
        assert!(!k.alt());
    }

    #[test]
    fn modifier_union() {
        let m = Modifiers::CTRL | Modifiers::SHIFT;
        assert!(m.contains(Modifiers::CTRL));
        assert!(m.contains(Modifiers::SHIFT));
        assert!(!m.contains(Modifiers::ALT));
    }

    #[test]
    fn resize_event() {
        let e = Event::Resize(80, 24);
        assert!(matches!(e, Event::Resize(80, 24)));
    }

    #[test]
    fn paste_event() {
        let e = Event::Paste("hello".into());
        assert!(matches!(e, Event::Paste(ref s) if s == "hello"));
    }

    #[test]
    fn mouse_event() {
        let m = MouseEvent {
            kind: MouseEventKind::Press,
            x: 10,
            y: 5,
            modifiers: Modifiers::NONE,
        };
        assert_eq!(m.kind, MouseEventKind::Press);
        assert_eq!(m.x, 10);
        assert_eq!(m.y, 5);
    }

    #[test]
    fn keycode_display() {
        assert_eq!(format!("{}", KeyCode::Char('a')), "a");
        assert_eq!(format!("{}", KeyCode::Enter), "Enter");
        assert_eq!(format!("{}", KeyCode::F(1)), "F1");
    }

    #[test]
    fn crossterm_key_conversion() {
        let ct = crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('x'),
            crossterm::event::KeyModifiers::CONTROL,
        );
        let k: KeyEvent = ct.into();
        assert_eq!(k.code, KeyCode::Char('x'));
        assert!(k.ctrl());
    }

    #[test]
    fn crossterm_resize_conversion() {
        let ct = crossterm::event::Event::Resize(120, 40);
        let e: Event = ct.into();
        assert!(matches!(e, Event::Resize(120, 40)));
    }
}
