//! Input event handling for the chat application.

use fae_core::event::{Event, KeyCode, Modifiers};

use crate::app::AppState;

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
}

/// Handle an input event and return the resulting action.
pub fn handle_event(state: &mut AppState, event: &Event) -> InputAction {
    match event {
        Event::Key(key) => handle_key(state, key.code.clone(), key.modifiers),
        Event::Resize(_, _) => InputAction::Redraw,
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

    // Only process editing keys when idle.
    if !state.is_idle() {
        return InputAction::None;
    }

    match code {
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
    use fae_core::event::KeyEvent;

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

    #[test]
    fn no_input_while_thinking() {
        let mut state = AppState::new("test");
        state.status = crate::app::AppStatus::Thinking;
        let action = handle_event(&mut state, &key_event(KeyCode::Char('a')));
        assert_eq!(action, InputAction::None);
    }
}
