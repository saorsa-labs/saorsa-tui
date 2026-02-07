//! Text preprocessing — tab expansion and control character filtering.

/// Configuration for text preprocessing.
#[derive(Debug, Clone)]
pub struct TextConfig {
    /// Tab stop width (default: 8).
    pub tab_width: u8,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self { tab_width: 8 }
    }
}

impl TextConfig {
    /// Create a text config with the given tab width.
    pub fn new(tab_width: u8) -> Self {
        Self { tab_width }
    }
}

/// Expand tabs to spaces according to tab stop positions.
///
/// Each tab character is replaced with enough spaces to reach the next
/// tab stop position. Tab stops are at every `tab_width` columns.
///
/// If `tab_width` is 0, tabs are simply removed.
pub fn expand_tabs(text: &str, tab_width: u8) -> String {
    if tab_width == 0 {
        return text.replace('\t', "");
    }

    let tw = tab_width as usize;
    let mut result = String::with_capacity(text.len());
    let mut column: usize = 0;

    for ch in text.chars() {
        if ch == '\t' {
            let spaces_needed = tw - (column % tw);
            for _ in 0..spaces_needed {
                result.push(' ');
            }
            column += spaces_needed;
        } else if ch == '\n' {
            result.push(ch);
            column = 0;
        } else {
            result.push(ch);
            column += 1;
        }
    }

    result
}

/// Remove or replace control characters.
///
/// Strips C0 control characters (except tab and newline) and C1 control characters.
/// Tab and newline are preserved since they have semantic meaning.
pub fn filter_control_chars(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for ch in text.chars() {
        // Preserve tab (0x09) and newline (0x0A)
        if ch == '\t' || ch == '\n' {
            result.push(ch);
            continue;
        }

        // Filter C0 control characters (0x00-0x1F) and DEL (0x7F)
        if ch.is_ascii_control() {
            continue;
        }

        // Filter C1 control characters (0x80-0x9F)
        let code = ch as u32;
        if (0x80..=0x9F).contains(&code) {
            continue;
        }

        result.push(ch);
    }

    result
}

/// Preprocess text: expand tabs then filter control characters.
///
/// This is a convenience function that first expands tabs to spaces
/// according to the given configuration, then strips control characters.
pub fn preprocess(text: &str, config: &TextConfig) -> String {
    let expanded = expand_tabs(text, config.tab_width);
    filter_control_chars(&expanded)
}

/// Truncate a string to a maximum byte length on a UTF-8 character boundary.
///
/// Returns a substring that is at most `max_bytes` bytes long, without
/// splitting any multi-byte characters. If the full string fits, it is
/// returned unchanged.
pub fn truncate_to_char_boundary(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    // Find the largest char boundary at or before max_bytes
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// Calculate the display width of text in terminal cells.
///
/// Uses the `unicode-width` crate to account for double-width characters
/// (CJK, emoji, etc.). Returns the width clamped to `u16::MAX`.
pub fn string_display_width(text: &str) -> u16 {
    use unicode_width::UnicodeWidthStr;
    let width = UnicodeWidthStr::width(text);
    if width > u16::MAX as usize {
        u16::MAX
    } else {
        width as u16
    }
}

/// Truncate a string to fit within a maximum display width.
///
/// Iterates over characters, accumulating their display widths until the
/// limit is reached. Returns a substring that fits within `max_width`
/// terminal cells without splitting any characters.
pub fn truncate_to_display_width(text: &str, max_width: usize) -> &str {
    use unicode_width::UnicodeWidthChar;
    let mut width = 0usize;
    for (byte_idx, ch) in text.char_indices() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            return &text[..byte_idx];
        }
        width += ch_width;
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_at_char_boundary_ascii() {
        let text = "Hello World";
        assert_eq!(truncate_to_char_boundary(text, 5), "Hello");
    }

    #[test]
    fn truncate_at_char_boundary_emoji() {
        // Emoji is 4 bytes; truncating at 7 bytes must not split the emoji
        let text = "Hello \u{1F600} World";
        let result = truncate_to_char_boundary(text, 7);
        // "Hello " is 6 bytes, emoji is 4 bytes, so 7 bytes truncates before emoji
        assert_eq!(result, "Hello ");
    }

    #[test]
    fn truncate_at_char_boundary_cjk() {
        // CJK chars are 3 bytes each
        let text = "\u{4F60}\u{597D}\u{4E16}\u{754C}"; // 你好世界
        let result = truncate_to_char_boundary(text, 7);
        // 3+3=6 fits, 3+3+3=9 doesn't fit in 7 bytes
        assert_eq!(result, "\u{4F60}\u{597D}");
    }

    #[test]
    fn truncate_at_char_boundary_empty() {
        assert_eq!(truncate_to_char_boundary("", 5), "");
    }

    #[test]
    fn truncate_at_char_boundary_zero_limit() {
        assert_eq!(truncate_to_char_boundary("Hello", 0), "");
    }

    #[test]
    fn truncate_at_char_boundary_larger_limit() {
        let text = "Hi";
        assert_eq!(truncate_to_char_boundary(text, 100), "Hi");
    }

    #[test]
    fn display_width_ascii() {
        assert_eq!(string_display_width("Hello"), 5);
    }

    #[test]
    fn display_width_emoji() {
        // Emoji typically has width 2
        assert_eq!(string_display_width("\u{1F600}"), 2);
    }

    #[test]
    fn display_width_cjk() {
        // Each CJK char has width 2
        assert_eq!(string_display_width("\u{4F60}\u{597D}"), 4);
    }

    #[test]
    fn display_width_empty() {
        assert_eq!(string_display_width(""), 0);
    }

    #[test]
    fn display_width_mixed() {
        // "Hi " = 3, emoji = 2 → 5
        assert_eq!(string_display_width("Hi \u{1F600}"), 5);
    }

    #[test]
    fn truncate_to_display_width_ascii() {
        assert_eq!(truncate_to_display_width("Hello World", 5), "Hello");
    }

    #[test]
    fn truncate_to_display_width_cjk() {
        // Each CJK is width 2; max_width 5 fits 2 chars (4 width), not 3 (6 width)
        let text = "\u{4F60}\u{597D}\u{4E16}"; // 你好世
        assert_eq!(truncate_to_display_width(text, 5), "\u{4F60}\u{597D}");
    }

    #[test]
    fn truncate_to_display_width_emoji() {
        // "Hi " is width 3, emoji is width 2 → total 5; max 4 stops before emoji
        assert_eq!(truncate_to_display_width("Hi \u{1F600}", 4), "Hi ");
    }

    #[test]
    fn expand_tabs_single_tab_at_position_zero() {
        // Tab at position 0, width 8 → 8 spaces
        let result = expand_tabs("\t", 8);
        assert_eq!(result, "        ");
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn expand_tabs_after_three_chars() {
        // "abc" (3 chars) then tab → 5 spaces to reach column 8
        let result = expand_tabs("abc\t", 8);
        assert_eq!(result, "abc     ");
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn expand_tabs_after_eight_chars() {
        // "abcdefgh" (8 chars) then tab → 8 spaces to reach column 16
        let result = expand_tabs("abcdefgh\t", 8);
        assert_eq!(result, "abcdefgh        ");
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn expand_tabs_no_tabs_unchanged() {
        let result = expand_tabs("hello world", 8);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn expand_tabs_custom_width_four() {
        // Tab at position 0, width 4 → 4 spaces
        let result = expand_tabs("\t", 4);
        assert_eq!(result, "    ");
        assert_eq!(result.len(), 4);

        // "ab" (2 chars) then tab → 2 spaces to reach column 4
        let result2 = expand_tabs("ab\t", 4);
        assert_eq!(result2, "ab  ");
        assert_eq!(result2.len(), 4);
    }

    #[test]
    fn filter_control_chars_removes_null() {
        let result = filter_control_chars("hello\x00world");
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn filter_control_chars_removes_bell() {
        let result = filter_control_chars("hello\x07world");
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn filter_control_chars_preserves_tab_and_newline() {
        let result = filter_control_chars("hello\tworld\n");
        assert_eq!(result, "hello\tworld\n");
    }

    #[test]
    fn filter_control_chars_clean_text_unchanged() {
        let result = filter_control_chars("Hello, World! 123");
        assert_eq!(result, "Hello, World! 123");
    }

    #[test]
    fn preprocess_combines_tab_expansion_and_filtering() {
        let config = TextConfig::new(4);
        // Tab should be expanded, bell should be removed
        let result = preprocess("a\tb\x07c", &config);
        // "a" at col 0, tab expands to 3 spaces (to col 4), then "b", then bell removed, then "c"
        assert_eq!(result, "a   bc");
    }

    #[test]
    fn empty_string_handling() {
        assert_eq!(expand_tabs("", 8), "");
        assert_eq!(filter_control_chars(""), "");
        let config = TextConfig::default();
        assert_eq!(preprocess("", &config), "");
    }

    #[test]
    fn expand_tabs_multiple_tabs() {
        // Two tabs in a row at position 0, width 4:
        // First tab: 4 spaces (col 0→4), second tab: 4 spaces (col 4→8)
        let result = expand_tabs("\t\t", 4);
        assert_eq!(result, "        ");
        assert_eq!(result.len(), 8);
    }

    #[test]
    fn filter_control_chars_removes_c1_range() {
        // U+0080 through U+009F are C1 control characters
        let text = format!("hello{}world", '\u{0085}'); // NEL
        let result = filter_control_chars(&text);
        assert_eq!(result, "helloworld");
    }

    #[test]
    fn expand_tabs_with_newline_resets_column() {
        // "abc\n\t" — after newline, column resets to 0, so tab expands to 4 spaces
        let result = expand_tabs("abc\n\t", 4);
        assert_eq!(result, "abc\n    ");
    }

    #[test]
    fn text_config_default_tab_width_eight() {
        let config = TextConfig::default();
        assert_eq!(config.tab_width, 8);
    }
}
