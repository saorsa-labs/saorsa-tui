//! Text style type for terminal rendering.

use crate::color::Color;

/// Style attributes for a piece of text.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Style {
    /// Foreground color.
    pub fg: Option<Color>,
    /// Background color.
    pub bg: Option<Color>,
    /// Bold text.
    pub bold: bool,
    /// Italic text.
    pub italic: bool,
    /// Underlined text.
    pub underline: bool,
    /// Strikethrough text.
    pub strikethrough: bool,
    /// Dim/faint text.
    pub dim: bool,
    /// Reverse video.
    pub reverse: bool,
    /// OSC 8 hyperlink URL.
    pub link: Option<String>,
}

impl Style {
    /// Create an empty style with no attributes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the foreground color.
    #[must_use]
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set the background color.
    #[must_use]
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Set bold.
    #[must_use]
    pub fn bold(mut self, val: bool) -> Self {
        self.bold = val;
        self
    }

    /// Set italic.
    #[must_use]
    pub fn italic(mut self, val: bool) -> Self {
        self.italic = val;
        self
    }

    /// Set underline.
    #[must_use]
    pub fn underline(mut self, val: bool) -> Self {
        self.underline = val;
        self
    }

    /// Set strikethrough.
    #[must_use]
    pub fn strikethrough(mut self, val: bool) -> Self {
        self.strikethrough = val;
        self
    }

    /// Set dim.
    #[must_use]
    pub fn dim(mut self, val: bool) -> Self {
        self.dim = val;
        self
    }

    /// Set reverse video.
    #[must_use]
    pub fn reverse(mut self, val: bool) -> Self {
        self.reverse = val;
        self
    }

    /// Set hyperlink URL.
    #[must_use]
    pub fn link(mut self, url: impl Into<String>) -> Self {
        self.link = Some(url.into());
        self
    }

    /// Merge another style on top of this one. The `other` style's
    /// set values take priority.
    #[must_use]
    pub fn merge(&self, other: &Style) -> Style {
        Style {
            fg: other.fg.clone().or_else(|| self.fg.clone()),
            bg: other.bg.clone().or_else(|| self.bg.clone()),
            bold: if other.bold { true } else { self.bold },
            italic: if other.italic { true } else { self.italic },
            underline: if other.underline {
                true
            } else {
                self.underline
            },
            strikethrough: if other.strikethrough {
                true
            } else {
                self.strikethrough
            },
            dim: if other.dim { true } else { self.dim },
            reverse: if other.reverse { true } else { self.reverse },
            link: other.link.clone().or_else(|| self.link.clone()),
        }
    }

    /// Returns true if no attributes are set.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl From<&Style> for crossterm::style::ContentStyle {
    fn from(style: &Style) -> Self {
        use crossterm::style::{Attribute, ContentStyle};

        let mut cs = ContentStyle::new();
        if let Some(ref fg) = style.fg {
            cs.foreground_color = Some(fg.into());
        }
        if let Some(ref bg) = style.bg {
            cs.background_color = Some(bg.into());
        }
        if style.bold {
            cs.attributes.set(Attribute::Bold);
        }
        if style.italic {
            cs.attributes.set(Attribute::Italic);
        }
        if style.underline {
            cs.attributes.set(Attribute::Underlined);
        }
        if style.strikethrough {
            cs.attributes.set(Attribute::CrossedOut);
        }
        if style.dim {
            cs.attributes.set(Attribute::Dim);
        }
        if style.reverse {
            cs.attributes.set(Attribute::Reverse);
        }
        cs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::NamedColor;

    #[test]
    fn builder_pattern() {
        let s = Style::new()
            .fg(Color::Named(NamedColor::Red))
            .bold(true)
            .italic(true);
        assert_eq!(s.fg, Some(Color::Named(NamedColor::Red)));
        assert!(s.bold);
        assert!(s.italic);
        assert!(!s.underline);
    }

    #[test]
    fn default_is_empty() {
        assert!(Style::new().is_empty());
    }

    #[test]
    fn non_empty_style() {
        assert!(!Style::new().bold(true).is_empty());
    }

    #[test]
    fn merge_fg_override() {
        let base = Style::new().fg(Color::Named(NamedColor::Red));
        let over = Style::new().fg(Color::Named(NamedColor::Blue));
        let merged = base.merge(&over);
        assert_eq!(merged.fg, Some(Color::Named(NamedColor::Blue)));
    }

    #[test]
    fn merge_preserves_base() {
        let base = Style::new().fg(Color::Named(NamedColor::Red)).bold(true);
        let over = Style::new().italic(true);
        let merged = base.merge(&over);
        assert_eq!(merged.fg, Some(Color::Named(NamedColor::Red)));
        assert!(merged.bold);
        assert!(merged.italic);
    }

    #[test]
    fn crossterm_conversion() {
        let s = Style::new().fg(Color::Rgb { r: 1, g: 2, b: 3 }).bold(true);
        let cs: crossterm::style::ContentStyle = (&s).into();
        assert_eq!(
            cs.foreground_color,
            Some(crossterm::style::Color::Rgb { r: 1, g: 2, b: 3 })
        );
    }
}
