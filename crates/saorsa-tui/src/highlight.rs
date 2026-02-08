//! Pluggable syntax highlighting trait and default implementations.
//!
//! Provides a [`Highlighter`] trait that can be implemented to add
//! syntax highlighting to [`crate::widget::TextArea`]. Includes a
//! [`NoHighlighter`] (no-op) and a [`SimpleKeywordHighlighter`] for
//! testing purposes.

use crate::style::Style;

/// A styled span within a single line of text.
///
/// Represents a range of characters `[start_col, end_col)` that
/// should be rendered with the given style.
#[derive(Clone, Debug, PartialEq)]
pub struct HighlightSpan {
    /// Start column (inclusive, zero-based character offset).
    pub start_col: usize,
    /// End column (exclusive, zero-based character offset).
    pub end_col: usize,
    /// The style to apply to this span.
    pub style: Style,
}

/// Trait for providing syntax highlighting to text buffers.
///
/// Implementors return styled spans for each line. The
/// [`NoHighlighter`] returns no spans (plain text). A tree-sitter
/// highlighter can be plugged in later without changing the widget API.
pub trait Highlighter {
    /// Return highlight spans for a given line.
    ///
    /// `line_idx` is the zero-based line index in the buffer.
    /// `text` is the content of that line (without trailing newline).
    fn highlight_line(&self, line_idx: usize, text: &str) -> Vec<HighlightSpan>;

    /// Notification that a line has been edited.
    ///
    /// Incremental parsers (e.g. tree-sitter) can use this to
    /// invalidate cached parse results for the affected region.
    fn on_edit(&mut self, line_idx: usize);
}

/// A no-op highlighter that returns no spans for any line.
///
/// This is the default highlighter used when no syntax highlighting
/// is configured.
#[derive(Clone, Debug, Default)]
pub struct NoHighlighter;

impl Highlighter for NoHighlighter {
    fn highlight_line(&self, _line_idx: usize, _text: &str) -> Vec<HighlightSpan> {
        Vec::new()
    }

    fn on_edit(&mut self, _line_idx: usize) {}
}

/// A simple keyword-based highlighter for testing.
///
/// Highlights exact keyword matches in each line. Keywords are
/// matched as substrings — no word boundary detection is performed.
#[derive(Clone, Debug)]
pub struct SimpleKeywordHighlighter {
    keywords: Vec<(String, Style)>,
}

impl SimpleKeywordHighlighter {
    /// Create a new keyword highlighter with the given keyword-style pairs.
    pub fn new(keywords: Vec<(String, Style)>) -> Self {
        Self { keywords }
    }
}

impl Highlighter for SimpleKeywordHighlighter {
    fn highlight_line(&self, _line_idx: usize, text: &str) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();

        for (keyword, style) in &self.keywords {
            let mut search_start = 0;
            while let Some(byte_idx) = text[search_start..].find(keyword.as_str()) {
                let abs_byte_idx = search_start + byte_idx;
                // Convert byte indices to character indices
                let start_col = text[..abs_byte_idx].chars().count();
                let end_col = start_col + keyword.chars().count();
                spans.push(HighlightSpan {
                    start_col,
                    end_col,
                    style: style.clone(),
                });
                search_start = abs_byte_idx + keyword.len();
            }
        }

        // Sort by start position for consistent ordering
        spans.sort_by_key(|s| s.start_col);
        spans
    }

    fn on_edit(&mut self, _line_idx: usize) {
        // No caching to invalidate in this simple implementation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_highlighter_returns_empty() {
        let h = NoHighlighter;
        let spans = h.highlight_line(0, "hello world");
        assert!(spans.is_empty());
    }

    #[test]
    fn keyword_highlighter_finds_keyword() {
        let h = SimpleKeywordHighlighter::new(vec![("fn".to_string(), Style::new().bold(true))]);
        let spans = h.highlight_line(0, "fn main() {}");
        assert!(spans.len() == 1);
        assert!(spans[0].start_col == 0);
        assert!(spans[0].end_col == 2);
        assert!(spans[0].style.bold);
    }

    #[test]
    fn multiple_keywords_same_line() {
        let h = SimpleKeywordHighlighter::new(vec![
            ("let".to_string(), Style::new().bold(true)),
            ("mut".to_string(), Style::new().italic(true)),
        ]);
        let spans = h.highlight_line(0, "let mut x = 5;");
        assert!(spans.len() == 2);
        // "let" at col 0-3, "mut" at col 4-7
        assert!(spans[0].start_col == 0);
        assert!(spans[0].end_col == 3);
        assert!(spans[1].start_col == 4);
        assert!(spans[1].end_col == 7);
    }

    #[test]
    fn no_match_returns_empty() {
        let h = SimpleKeywordHighlighter::new(vec![("class".to_string(), Style::new().bold(true))]);
        let spans = h.highlight_line(0, "fn main() {}");
        assert!(spans.is_empty());
    }

    #[test]
    fn partial_match_not_highlighted() {
        // "fn" is NOT a substring of "function" (f-u-n vs f-n)
        let h = SimpleKeywordHighlighter::new(vec![("fn".to_string(), Style::new().bold(true))]);
        let spans = h.highlight_line(0, "function");
        assert!(spans.is_empty());
    }

    #[test]
    fn unicode_keyword_matching() {
        let h = SimpleKeywordHighlighter::new(vec![("日本".to_string(), Style::new().bold(true))]);
        let spans = h.highlight_line(0, "hello 日本語 world");
        assert!(spans.len() == 1);
        assert!(spans[0].start_col == 6);
        assert!(spans[0].end_col == 8);
    }

    #[test]
    fn multiple_occurrences_of_keyword() {
        let h = SimpleKeywordHighlighter::new(vec![("ab".to_string(), Style::new().bold(true))]);
        let spans = h.highlight_line(0, "ab cd ab");
        assert!(spans.len() == 2);
        assert!(spans[0].start_col == 0);
        assert!(spans[1].start_col == 6);
    }

    #[test]
    fn on_edit_no_panic() {
        let mut h = NoHighlighter;
        h.on_edit(0);
        let mut kh =
            SimpleKeywordHighlighter::new(vec![("x".to_string(), Style::new().bold(true))]);
        kh.on_edit(5);
    }
}
