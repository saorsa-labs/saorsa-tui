//! Streaming markdown renderer for styled terminal output.
//!
//! Uses [`pulldown_cmark`] to parse CommonMark and produce styled
//! [`Segment`] lines suitable for rendering in a terminal. Designed
//! for incremental rendering of streaming LLM output.

use crate::color::{Color, NamedColor};
use crate::segment::Segment;
use crate::style::Style;
use crate::text::truncate_to_display_width;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use unicode_width::UnicodeWidthStr;

/// A block-level element in rendered markdown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MarkdownBlock {
    /// A paragraph of text.
    Paragraph,
    /// A heading (level 1-6).
    Heading(u8),
    /// A fenced or indented code block with optional language.
    CodeBlock(Option<String>),
    /// A list item with its nesting depth.
    ListItem(usize),
    /// A block quote.
    BlockQuote,
    /// A horizontal rule / thematic break.
    ThematicBreak,
    /// A table.
    Table,
}

/// A stateful incremental markdown renderer.
///
/// Text is pushed in chunks via [`push_str`](MarkdownRenderer::push_str)
/// and the full rendered output is produced by
/// [`render_to_lines`](MarkdownRenderer::render_to_lines). Incomplete
/// markdown is handled gracefully — the renderer re-parses the
/// accumulated text on each call.
pub struct MarkdownRenderer {
    text: String,
}

impl MarkdownRenderer {
    /// Create a new empty markdown renderer.
    pub fn new() -> Self {
        Self {
            text: String::new(),
        }
    }

    /// Append a text chunk (supports streaming).
    pub fn push_str(&mut self, text: &str) {
        self.text.push_str(text);
    }

    /// Clear all accumulated text and reset the renderer.
    pub fn clear(&mut self) {
        self.text.clear();
    }

    /// Render the current accumulated text to styled lines.
    ///
    /// Each line is a `Vec<Segment>`. The text is word-wrapped to
    /// the given width. Styles are applied for headings, bold, italic,
    /// inline code, code blocks, and list items.
    pub fn render_to_lines(&self, width: u16) -> Vec<Vec<Segment>> {
        let w = width as usize;
        if w == 0 || self.text.is_empty() {
            return Vec::new();
        }

        let mut lines: Vec<Vec<Segment>> = Vec::new();
        let mut style_stack: Vec<Style> = vec![Style::default()];
        let mut current_line: Vec<Segment> = Vec::new();
        let mut current_width: usize = 0;
        let mut in_code_block = false;
        let mut list_depth: usize = 0;
        let mut in_list_item = false;

        let opts = Options::empty();
        let parser = Parser::new_ext(&self.text, opts);

        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading { level, .. } => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        let level_num = level as u8;
                        let heading_style = heading_style(level_num);
                        style_stack.push(heading_style);
                    }
                    Tag::Paragraph => {
                        // Add blank line before paragraph (unless at start)
                        if !lines.is_empty() {
                            lines.push(Vec::new());
                        }
                    }
                    Tag::CodeBlock(kind) => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        in_code_block = true;
                        let _lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                                let l = lang.to_string();
                                if l.is_empty() { None } else { Some(l) }
                            }
                            pulldown_cmark::CodeBlockKind::Indented => None,
                        };
                        style_stack.push(code_block_style());
                    }
                    Tag::Emphasis => {
                        let base = current_style(&style_stack);
                        style_stack.push(base.italic(true));
                    }
                    Tag::Strong => {
                        let base = current_style(&style_stack);
                        style_stack.push(base.bold(true));
                    }
                    Tag::List(_) => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        list_depth += 1;
                    }
                    Tag::Item => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        in_list_item = true;
                        // Add list marker
                        let indent = "  ".repeat(list_depth.saturating_sub(1));
                        let marker = format!("{indent}- ");
                        let marker_w = UnicodeWidthStr::width(marker.as_str());
                        current_line.push(Segment::new(marker));
                        current_width = marker_w;
                    }
                    Tag::BlockQuote(_) => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        let base = current_style(&style_stack);
                        style_stack.push(base.dim(true));
                    }
                    _ => {}
                },
                Event::End(tag_end) => match tag_end {
                    TagEnd::Heading(_) => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        style_stack.pop();
                    }
                    TagEnd::Paragraph => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                    }
                    TagEnd::CodeBlock => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        in_code_block = false;
                        style_stack.pop();
                    }
                    TagEnd::Emphasis | TagEnd::Strong => {
                        style_stack.pop();
                    }
                    TagEnd::List(_) => {
                        list_depth = list_depth.saturating_sub(1);
                    }
                    TagEnd::Item => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        in_list_item = false;
                    }
                    TagEnd::BlockQuote(_) => {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                        style_stack.pop();
                    }
                    _ => {}
                },
                Event::Text(text) => {
                    let style = current_style(&style_stack);
                    if in_code_block {
                        // Code blocks: render each line as-is (no word wrap)
                        for (i, line) in text.lines().enumerate() {
                            if i > 0 {
                                flush_line(&mut lines, &mut current_line, &mut current_width);
                            }
                            let truncated = truncate_to_display_width(line, w);
                            current_line
                                .push(Segment::styled(truncated.to_string(), style.clone()));
                            current_width += UnicodeWidthStr::width(truncated);
                        }
                        if text.ends_with('\n') {
                            flush_line(&mut lines, &mut current_line, &mut current_width);
                        }
                    } else {
                        // Normal text: word wrap
                        let mut state = WrapState {
                            lines: &mut lines,
                            current_line: &mut current_line,
                            current_width: &mut current_width,
                            in_list_item,
                            list_depth,
                        };
                        wrap_text_into(&text, &style, w, &mut state);
                    }
                }
                Event::Code(code) => {
                    let style = inline_code_style();
                    let code_str = format!("`{code}`");
                    let code_w = UnicodeWidthStr::width(code_str.as_str());
                    if current_width + code_w > w && !current_line.is_empty() {
                        flush_line(&mut lines, &mut current_line, &mut current_width);
                    }
                    current_line.push(Segment::styled(code_str, style));
                    current_width += code_w;
                }
                Event::SoftBreak => {
                    // Treat as space
                    let style = current_style(&style_stack);
                    if current_width < w {
                        current_line.push(Segment::styled(" ".to_string(), style));
                        current_width += 1;
                    }
                }
                Event::HardBreak => {
                    flush_line(&mut lines, &mut current_line, &mut current_width);
                }
                Event::Rule => {
                    flush_line(&mut lines, &mut current_line, &mut current_width);
                    let rule = "─".repeat(w.min(80));
                    lines.push(vec![Segment::styled(rule, Style::new().dim(true))]);
                }
                _ => {}
            }
        }

        // Flush remaining content
        flush_line(&mut lines, &mut current_line, &mut current_width);

        lines
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Flush the current line segments into the lines list.
fn flush_line(
    lines: &mut Vec<Vec<Segment>>,
    current_line: &mut Vec<Segment>,
    current_width: &mut usize,
) {
    if !current_line.is_empty() {
        lines.push(std::mem::take(current_line));
    }
    *current_width = 0;
}

/// Get the current style from the style stack.
fn current_style(stack: &[Style]) -> Style {
    stack.last().cloned().unwrap_or_default()
}

/// Style for headings by level.
fn heading_style(level: u8) -> Style {
    match level {
        1 => Style::new().bold(true).fg(Color::Named(NamedColor::Cyan)),
        2 => Style::new().bold(true).fg(Color::Named(NamedColor::Green)),
        3 => Style::new().bold(true).fg(Color::Named(NamedColor::Yellow)),
        _ => Style::new().bold(true),
    }
}

/// Style for inline code.
fn inline_code_style() -> Style {
    Style::new().fg(Color::Named(NamedColor::Yellow))
}

/// Style for code blocks.
fn code_block_style() -> Style {
    Style::new().dim(true)
}

/// Mutable rendering state for word wrapping.
struct WrapState<'a> {
    lines: &'a mut Vec<Vec<Segment>>,
    current_line: &'a mut Vec<Segment>,
    current_width: &'a mut usize,
    in_list_item: bool,
    list_depth: usize,
}

/// Word-wrap text and push segments into the output.
fn wrap_text_into(text: &str, style: &Style, width: usize, state: &mut WrapState<'_>) {
    for word in text.split_whitespace() {
        let word_w = UnicodeWidthStr::width(word);

        // If adding this word would overflow, wrap
        let space_needed = if state.current_line.is_empty() || *state.current_width == 0 {
            0
        } else {
            1
        };

        if *state.current_width + space_needed + word_w > width && !state.current_line.is_empty() {
            flush_line(state.lines, state.current_line, state.current_width);
            // Add indent for continuation lines in list items
            if state.in_list_item {
                let indent = "  ".repeat(state.list_depth.saturating_sub(1));
                let cont_indent = format!("{indent}  ");
                let indent_w = UnicodeWidthStr::width(cont_indent.as_str());
                state.current_line.push(Segment::new(cont_indent));
                *state.current_width = indent_w;
            }
        }

        // Add space before word (unless at start of line)
        if !state.current_line.is_empty() && *state.current_width > 0 {
            state
                .current_line
                .push(Segment::styled(" ".to_string(), style.clone()));
            *state.current_width += 1;
        }

        state
            .current_line
            .push(Segment::styled(word.to_string(), style.clone()));
        *state.current_width += word_w;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_renders() {
        let mut r = MarkdownRenderer::new();
        r.push_str("Hello world");
        let lines = r.render_to_lines(80);
        assert!(!lines.is_empty());
        let text: String = lines[0].iter().map(|s| &*s.text).collect();
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn bold_italic_styled() {
        let mut r = MarkdownRenderer::new();
        r.push_str("**bold** and *italic*");
        let lines = r.render_to_lines(80);
        assert!(!lines.is_empty());
        // Check that bold segment exists
        let has_bold = lines[0].iter().any(|s| s.style.bold);
        assert!(has_bold);
        let has_italic = lines[0].iter().any(|s| s.style.italic);
        assert!(has_italic);
    }

    #[test]
    fn heading_styles() {
        let mut r = MarkdownRenderer::new();
        r.push_str("# Heading 1\n\n## Heading 2\n\n### Heading 3");
        let lines = r.render_to_lines(80);
        assert!(!lines.is_empty());
        // H1 should be bold and cyan
        let h1_line = &lines[0];
        assert!(h1_line.iter().any(|s| s.style.bold));
    }

    #[test]
    fn inline_code_styled() {
        let mut r = MarkdownRenderer::new();
        r.push_str("Use `code` here");
        let lines = r.render_to_lines(80);
        assert!(!lines.is_empty());
        let has_code = lines[0].iter().any(|s| s.text.contains("`code`"));
        assert!(has_code);
    }

    #[test]
    fn code_block_rendered() {
        let mut r = MarkdownRenderer::new();
        r.push_str("```rust\nfn main() {}\n```");
        let lines = r.render_to_lines(80);
        // Should contain the code line
        let all_text: String = lines
            .iter()
            .flat_map(|l| l.iter())
            .map(|s| &*s.text)
            .collect();
        assert!(all_text.contains("fn main()"));
    }

    #[test]
    fn list_items_with_markers() {
        let mut r = MarkdownRenderer::new();
        r.push_str("- item one\n- item two\n- item three");
        let lines = r.render_to_lines(80);
        assert!(lines.len() >= 3);
        let first_text: String = lines[0].iter().map(|s| &*s.text).collect();
        assert!(first_text.contains("-"));
        assert!(first_text.contains("item"));
    }

    #[test]
    fn incremental_push_str() {
        let mut r = MarkdownRenderer::new();
        r.push_str("Hello ");
        let lines1 = r.render_to_lines(80);
        r.push_str("world");
        let lines2 = r.render_to_lines(80);
        // After pushing "world", the output should include it
        let text: String = lines2
            .iter()
            .flat_map(|l| l.iter())
            .map(|s| &*s.text)
            .collect();
        assert!(text.contains("world"));
        // First render should only have "Hello"
        let text1: String = lines1
            .iter()
            .flat_map(|l| l.iter())
            .map(|s| &*s.text)
            .collect();
        assert!(text1.contains("Hello"));
    }

    #[test]
    fn width_wrapping() {
        let mut r = MarkdownRenderer::new();
        r.push_str("This is a long paragraph that should be wrapped to fit");
        let lines = r.render_to_lines(20);
        assert!(lines.len() > 1);
    }

    #[test]
    fn empty_input() {
        let r = MarkdownRenderer::new();
        let lines = r.render_to_lines(80);
        assert!(lines.is_empty());
    }

    #[test]
    fn clear_resets() {
        let mut r = MarkdownRenderer::new();
        r.push_str("some text");
        r.clear();
        let lines = r.render_to_lines(80);
        assert!(lines.is_empty());
    }

    #[test]
    fn mixed_content() {
        let mut r = MarkdownRenderer::new();
        r.push_str("# Title\n\nSome **bold** text.\n\n```\ncode\n```\n\n- list");
        let lines = r.render_to_lines(80);
        assert!(lines.len() >= 4);
    }
}
