//! Application state for the chat interface.

/// Current status of the application.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppStatus {
    /// Idle, waiting for user input.
    Idle,
    /// Waiting for LLM response.
    Thinking,
    /// A tool is currently executing.
    ToolRunning {
        /// Name of the running tool.
        tool_name: String,
    },
}

/// A message displayed in the chat interface.
#[derive(Clone, Debug)]
pub struct ChatMessage {
    /// Who sent this message.
    pub role: ChatRole,
    /// The content of the message.
    pub content: String,
}

/// Role of a chat message sender.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatRole {
    /// The user.
    User,
    /// The AI assistant.
    Assistant,
    /// A tool result.
    Tool {
        /// Name of the tool.
        name: String,
    },
    /// System notification.
    System,
}

/// Application state.
pub struct AppState {
    /// Chat message history for display.
    pub messages: Vec<ChatMessage>,
    /// Current input text.
    pub input: String,
    /// Input cursor position (byte offset).
    pub cursor: usize,
    /// Current application status.
    pub status: AppStatus,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Model name for display.
    pub model: String,
    /// Streaming text buffer for current assistant response.
    pub streaming_text: String,
}

impl AppState {
    /// Create a new app state.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            cursor: 0,
            status: AppStatus::Idle,
            should_quit: false,
            model: model.into(),
            streaming_text: String::new(),
        }
    }

    /// Add a user message to the chat.
    pub fn add_user_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: ChatRole::User,
            content: text.into(),
        });
    }

    /// Add an assistant message to the chat.
    pub fn add_assistant_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: text.into(),
        });
    }

    /// Add a tool result message to the chat.
    pub fn add_tool_message(&mut self, name: impl Into<String>, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: ChatRole::Tool {
                name: name.into(),
            },
            content: content.into(),
        });
    }

    /// Add a system notification to the chat.
    pub fn add_system_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: ChatRole::System,
            content: text.into(),
        });
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char_before(&mut self) {
        if self.cursor > 0 {
            // Find the previous character boundary.
            let prev = self.input[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.input.remove(prev);
            self.cursor = prev;
        }
    }

    /// Move cursor left.
    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.input[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right.
    pub fn cursor_right(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor = self.input[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.input.len());
        }
    }

    /// Take the current input text and clear it.
    pub fn take_input(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.input)
    }

    /// Check if the app is idle.
    pub fn is_idle(&self) -> bool {
        self.status == AppStatus::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state() {
        let state = AppState::new("test-model");
        assert!(state.messages.is_empty());
        assert!(state.input.is_empty());
        assert_eq!(state.cursor, 0);
        assert_eq!(state.status, AppStatus::Idle);
        assert!(!state.should_quit);
        assert_eq!(state.model, "test-model");
    }

    #[test]
    fn add_messages() {
        let mut state = AppState::new("test");
        state.add_user_message("Hello");
        state.add_assistant_message("Hi there");
        state.add_tool_message("bash", "output");
        state.add_system_message("Connected");
        assert_eq!(state.messages.len(), 4);
        assert_eq!(state.messages[0].role, ChatRole::User);
        assert_eq!(state.messages[1].role, ChatRole::Assistant);
        assert!(matches!(state.messages[2].role, ChatRole::Tool { .. }));
        assert_eq!(state.messages[3].role, ChatRole::System);
    }

    #[test]
    fn input_editing() {
        let mut state = AppState::new("test");
        state.insert_char('h');
        state.insert_char('i');
        assert_eq!(state.input, "hi");
        assert_eq!(state.cursor, 2);

        state.delete_char_before();
        assert_eq!(state.input, "h");
        assert_eq!(state.cursor, 1);
    }

    #[test]
    fn cursor_movement() {
        let mut state = AppState::new("test");
        state.input = "abc".into();
        state.cursor = 3;

        state.cursor_left();
        assert_eq!(state.cursor, 2);
        state.cursor_left();
        assert_eq!(state.cursor, 1);
        state.cursor_right();
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn cursor_at_boundaries() {
        let mut state = AppState::new("test");
        state.input = "ab".into();
        state.cursor = 0;
        state.cursor_left(); // Should stay at 0.
        assert_eq!(state.cursor, 0);

        state.cursor = 2;
        state.cursor_right(); // Should stay at end.
        assert_eq!(state.cursor, 2);
    }

    #[test]
    fn take_input() {
        let mut state = AppState::new("test");
        state.input = "hello".into();
        state.cursor = 5;
        let text = state.take_input();
        assert_eq!(text, "hello");
        assert!(state.input.is_empty());
        assert_eq!(state.cursor, 0);
    }

    #[test]
    fn delete_char_at_start() {
        let mut state = AppState::new("test");
        state.input = "abc".into();
        state.cursor = 0;
        state.delete_char_before(); // Should be no-op.
        assert_eq!(state.input, "abc");
    }

    #[test]
    fn status_checks() {
        let state = AppState::new("test");
        assert!(state.is_idle());
    }

    #[test]
    fn unicode_input() {
        let mut state = AppState::new("test");
        state.insert_char('e');
        state.insert_char('\u{0301}'); // Combining acute accent.
        assert_eq!(state.cursor, 3); // 'e' + 2 bytes for combining.
        state.delete_char_before();
        assert_eq!(state.input, "e");
    }
}
