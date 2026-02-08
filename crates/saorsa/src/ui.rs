//! UI rendering for the chat interface.

use saorsa_tui::geometry::Rect;
use saorsa_tui::layout::{Constraint, Direction, Layout};
use saorsa_tui::style::Style;
use saorsa_tui::widget::Widget;
use saorsa_tui::widget::container::{BorderStyle, Container};
use saorsa_tui::widget::label::{Alignment, Label};
use saorsa_tui::{Color, ScreenBuffer};

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
    render_autocomplete(state, buf, chunks[2]);
}

/// Render the header bar showing model and status.
fn render_header(state: &AppState, buf: &mut ScreenBuffer, area: Rect) {
    let status_text = match &state.status {
        AppStatus::Idle => "Ready",
        AppStatus::Thinking => "Thinking...",
        AppStatus::ToolRunning { tool_name } => tool_name.as_str(),
    };

    let header_text = format!(" saorsa | {} | {}", state.model, status_text);

    let style = Style::default()
        .fg(Color::Named(saorsa_tui::color::NamedColor::Black))
        .bg(Color::Named(saorsa_tui::color::NamedColor::White))
        .bold(true);

    let label = Label::new(&header_text)
        .alignment(Alignment::Left)
        .style(style);
    label.render(area, buf);
}

/// Render the message history with scroll support.
fn render_messages(state: &AppState, buf: &mut ScreenBuffer, area: Rect) {
    if area.size.height == 0 {
        return;
    }

    let max_visible = area.size.height as usize;
    let total = state.messages.len();
    let scroll_offset = state.scroll_offset();

    // Calculate the window of messages to display.
    // scroll_offset is from the bottom: 0 = latest, N = N messages from bottom.
    let end = total.saturating_sub(scroll_offset);
    let start = end.saturating_sub(max_visible);
    let visible_messages = &state.messages[start..end];

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
                    .fg(Color::Named(saorsa_tui::color::NamedColor::Green))
                    .bold(true),
            ),
            ChatRole::Assistant => (
                "  ",
                Style::default().fg(Color::Named(saorsa_tui::color::NamedColor::Cyan)),
            ),
            ChatRole::Tool { name } => {
                let prefix_str = format!("  [{name}] ");
                let style = Style::default()
                    .fg(Color::Named(saorsa_tui::color::NamedColor::Yellow))
                    .dim(true);
                // Render tool prefix inline.
                let label = Label::new(format!("{prefix_str}{}", msg.content)).style(style);
                label.render(row_area, buf);
                continue;
            }
            ChatRole::System => (
                "  ",
                Style::default()
                    .fg(Color::Named(saorsa_tui::color::NamedColor::Magenta))
                    .italic(true),
            ),
        };

        let text = format!("{prefix}{}", msg.content);
        let label = Label::new(&text).style(style);
        label.render(row_area, buf);
    }

    // Show streaming text only when at the bottom (not scrolled up).
    if !state.is_scrolled_up() && !state.streaming_text.is_empty() && !visible_messages.is_empty() {
        let y = area.position.y + visible_messages.len() as u16;
        if y < area.position.y + area.size.height {
            let row_area = Rect::new(area.position.x, y, area.size.width, 1);
            let style = Style::default().fg(Color::Named(saorsa_tui::color::NamedColor::Cyan));
            let text = format!("  {}", state.streaming_text);
            let label = Label::new(&text).style(style);
            label.render(row_area, buf);
        }
    }

    // Show scroll indicator when scrolled up.
    if state.is_scrolled_up() {
        let indicator = format!(" [{scroll_offset} more] ");
        let indicator_style = Style::default()
            .fg(Color::Named(saorsa_tui::color::NamedColor::Yellow))
            .bold(true);
        // Render at bottom-right of the message area.
        let indicator_len = indicator.len().min(area.size.width as usize);
        let x = area.position.x + area.size.width - indicator_len as u16;
        let y = area.position.y + area.size.height - 1;
        let indicator_area = Rect::new(x, y, indicator_len as u16, 1);
        let label = Label::new(&indicator).style(indicator_style);
        label.render(indicator_area, buf);
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

/// Render the autocomplete dropdown above the input area.
///
/// The dropdown appears directly above the input box and shows matching
/// command suggestions with descriptions. The selected item is highlighted.
fn render_autocomplete(state: &AppState, buf: &mut ScreenBuffer, input_area: Rect) {
    let suggestions = state.autocomplete_suggestions();
    if suggestions.is_empty() {
        return;
    }

    let max_visible = state.max_visible_suggestions();
    let visible_count = suggestions.len().min(max_visible);
    let selected = state.autocomplete_index();

    // Calculate scroll window within suggestions when there are more than max_visible.
    let (scroll_start, scroll_end) = if suggestions.len() <= max_visible {
        (0, suggestions.len())
    } else {
        // Keep selected item visible in the scroll window.
        let half = max_visible / 2;
        let start = if selected < half {
            0
        } else if selected + half >= suggestions.len() {
            suggestions.len() - max_visible
        } else {
            selected - half
        };
        (start, start + max_visible)
    };

    let visible_suggestions = &suggestions[scroll_start..scroll_end];

    // Calculate dropdown dimensions.
    // Height: visible items + 2 for border.
    let dropdown_height = (visible_count as u16) + 2;

    // Width: fit to longest suggestion, capped at terminal width - 4.
    let max_content_width = visible_suggestions
        .iter()
        .map(|s| {
            let desc_len = s.description.as_ref().map_or(0, |d| d.len() + 3); // " - desc"
            s.text.len() + desc_len
        })
        .max()
        .unwrap_or(10);
    // Add 2 for border padding, cap at terminal width - 4.
    let dropdown_width = (max_content_width + 4).min(buf.width() as usize).max(20) as u16;

    // Position: directly above the input box.
    let dropdown_y = input_area.position.y.saturating_sub(dropdown_height);
    let dropdown_x = input_area.position.x;

    let dropdown_area = Rect::new(dropdown_x, dropdown_y, dropdown_width, dropdown_height);

    // Draw border.
    let has_more = suggestions.len() > max_visible;
    let title = if has_more {
        format!("Commands ({}/{})", selected + 1, suggestions.len())
    } else {
        "Commands".to_string()
    };

    let dropdown_border_style = Style::default()
        .fg(Color::Named(saorsa_tui::color::NamedColor::Blue))
        .bold(true);

    let container = Container::new()
        .border(BorderStyle::Rounded)
        .title(&title)
        .border_style(dropdown_border_style);
    container.render(dropdown_area, buf);

    let inner = container.inner_area(dropdown_area);
    if inner.size.height == 0 || inner.size.width == 0 {
        return;
    }

    // Render each suggestion row.
    for (i, suggestion) in visible_suggestions.iter().enumerate() {
        let y = inner.position.y + i as u16;
        if y >= inner.position.y + inner.size.height {
            break;
        }

        let row_area = Rect::new(inner.position.x, y, inner.size.width, 1);
        let is_selected = (scroll_start + i) == selected;

        let desc_text = suggestion
            .description
            .as_ref()
            .map_or(String::new(), |d| format!(" - {d}"));

        let row_text = format!("{}{}", suggestion.text, desc_text);

        // Truncate to fit width.
        let display_width = inner.size.width as usize;
        let display_text = if row_text.len() > display_width {
            format!("{}...", &row_text[..display_width.saturating_sub(3)])
        } else {
            row_text
        };

        let style = if is_selected {
            Style::default()
                .fg(Color::Named(saorsa_tui::color::NamedColor::Black))
                .bg(Color::Named(saorsa_tui::color::NamedColor::Cyan))
                .bold(true)
        } else {
            Style::default().fg(Color::Named(saorsa_tui::color::NamedColor::White))
        };

        let label = Label::new(&display_text).style(style);
        label.render(row_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocomplete::Autocomplete;
    use saorsa_tui::geometry::Size;

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

    #[test]
    fn render_with_autocomplete_visible() {
        let mut state = AppState::new("test-model");
        state.input = "/".into();
        state.cursor = 1;
        let ac = Autocomplete::new();
        state.update_autocomplete(&ac);
        assert!(state.is_autocomplete_visible());

        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        render(&state, &mut buf);
        // Should not panic.
    }

    #[test]
    fn render_autocomplete_in_small_terminal() {
        let mut state = AppState::new("test-model");
        state.input = "/he".into();
        state.cursor = 3;
        let ac = Autocomplete::new();
        state.update_autocomplete(&ac);

        let mut buf = ScreenBuffer::new(Size::new(30, 8));
        render(&state, &mut buf);
        // Should not panic even with limited space.
    }

    #[test]
    fn render_autocomplete_no_suggestions() {
        let mut state = AppState::new("test-model");
        state.input = "/zzzzz".into();
        state.cursor = 6;
        let ac = Autocomplete::new();
        state.update_autocomplete(&ac);
        assert!(!state.is_autocomplete_visible());

        let mut buf = ScreenBuffer::new(Size::new(80, 24));
        render(&state, &mut buf);
        // Should not panic, no dropdown shown.
    }
}
