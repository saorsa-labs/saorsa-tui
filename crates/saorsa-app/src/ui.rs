//! UI rendering for the chat interface.

use saorsa_core::geometry::Rect;
use saorsa_core::layout::{Constraint, Direction, Layout};
use saorsa_core::style::Style;
use saorsa_core::widget::Widget;
use saorsa_core::widget::container::{BorderStyle, Container};
use saorsa_core::widget::label::{Alignment, Label};
use saorsa_core::{Color, ScreenBuffer};

use crate::app::{AppState, AppStatus, ChatRole};

/// Render the application UI into the screen buffer.
pub fn render(state: &AppState, buf: &mut ScreenBuffer) {
    let area = Rect::new(0, 0, buf.width(), buf.height());

    // Split into header (1 row), body (fill), footer (3 rows).
    let chunks = Layout::split(
        area,
        Direction::Vertical,
        &[Constraint::Fixed(1), Constraint::Fill, Constraint::Fixed(3)],
    );

    render_header(state, buf, chunks[0]);
    render_messages(state, buf, chunks[1]);
    render_input(state, buf, chunks[2]);
}

/// Render the header bar showing model and status.
fn render_header(state: &AppState, buf: &mut ScreenBuffer, area: Rect) {
    let status_text = match &state.status {
        AppStatus::Idle => "Ready",
        AppStatus::Thinking => "Thinking...",
        AppStatus::ToolRunning { tool_name } => tool_name.as_str(),
    };

    let header_text = format!(" saorsa-tui | {} | {}", state.model, status_text);

    let style = Style::default()
        .fg(Color::Named(saorsa_core::color::NamedColor::Black))
        .bg(Color::Named(saorsa_core::color::NamedColor::White))
        .bold(true);

    let label = Label::new(&header_text)
        .alignment(Alignment::Left)
        .style(style);
    label.render(area, buf);
}

/// Render the message history.
fn render_messages(state: &AppState, buf: &mut ScreenBuffer, area: Rect) {
    if area.size.height == 0 {
        return;
    }

    // Calculate how many messages we can show (1 line per message, simplified).
    let max_visible = area.size.height as usize;

    // Show the most recent messages.
    let start = if state.messages.len() > max_visible {
        state.messages.len() - max_visible
    } else {
        0
    };

    let visible_messages = &state.messages[start..];

    for (i, msg) in visible_messages.iter().enumerate() {
        let y = area.position.y + i as u16;
        if y >= area.position.y + area.size.height {
            break;
        }

        let row_area = Rect::new(area.position.x, y, area.size.width, 1);

        let (prefix, style) = match &msg.role {
            ChatRole::User => (
                "> ",
                Style::default()
                    .fg(Color::Named(saorsa_core::color::NamedColor::Green))
                    .bold(true),
            ),
            ChatRole::Assistant => (
                "  ",
                Style::default().fg(Color::Named(saorsa_core::color::NamedColor::Cyan)),
            ),
            ChatRole::Tool { name } => {
                let prefix_str = format!("  [{name}] ");
                let style = Style::default()
                    .fg(Color::Named(saorsa_core::color::NamedColor::Yellow))
                    .dim(true);
                // Render tool prefix inline.
                let label = Label::new(format!("{prefix_str}{}", msg.content)).style(style);
                label.render(row_area, buf);
                continue;
            }
            ChatRole::System => (
                "  ",
                Style::default()
                    .fg(Color::Named(saorsa_core::color::NamedColor::Magenta))
                    .italic(true),
            ),
        };

        let text = format!("{prefix}{}", msg.content);
        let label = Label::new(&text).style(style);
        label.render(row_area, buf);
    }

    // If we're streaming, show the current streaming text.
    if !state.streaming_text.is_empty() && !visible_messages.is_empty() {
        let y = area.position.y + visible_messages.len() as u16;
        if y < area.position.y + area.size.height {
            let row_area = Rect::new(area.position.x, y, area.size.width, 1);
            let style = Style::default().fg(Color::Named(saorsa_core::color::NamedColor::Cyan));
            let text = format!("  {}", state.streaming_text);
            let label = Label::new(&text).style(style);
            label.render(row_area, buf);
        }
    }
}

/// Render the input area with a border.
fn render_input(state: &AppState, buf: &mut ScreenBuffer, area: Rect) {
    let container = Container::new()
        .border(BorderStyle::Rounded)
        .title(if state.is_idle() {
            "Type a message"
        } else {
            "Waiting..."
        });
    container.render(area, buf);

    // Render the input text inside the container.
    let inner = container.inner_area(area);
    if inner.size.height > 0 && inner.size.width > 0 {
        let label = Label::new(&state.input);
        label.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_core::geometry::Size;

    #[test]
    fn render_empty_state() {
        let state = AppState::new("test-model");
        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        render(&state, &mut buf);
        // Should not panic.
    }

    #[test]
    fn render_with_messages() {
        let mut state = AppState::new("test-model");
        state.add_user_message("Hello");
        state.add_assistant_message("Hi there!");
        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        render(&state, &mut buf);
        // Should not panic.
    }

    #[test]
    fn render_with_streaming() {
        let mut state = AppState::new("test-model");
        state.add_user_message("Hello");
        state.streaming_text = "Partial response...".into();
        state.status = AppStatus::Thinking;
        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        render(&state, &mut buf);
        // Should not panic.
    }

    #[test]
    fn render_small_terminal() {
        let state = AppState::new("test-model");
        let mut buf = ScreenBuffer::new(Size::new(20, 5));
        render(&state, &mut buf);
        // Should not panic even with minimal space.
    }

    #[test]
    fn render_many_messages() {
        let mut state = AppState::new("test-model");
        for i in 0..100 {
            state.add_user_message(format!("Message {i}"));
        }
        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        render(&state, &mut buf);
        // Should only show the last few messages.
    }

    #[test]
    fn render_tool_message() {
        let mut state = AppState::new("test-model");
        state.add_tool_message("bash", "ls output");
        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        render(&state, &mut buf);
    }

    #[test]
    fn render_system_message() {
        let mut state = AppState::new("test-model");
        state.add_system_message("Connected to Claude");
        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        render(&state, &mut buf);
    }
}
