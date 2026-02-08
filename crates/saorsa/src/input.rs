//! Input event handling for the chat application.

use saorsa_core::event::{Event, KeyCode, Modifiers, MouseEventKind};

use crate::app::AppState;

/// Number of lines to scroll per mouse wheel tick.
const MOUSE_SCROLL_LINES: usize = 3;

/// Number of lines to scroll per PageUp/PageDown press.
const PAGE_SCROLL_LINES: usize = 10;

/// Result of handling an input event.
#[derive(Debug, PartialEq, Eq)]
pub enum InputAction {
    /// No action needed.
    None,
    /// The user submitted a message.
    Submit(String),
    /// The user wants to quit.
    Quit,
    /// The UI needs to be redrawn.
    Redraw,
    /// Cycle to the next model (Ctrl+P).
    CycleModel,
    /// Cycle to the previous model (Shift+Ctrl+P).
    CycleModelBackward,
    /// Scroll message history up by the given number of lines.
    ScrollUp(usize),
    /// Scroll message history down by the given number of lines.
    ScrollDown(usize),
    /// Open the model selector overlay (Ctrl+L).
    OpenModelSelector,
    /// Tab-complete the current input.
    TabComplete,
}

/// Handle an input event and return the resulting action.
pub fn handle_event(state: &mut AppState, event: &Event) -> InputAction {
    match event {
        Event::Key(key) => handle_key(state, key.code.clone(), key.modifiers),
        Event::Mouse(mouse) => handle_mouse(mouse.kind),
        Event::Resize(_, _) => InputAction::Redraw,
        _ => InputAction::None,
    }
}

/// Handle a mouse event.
fn handle_mouse(kind: MouseEventKind) -> InputAction {
    match kind {
        MouseEventKind::ScrollUp => InputAction::ScrollUp(MOUSE_SCROLL_LINES),
        MouseEventKind::ScrollDown => InputAction::ScrollDown(MOUSE_SCROLL_LINES),
        _ => InputAction::None,
    }
}

/// Handle a key event.
fn handle_key(state: &mut AppState, code: KeyCode, modifiers: Modifiers) -> InputAction {
    // Ctrl-C always quits.
    if code == KeyCode::Char('c') && modifiers.contains(Modifiers::CTRL) {
        state.should_quit = true;
        return InputAction::Quit;
    }

    // Ctrl-D quits on empty input.
    if code == KeyCode::Char('d') && modifiers.contains(Modifiers::CTRL) && state.input.is_empty() {
        state.should_quit = true;
        return InputAction::Quit;
    }

    // Scrolling works even when the AI is thinking.
    if code == KeyCode::PageUp {
        return InputAction::ScrollUp(PAGE_SCROLL_LINES);
    }
    if code == KeyCode::PageDown {
        return InputAction::ScrollDown(PAGE_SCROLL_LINES);
    }

    // Only process editing keys when idle.
    if !state.is_idle() {
        return InputAction::None;
    }

    // Ctrl+P: cycle model forward.
    if code == KeyCode::Char('p') && modifiers.contains(Modifiers::CTRL) {
        if modifiers.contains(Modifiers::SHIFT) {
            return InputAction::CycleModelBackward;
        }
        return InputAction::CycleModel;
    }

    // Ctrl+L: open model selector overlay.
    if code == KeyCode::Char('l') && modifiers.contains(Modifiers::CTRL) {
        return InputAction::OpenModelSelector;
    }

    match code {
        KeyCode::Tab if modifiers == Modifiers::NONE => InputAction::TabComplete,
        KeyCode::Enter => {
            let text = state.take_input();
            if text.is_empty() {
                InputAction::None
            } else {
                InputAction::Submit(text)
            }
        }
        KeyCode::Char(c)
            if !modifiers.contains(Modifiers::CTRL) && !modifiers.contains(Modifiers::ALT) =>
        {
            state.insert_char(c);
            InputAction::Redraw
        }
        KeyCode::Backspace => {
            state.delete_char_before();
            InputAction::Redraw
        }
        KeyCode::Left => {
            state.cursor_left();
            InputAction::Redraw
        }
        KeyCode::Right => {
            state.cursor_right();
            InputAction::Redraw
        }
        KeyCode::Home => {
            state.cursor = 0;
            InputAction::Redraw
        }
        KeyCode::End => {
            state.cursor = state.input.len();
            InputAction::Redraw
        }
        KeyCode::Escape => {
            state.input.clear();
            state.cursor = 0;
            InputAction::Redraw
        }
        _ => InputAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_core::event::KeyEvent;

    fn key_event(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: Modifiers::NONE,
        })
    }

    fn ctrl_key(c: char) -> Event {
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: Modifiers::CTRL,
        })
    }

    #[test]
    fn typing_characters() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &key_event(KeyCode::Char('h')));
        assert_eq!(action, InputAction::Redraw);
        assert_eq!(state.input, "h");
    }

    #[test]
    fn submit_on_enter() {
        let mut state = AppState::new("test");
        state.input = "hello".into();
        state.cursor = 5;
        let action = handle_event(&mut state, &key_event(KeyCode::Enter));
        assert_eq!(action, InputAction::Submit("hello".into()));
        assert!(state.input.is_empty());
    }

    #[test]
    fn empty_enter_does_nothing() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &key_event(KeyCode::Enter));
        assert_eq!(action, InputAction::None);
    }

    #[test]
    fn ctrl_c_quits() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &ctrl_key('c'));
        assert_eq!(action, InputAction::Quit);
        assert!(state.should_quit);
    }

    #[test]
    fn ctrl_d_quits_on_empty() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &ctrl_key('d'));
        assert_eq!(action, InputAction::Quit);
    }

    #[test]
    fn ctrl_d_does_not_quit_with_input() {
        let mut state = AppState::new("test");
        state.input = "text".into();
        let action = handle_event(&mut state, &ctrl_key('d'));
        assert_eq!(action, InputAction::None);
    }

    #[test]
    fn backspace_deletes() {
        let mut state = AppState::new("test");
        state.input = "ab".into();
        state.cursor = 2;
        let action = handle_event(&mut state, &key_event(KeyCode::Backspace));
        assert_eq!(action, InputAction::Redraw);
        assert_eq!(state.input, "a");
    }

    #[test]
    fn arrow_keys() {
        let mut state = AppState::new("test");
        state.input = "abc".into();
        state.cursor = 3;

        handle_event(&mut state, &key_event(KeyCode::Left));
        assert_eq!(state.cursor, 2);

        handle_event(&mut state, &key_event(KeyCode::Right));
        assert_eq!(state.cursor, 3);
    }

    #[test]
    fn home_end_keys() {
        let mut state = AppState::new("test");
        state.input = "hello".into();
        state.cursor = 3;

        handle_event(&mut state, &key_event(KeyCode::Home));
        assert_eq!(state.cursor, 0);

        handle_event(&mut state, &key_event(KeyCode::End));
        assert_eq!(state.cursor, 5);
    }

    #[test]
    fn esc_clears_input() {
        let mut state = AppState::new("test");
        state.input = "hello".into();
        state.cursor = 5;

        let action = handle_event(&mut state, &key_event(KeyCode::Escape));
        assert_eq!(action, InputAction::Redraw);
        assert!(state.input.is_empty());
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn resize_triggers_redraw() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &Event::Resize(80, 24));
        assert_eq!(action, InputAction::Redraw);
    }

    fn shift_ctrl_key(c: char) -> Event {
        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            modifiers: Modifiers::CTRL | Modifiers::SHIFT,
        })
    }

    #[test]
    fn ctrl_p_cycles_model_forward() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &ctrl_key('p'));
        assert_eq!(action, InputAction::CycleModel);
    }

    #[test]
    fn shift_ctrl_p_cycles_model_backward() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &shift_ctrl_key('p'));
        assert_eq!(action, InputAction::CycleModelBackward);
    }

    #[test]
    fn ctrl_p_blocked_while_thinking() {
        let mut state = AppState::new("test");
        state.status = crate::app::AppStatus::Thinking;
        let action = handle_event(&mut state, &ctrl_key('p'));
        assert_eq!(action, InputAction::None);
    }

    #[test]
    fn no_input_while_thinking() {
        let mut state = AppState::new("test");
        state.status = crate::app::AppStatus::Thinking;
        let action = handle_event(&mut state, &key_event(KeyCode::Char('a')));
        assert_eq!(action, InputAction::None);
    }

    #[test]
    fn page_up_scrolls() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &key_event(KeyCode::PageUp));
        assert_eq!(action, InputAction::ScrollUp(PAGE_SCROLL_LINES));
    }

    #[test]
    fn page_down_scrolls() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &key_event(KeyCode::PageDown));
        assert_eq!(action, InputAction::ScrollDown(PAGE_SCROLL_LINES));
    }

    #[test]
    fn mouse_scroll_up() {
        let mut state = AppState::new("test");
        let event = Event::Mouse(saorsa_core::event::MouseEvent {
            kind: MouseEventKind::ScrollUp,
            x: 0,
            y: 0,
            modifiers: Modifiers::NONE,
        });
        let action = handle_event(&mut state, &event);
        assert_eq!(action, InputAction::ScrollUp(MOUSE_SCROLL_LINES));
    }

    #[test]
    fn mouse_scroll_down() {
        let mut state = AppState::new("test");
        let event = Event::Mouse(saorsa_core::event::MouseEvent {
            kind: MouseEventKind::ScrollDown,
            x: 0,
            y: 0,
            modifiers: Modifiers::NONE,
        });
        let action = handle_event(&mut state, &event);
        assert_eq!(action, InputAction::ScrollDown(MOUSE_SCROLL_LINES));
    }

    #[test]
    fn tab_triggers_autocomplete() {
        let mut state = AppState::new("test");
        state.input = "/mod".into();
        state.cursor = 4;
        let action = handle_event(&mut state, &key_event(KeyCode::Tab));
        assert_eq!(action, InputAction::TabComplete);
    }

    #[test]
    fn tab_blocked_while_thinking() {
        let mut state = AppState::new("test");
        state.status = crate::app::AppStatus::Thinking;
        let action = handle_event(&mut state, &key_event(KeyCode::Tab));
        assert_eq!(action, InputAction::None);
    }

    #[test]
    fn ctrl_l_opens_model_selector() {
        let mut state = AppState::new("test");
        let action = handle_event(&mut state, &ctrl_key('l'));
        assert_eq!(action, InputAction::OpenModelSelector);
    }

    #[test]
    fn ctrl_l_blocked_while_thinking() {
        let mut state = AppState::new("test");
        state.status = crate::app::AppStatus::Thinking;
        let action = handle_event(&mut state, &ctrl_key('l'));
        assert_eq!(action, InputAction::None);
    }

    #[test]
    fn page_up_works_while_thinking() {
        let mut state = AppState::new("test");
        state.status = crate::app::AppStatus::Thinking;
        let action = handle_event(&mut state, &key_event(KeyCode::PageUp));
        assert_eq!(action, InputAction::ScrollUp(PAGE_SCROLL_LINES));
    }
}
