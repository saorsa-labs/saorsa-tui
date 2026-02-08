//! Color types for terminal rendering.

use crate::error::{Result, SaorsaCoreError};

/// A terminal color.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Color {
    /// True color RGB.
    Rgb {
        /// Red component.
        r: u8,
        /// Green component.
        g: u8,
        /// Blue component.
        b: u8,
    },
    /// 256-color palette index.
    Indexed(u8),
    /// Named ANSI color.
    Named(NamedColor),
    /// Reset to terminal default.
    Reset,
}

/// The 16 standard ANSI colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NamedColor {
    /// Black (0).
    Black,
    /// Red (1).
    Red,
    /// Green (2).
    Green,
    /// Yellow (3).
    Yellow,
    /// Blue (4).
    Blue,
    /// Magenta (5).
    Magenta,
    /// Cyan (6).
    Cyan,
    /// White (7).
    White,
    /// Bright black / dark gray (8).
    BrightBlack,
    /// Bright red (9).
    BrightRed,
    /// Bright green (10).
    BrightGreen,
    /// Bright yellow (11).
    BrightYellow,
    /// Bright blue (12).
    BrightBlue,
    /// Bright magenta (13).
    BrightMagenta,
    /// Bright cyan (14).
    BrightCyan,
    /// Bright white (15).
    BrightWhite,
}

impl Color {
    /// Parse a hex color string like `"#rrggbb"` or `"#rgb"`.
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|e| SaorsaCoreError::Style(format!("invalid hex color: {e}")))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|e| SaorsaCoreError::Style(format!("invalid hex color: {e}")))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|e| SaorsaCoreError::Style(format!("invalid hex color: {e}")))?;
                Ok(Self::Rgb { r, g, b })
            }
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16)
                    .map_err(|e| SaorsaCoreError::Style(format!("invalid hex color: {e}")))?;
                let g = u8::from_str_radix(&hex[1..2], 16)
                    .map_err(|e| SaorsaCoreError::Style(format!("invalid hex color: {e}")))?;
                let b = u8::from_str_radix(&hex[2..3], 16)
                    .map_err(|e| SaorsaCoreError::Style(format!("invalid hex color: {e}")))?;
                Ok(Self::Rgb {
                    r: r * 17,
                    g: g * 17,
                    b: b * 17,
                })
            }
            _ => Err(SaorsaCoreError::Style(format!(
                "invalid hex color length: expected 3 or 6, got {}",
                hex.len()
            ))),
        }
    }

    /// Look up a color by CSS name.
    pub fn from_css_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "black" => Some(Self::Named(NamedColor::Black)),
            "red" => Some(Self::Named(NamedColor::Red)),
            "green" => Some(Self::Named(NamedColor::Green)),
            "yellow" => Some(Self::Named(NamedColor::Yellow)),
            "blue" => Some(Self::Named(NamedColor::Blue)),
            "magenta" => Some(Self::Named(NamedColor::Magenta)),
            "cyan" => Some(Self::Named(NamedColor::Cyan)),
            "white" => Some(Self::Named(NamedColor::White)),
            "gray" | "grey" => Some(Self::Named(NamedColor::BrightBlack)),
            _ => None,
        }
    }
}

impl From<Color> for crossterm::style::Color {
    fn from(color: Color) -> Self {
        match color {
            Color::Rgb { r, g, b } => crossterm::style::Color::Rgb { r, g, b },
            Color::Indexed(i) => crossterm::style::Color::AnsiValue(i),
            Color::Named(n) => match n {
                NamedColor::Black => crossterm::style::Color::Black,
                NamedColor::Red => crossterm::style::Color::DarkRed,
                NamedColor::Green => crossterm::style::Color::DarkGreen,
                NamedColor::Yellow => crossterm::style::Color::DarkYellow,
                NamedColor::Blue => crossterm::style::Color::DarkBlue,
                NamedColor::Magenta => crossterm::style::Color::DarkMagenta,
                NamedColor::Cyan => crossterm::style::Color::DarkCyan,
                NamedColor::White => crossterm::style::Color::Grey,
                NamedColor::BrightBlack => crossterm::style::Color::DarkGrey,
                NamedColor::BrightRed => crossterm::style::Color::Red,
                NamedColor::BrightGreen => crossterm::style::Color::Green,
                NamedColor::BrightYellow => crossterm::style::Color::Yellow,
                NamedColor::BrightBlue => crossterm::style::Color::Blue,
                NamedColor::BrightMagenta => crossterm::style::Color::Magenta,
                NamedColor::BrightCyan => crossterm::style::Color::Cyan,
                NamedColor::BrightWhite => crossterm::style::Color::White,
            },
            Color::Reset => crossterm::style::Color::Reset,
        }
    }
}

impl From<&Color> for crossterm::style::Color {
    fn from(color: &Color) -> Self {
        color.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_6_digit() {
        let c = Color::from_hex("#1e1e2e").ok();
        assert_eq!(
            c,
            Some(Color::Rgb {
                r: 30,
                g: 30,
                b: 46
            })
        );
    }

    #[test]
    fn hex_3_digit() {
        let c = Color::from_hex("#f0a").ok();
        assert_eq!(
            c,
            Some(Color::Rgb {
                r: 255,
                g: 0,
                b: 170
            })
        );
    }

    #[test]
    fn hex_no_hash() {
        let c = Color::from_hex("ff0000").ok();
        assert_eq!(c, Some(Color::Rgb { r: 255, g: 0, b: 0 }));
    }

    #[test]
    fn hex_invalid() {
        assert!(Color::from_hex("#gg0000").is_err());
        assert!(Color::from_hex("#1234").is_err());
        assert!(Color::from_hex("").is_err());
    }

    #[test]
    fn css_name_lookup() {
        assert_eq!(
            Color::from_css_name("red"),
            Some(Color::Named(NamedColor::Red))
        );
        assert_eq!(
            Color::from_css_name("Red"),
            Some(Color::Named(NamedColor::Red))
        );
        assert_eq!(Color::from_css_name("nonexistent"), None);
    }

    #[test]
    fn crossterm_conversion() {
        let ct: crossterm::style::Color = Color::Rgb { r: 1, g: 2, b: 3 }.into();
        assert_eq!(ct, crossterm::style::Color::Rgb { r: 1, g: 2, b: 3 });

        let ct: crossterm::style::Color = Color::Named(NamedColor::Red).into();
        assert_eq!(ct, crossterm::style::Color::DarkRed);

        let ct: crossterm::style::Color = Color::Indexed(42).into();
        assert_eq!(ct, crossterm::style::Color::AnsiValue(42));
    }
}
