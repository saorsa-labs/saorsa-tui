//! Application state for the chat interface.

/// Active overlay mode for the application.
///
/// Overlays capture input while visible. The main input field is inactive
/// when an overlay is active.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayMode {
    /// No overlay — normal input mode.
    #[default]
    None,
    /// Model selector overlay (Ctrl+L).
    ModelSelector,
    /// Settings screen overlay (/settings --ui).
    Settings,
}

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
    /// Available model IDs for cycling.
    pub enabled_models: Vec<String>,
    /// Current position in enabled_models list.
    pub model_index: usize,
    /// Extended-thinking level for the current session.
    pub thinking_level: saorsa_agent::ThinkingLevel,
    /// Whether compact display mode is active.
    pub compact_mode: bool,
    /// Cost tracker for the current session.
    pub cost_tracker: saorsa_agent::CostTracker,
    /// Active overlay mode.
    pub overlay_mode: OverlayMode,
    /// Scroll offset: number of messages scrolled up from the bottom.
    ///
    /// 0 = at the bottom (latest messages visible).
    scroll_offset: usize,
    /// Whether the UI needs to be re-rendered.
    dirty: bool,
    /// Pending stream text accumulated between render frames.
    pending_stream_text: String,
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
            enabled_models: Vec::new(),
            model_index: 0,
            thinking_level: saorsa_agent::ThinkingLevel::default(),
            compact_mode: false,
            cost_tracker: saorsa_agent::CostTracker::new(),
            overlay_mode: OverlayMode::None,
            scroll_offset: 0,
            dirty: true,
            pending_stream_text: String::new(),
        }
    }

    /// Mark the UI state as needing a re-render.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if the state is dirty and clear the flag.
    ///
    /// Returns `true` if a render is needed, then resets the flag to `false`.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::replace(&mut self.dirty, false)
    }

    /// Accumulate streaming text without triggering a render.
    ///
    /// Text is buffered and flushed to `streaming_text` at the next frame
    /// boundary via [`flush_stream_text`](Self::flush_stream_text).
    pub fn accumulate_stream_text(&mut self, text: &str) {
        self.pending_stream_text.push_str(text);
    }

    /// Flush pending stream text into `streaming_text` and mark dirty.
    ///
    /// Returns `true` if any text was flushed.
    pub fn flush_stream_text(&mut self) -> bool {
        if self.pending_stream_text.is_empty() {
            return false;
        }
        self.streaming_text.push_str(&self.pending_stream_text);
        self.pending_stream_text.clear();
        self.dirty = true;
        true
    }

    /// Add a user message to the chat.
    ///
    /// User messages always scroll to the bottom since the user is actively
    /// participating.
    pub fn add_user_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: ChatRole::User,
            content: text.into(),
        });
        self.scroll_offset = 0;
        self.dirty = true;
    }

    /// Add an assistant message to the chat.
    pub fn add_assistant_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: text.into(),
        });
        self.dirty = true;
    }

    /// Add a tool result message to the chat.
    pub fn add_tool_message(&mut self, name: impl Into<String>, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: ChatRole::Tool { name: name.into() },
            content: content.into(),
        });
        self.dirty = true;
    }

    /// Add a system notification to the chat.
    pub fn add_system_message(&mut self, text: impl Into<String>) {
        self.messages.push(ChatMessage {
            role: ChatRole::System,
            content: text.into(),
        });
        self.dirty = true;
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.dirty = true;
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
            self.dirty = true;
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
            self.dirty = true;
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
            self.dirty = true;
        }
    }

    /// Take the current input text and clear it.
    pub fn take_input(&mut self) -> String {
        self.cursor = 0;
        self.dirty = true;
        std::mem::take(&mut self.input)
    }

    /// Cycle to the next model in the enabled list.
    ///
    /// Returns `None` if fewer than two models are available.
    pub fn cycle_model_forward(&mut self) -> Option<&str> {
        if self.enabled_models.len() < 2 {
            return None;
        }
        self.model_index = (self.model_index + 1) % self.enabled_models.len();
        Some(&self.enabled_models[self.model_index])
    }

    /// Cycle to the previous model in the enabled list.
    ///
    /// Returns `None` if fewer than two models are available.
    pub fn cycle_model_backward(&mut self) -> Option<&str> {
        if self.enabled_models.len() < 2 {
            return None;
        }
        if self.model_index == 0 {
            self.model_index = self.enabled_models.len() - 1;
        } else {
            self.model_index -= 1;
        }
        Some(&self.enabled_models[self.model_index])
    }

    /// Check if the app is idle.
    pub fn is_idle(&self) -> bool {
        self.status == AppStatus::Idle
    }

    /// Scroll up by the given number of lines.
    ///
    /// The offset is clamped so it never exceeds the message count.
    pub fn scroll_up(&mut self, lines: usize) {
        let max_offset = self.messages.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
        self.dirty = true;
    }

    /// Scroll down by the given number of lines (towards latest messages).
    ///
    /// The offset is clamped to zero (the bottom).
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        self.dirty = true;
    }

    /// Jump to the bottom of the message history.
    pub fn scroll_to_bottom(&mut self) {
        if self.scroll_offset != 0 {
            self.scroll_offset = 0;
            self.dirty = true;
        }
    }

    /// Whether the user has scrolled up from the bottom.
    pub fn is_scrolled_up(&self) -> bool {
        self.scroll_offset > 0
    }

    /// Current scroll offset (number of messages from the bottom).
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
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
    fn cycle_model_forward() {
        let mut state = AppState::new("model-a");
        state.enabled_models = vec!["model-a".into(), "model-b".into(), "model-c".into()];
        state.model_index = 0;

        assert_eq!(state.cycle_model_forward(), Some("model-b"));
        assert_eq!(state.model_index, 1);
        assert_eq!(state.cycle_model_forward(), Some("model-c"));
        assert_eq!(state.model_index, 2);
        // Wraps around.
        assert_eq!(state.cycle_model_forward(), Some("model-a"));
        assert_eq!(state.model_index, 0);
    }

    #[test]
    fn cycle_model_backward() {
        let mut state = AppState::new("model-a");
        state.enabled_models = vec!["model-a".into(), "model-b".into(), "model-c".into()];
        state.model_index = 0;

        // Wraps to end.
        assert_eq!(state.cycle_model_backward(), Some("model-c"));
        assert_eq!(state.model_index, 2);
        assert_eq!(state.cycle_model_backward(), Some("model-b"));
        assert_eq!(state.model_index, 1);
        assert_eq!(state.cycle_model_backward(), Some("model-a"));
        assert_eq!(state.model_index, 0);
    }

    #[test]
    fn cycle_model_returns_none_when_empty() {
        let mut state = AppState::new("test");
        assert!(state.cycle_model_forward().is_none());
        assert!(state.cycle_model_backward().is_none());
    }

    #[test]
    fn cycle_model_returns_none_with_single_model() {
        let mut state = AppState::new("test");
        state.enabled_models = vec!["only-model".into()];
        assert!(state.cycle_model_forward().is_none());
        assert!(state.cycle_model_backward().is_none());
    }

    #[test]
    fn new_state_is_dirty() {
        let state = AppState::new("test");
        assert!(state.dirty);
    }

    #[test]
    fn take_dirty_returns_true_then_false() {
        let mut state = AppState::new("test");
        assert!(state.take_dirty());
        assert!(!state.take_dirty());
    }

    #[test]
    fn mark_dirty_sets_flag() {
        let mut state = AppState::new("test");
        state.take_dirty(); // clear
        state.mark_dirty();
        assert!(state.take_dirty());
    }

    #[test]
    fn add_messages_mark_dirty() {
        let mut state = AppState::new("test");
        state.take_dirty(); // clear initial

        state.add_user_message("hi");
        assert!(state.take_dirty());

        state.add_assistant_message("hello");
        assert!(state.take_dirty());

        state.add_tool_message("bash", "ok");
        assert!(state.take_dirty());

        state.add_system_message("info");
        assert!(state.take_dirty());
    }

    #[test]
    fn input_editing_marks_dirty() {
        let mut state = AppState::new("test");
        state.take_dirty(); // clear

        state.insert_char('a');
        assert!(state.take_dirty());

        state.delete_char_before();
        assert!(state.take_dirty());
    }

    #[test]
    fn cursor_movement_marks_dirty() {
        let mut state = AppState::new("test");
        state.input = "abc".into();
        state.cursor = 1;
        state.take_dirty(); // clear

        state.cursor_left();
        assert!(state.take_dirty());

        state.cursor_right();
        assert!(state.take_dirty());
    }

    #[test]
    fn accumulate_stream_text_does_not_mark_dirty() {
        let mut state = AppState::new("test");
        state.take_dirty(); // clear
        state.accumulate_stream_text("hello");
        assert!(!state.take_dirty());
        assert!(state.pending_stream_text == "hello");
    }

    #[test]
    fn flush_stream_text_moves_to_streaming() {
        let mut state = AppState::new("test");
        state.take_dirty(); // clear
        state.accumulate_stream_text("hello ");
        state.accumulate_stream_text("world");
        assert!(state.flush_stream_text());
        assert_eq!(state.streaming_text, "hello world");
        assert!(state.pending_stream_text.is_empty());
        assert!(state.take_dirty());
    }

    #[test]
    fn flush_stream_text_returns_false_when_empty() {
        let mut state = AppState::new("test");
        state.take_dirty(); // clear
        assert!(!state.flush_stream_text());
        assert!(!state.take_dirty());
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

    #[test]
    fn scroll_offset_starts_at_zero() {
        let state = AppState::new("test");
        assert_eq!(state.scroll_offset(), 0);
        assert!(!state.is_scrolled_up());
    }

    #[test]
    fn scroll_up_increases_offset() {
        let mut state = AppState::new("test");
        for i in 0..20 {
            state.add_user_message(format!("msg {i}"));
        }
        state.take_dirty(); // clear

        state.scroll_up(5);
        assert_eq!(state.scroll_offset(), 5);
        assert!(state.is_scrolled_up());
        assert!(state.take_dirty());
    }

    #[test]
    fn scroll_up_clamps_to_max() {
        let mut state = AppState::new("test");
        for i in 0..10 {
            state.add_user_message(format!("msg {i}"));
        }
        // Max offset is messages.len() - 1 = 9.
        state.scroll_up(100);
        assert_eq!(state.scroll_offset(), 9);
    }

    #[test]
    fn scroll_up_no_messages_stays_zero() {
        let mut state = AppState::new("test");
        state.scroll_up(5);
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn scroll_down_decreases_offset() {
        let mut state = AppState::new("test");
        for i in 0..20 {
            state.add_user_message(format!("msg {i}"));
        }
        state.scroll_up(10);
        state.scroll_down(3);
        assert_eq!(state.scroll_offset(), 7);
    }

    #[test]
    fn scroll_down_clamps_to_zero() {
        let mut state = AppState::new("test");
        for i in 0..10 {
            state.add_user_message(format!("msg {i}"));
        }
        state.scroll_up(5);
        state.scroll_down(100);
        assert_eq!(state.scroll_offset(), 0);
        assert!(!state.is_scrolled_up());
    }

    #[test]
    fn scroll_to_bottom_resets_offset() {
        let mut state = AppState::new("test");
        for i in 0..10 {
            state.add_user_message(format!("msg {i}"));
        }
        state.scroll_up(5);
        state.take_dirty(); // clear

        state.scroll_to_bottom();
        assert_eq!(state.scroll_offset(), 0);
        assert!(state.take_dirty());
    }

    #[test]
    fn scroll_to_bottom_noop_when_already_at_bottom() {
        let mut state = AppState::new("test");
        state.take_dirty(); // clear
        state.scroll_to_bottom();
        // Should not mark dirty since offset was already 0.
        assert!(!state.take_dirty());
    }

    #[test]
    fn add_user_message_scrolls_to_bottom() {
        let mut state = AppState::new("test");
        for i in 0..20 {
            state.add_user_message(format!("msg {i}"));
        }
        state.scroll_up(10);
        assert!(state.is_scrolled_up());

        state.add_user_message("new message");
        assert!(!state.is_scrolled_up());
        assert_eq!(state.scroll_offset(), 0);
    }

    #[test]
    fn scroll_marks_dirty() {
        let mut state = AppState::new("test");
        for i in 0..10 {
            state.add_user_message(format!("msg {i}"));
        }
        state.take_dirty(); // clear

        state.scroll_up(3);
        assert!(state.take_dirty());

        state.scroll_down(1);
        assert!(state.take_dirty());
    }
}
