//! Message queue widget for batching messages.

use fae_core::buffer::ScreenBuffer;
use fae_core::cell::Cell;
use fae_core::color::NamedColor;
use fae_core::event::{Event, KeyCode, KeyEvent, Modifiers};
use fae_core::geometry::Rect;
use fae_core::style::Style;
use fae_core::widget::{EventResult, InteractiveWidget, Widget};

/// A queued message.
#[derive(Clone, Debug)]
pub struct QueuedMessage {
    /// Message text.
    pub text: String,
    /// Message ID.
    pub id: usize,
}

/// Message queue widget for batching messages before sending.
pub struct MessageQueue {
    /// Queued messages.
    messages: Vec<QueuedMessage>,
    /// Next message ID.
    next_id: usize,
    /// Selected message index.
    selected: usize,
    /// Whether the queue is visible.
    is_visible: bool,
}

impl MessageQueue {
    /// Create a new message queue.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            next_id: 1,
            selected: 0,
            is_visible: false,
        }
    }

    /// Add a message to the queue.
    pub fn add_message(&mut self, text: String) {
        let id = self.next_id;
        self.next_id += 1;
        self.messages.push(QueuedMessage { text, id });
        self.selected = self.messages.len().saturating_sub(1);
    }

    /// Remove the currently selected message.
    pub fn remove_selected(&mut self) {
        if !self.messages.is_empty() && self.selected < self.messages.len() {
            self.messages.remove(self.selected);
            if self.selected >= self.messages.len() {
                self.selected = self.messages.len().saturating_sub(1);
            }
        }
    }

    /// Get all queued messages.
    pub fn messages(&self) -> &[QueuedMessage] {
        &self.messages
    }

    /// Clear all messages.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.selected = 0;
    }

    /// Show the queue.
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    /// Hide the queue.
    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    /// Check if the queue is visible.
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Check if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get the number of queued messages.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Move selection up.
    fn move_up(&mut self) {
        if !self.messages.is_empty() {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Move selection down.
    fn move_down(&mut self) {
        if !self.messages.is_empty() {
            let max = self.messages.len().saturating_sub(1);
            self.selected = (self.selected + 1).min(max);
        }
    }

    /// Helper to write text to buffer.
    fn write_text(&self, buffer: &mut ScreenBuffer, x: u16, y: u16, text: &str, style: Style) {
        for (i, ch) in text.chars().enumerate() {
            if let Some(cell) = buffer.get_mut(x + i as u16, y) {
                *cell = Cell::new(ch.to_string(), style.clone());
            }
        }
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MessageQueue {
    fn render(&self, area: Rect, buffer: &mut ScreenBuffer) {
        if !self.is_visible {
            return;
        }

        // Render header
        self.write_text(
            buffer,
            area.position.x + 2,
            area.position.y,
            "Message Queue",
            Style::default().bold(true),
        );

        // Render message count
        let count_text = format!(" ({} messages)", self.messages.len());
        self.write_text(
            buffer,
            area.position.x + 18,
            area.position.y,
            &count_text,
            Style::default(),
        );

        // Render messages
        let mut y = area.position.y + 2;
        for (idx, msg) in self.messages.iter().enumerate() {
            if y >= area.position.y + area.size.height {
                break;
            }

            let style = if idx == self.selected {
                Style::default().reverse(true)
            } else {
                Style::default()
            };

            let prefix = format!("{}. ", idx + 1);
            self.write_text(buffer, area.position.x + 2, y, &prefix, style.clone());

            let text_start = area.position.x + 5;
            let max_width = (area.size.width.saturating_sub(7)) as usize;
            let truncated = if msg.text.len() > max_width {
                format!("{}...", &msg.text[..max_width.saturating_sub(3)])
            } else {
                msg.text.clone()
            };
            self.write_text(buffer, text_start, y, &truncated, style);

            y += 1;
        }

        // Render help text
        let help_y = area.position.y + area.size.height.saturating_sub(2);
        if help_y > area.position.y {
            self.write_text(
                buffer,
                area.position.x + 2,
                help_y,
                "[Enter] Send All  [D] Delete  [X] Clear  [Q] Close",
                Style::default().fg(fae_core::color::Color::Named(NamedColor::Cyan)),
            );
        }
    }
}

impl InteractiveWidget for MessageQueue {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        if !self.is_visible {
            return EventResult::Ignored;
        }

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: Modifiers::NONE,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('Q'),
                modifiers: Modifiers::SHIFT,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Escape,
                modifiers: Modifiers::NONE,
            }) => {
                self.hide();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: Modifiers::NONE,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('D'),
                modifiers: Modifiers::SHIFT,
            }) => {
                self.remove_selected();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: Modifiers::NONE,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('X'),
                modifiers: Modifiers::SHIFT,
            }) => {
                self.clear();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: Modifiers::NONE,
            }) => {
                self.move_up();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: Modifiers::NONE,
            }) => {
                self.move_down();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: Modifiers::NONE,
            }) => {
                // Send all - handled by application
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue() {
        let queue = MessageQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert!(!queue.is_visible());
    }

    #[test]
    fn add_message() {
        let mut queue = MessageQueue::new();
        queue.add_message("Hello".to_string());
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
    }

    #[test]
    fn remove_selected() {
        let mut queue = MessageQueue::new();
        queue.add_message("First".to_string());
        queue.add_message("Second".to_string());
        assert_eq!(queue.len(), 2);

        queue.remove_selected();
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn clear_queue() {
        let mut queue = MessageQueue::new();
        queue.add_message("One".to_string());
        queue.add_message("Two".to_string());
        assert_eq!(queue.len(), 2);

        queue.clear();
        assert!(queue.is_empty());
    }

    #[test]
    fn show_hide() {
        let mut queue = MessageQueue::new();
        assert!(!queue.is_visible());

        queue.show();
        assert!(queue.is_visible());

        queue.hide();
        assert!(!queue.is_visible());
    }

    #[test]
    fn move_selection() {
        let mut queue = MessageQueue::new();
        queue.add_message("One".to_string());
        queue.add_message("Two".to_string());
        queue.add_message("Three".to_string());

        queue.selected = 0;
        queue.move_down();
        assert_eq!(queue.selected, 1);

        queue.move_down();
        assert_eq!(queue.selected, 2);

        queue.move_down(); // Should stay at max
        assert_eq!(queue.selected, 2);

        queue.move_up();
        assert_eq!(queue.selected, 1);
    }

    #[test]
    fn handle_delete_key() {
        let mut queue = MessageQueue::new();
        queue.show();
        queue.add_message("Test".to_string());

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: Modifiers::NONE,
        });
        let result = queue.handle_event(&event);
        assert!(matches!(result, EventResult::Consumed));
        assert!(queue.is_empty());
    }

    #[test]
    fn handle_clear_key() {
        let mut queue = MessageQueue::new();
        queue.show();
        queue.add_message("One".to_string());
        queue.add_message("Two".to_string());

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: Modifiers::NONE,
        });
        let result = queue.handle_event(&event);
        assert!(matches!(result, EventResult::Consumed));
        assert!(queue.is_empty());
    }

    #[test]
    fn handle_escape_hides() {
        let mut queue = MessageQueue::new();
        queue.show();

        let event = Event::Key(KeyEvent {
            code: KeyCode::Escape,
            modifiers: Modifiers::NONE,
        });
        let result = queue.handle_event(&event);
        assert!(matches!(result, EventResult::Consumed));
        assert!(!queue.is_visible());
    }

    #[test]
    fn handle_event_when_hidden() {
        let mut queue = MessageQueue::new();
        let event = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: Modifiers::NONE,
        });
        let result = queue.handle_event(&event);
        assert!(matches!(result, EventResult::Ignored));
    }
}
