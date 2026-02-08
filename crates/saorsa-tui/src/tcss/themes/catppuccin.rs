//! Catppuccin color theme with all four flavors.
//!
//! Official palette: <https://github.com/catppuccin/catppuccin>
//!
//! Provides:
//! - Latte (light)
//! - Frappe (dark)
//! - Macchiato (dark)
//! - Mocha (dark)

use crate::Color;
use crate::tcss::theme::Theme;
use crate::tcss::value::CssValue;
use crate::tcss::variable::VariableMap;

/// Create the Catppuccin Mocha (dark) theme.
pub fn catppuccin_mocha() -> Theme {
    let mut vars = VariableMap::new();

    // Base colors
    vars.set(
        "base",
        CssValue::Color(Color::Rgb {
            r: 30,
            g: 30,
            b: 46,
        }),
    );
    vars.set(
        "mantle",
        CssValue::Color(Color::Rgb {
            r: 24,
            g: 24,
            b: 37,
        }),
    );
    vars.set(
        "crust",
        CssValue::Color(Color::Rgb {
            r: 17,
            g: 17,
            b: 27,
        }),
    );

    // Text colors
    vars.set(
        "text",
        CssValue::Color(Color::Rgb {
            r: 205,
            g: 214,
            b: 244,
        }),
    );
    vars.set(
        "subtext1",
        CssValue::Color(Color::Rgb {
            r: 186,
            g: 194,
            b: 222,
        }),
    );
    vars.set(
        "subtext0",
        CssValue::Color(Color::Rgb {
            r: 166,
            g: 173,
            b: 200,
        }),
    );

    // Overlay colors
    vars.set(
        "overlay2",
        CssValue::Color(Color::Rgb {
            r: 147,
            g: 153,
            b: 178,
        }),
    );
    vars.set(
        "overlay1",
        CssValue::Color(Color::Rgb {
            r: 127,
            g: 132,
            b: 156,
        }),
    );
    vars.set(
        "overlay0",
        CssValue::Color(Color::Rgb {
            r: 108,
            g: 112,
            b: 134,
        }),
    );

    // Surface colors
    vars.set(
        "surface2",
        CssValue::Color(Color::Rgb {
            r: 88,
            g: 91,
            b: 112,
        }),
    );
    vars.set(
        "surface1",
        CssValue::Color(Color::Rgb {
            r: 69,
            g: 71,
            b: 90,
        }),
    );
    vars.set(
        "surface0",
        CssValue::Color(Color::Rgb {
            r: 49,
            g: 50,
            b: 68,
        }),
    );

    // Accent colors
    vars.set(
        "rosewater",
        CssValue::Color(Color::Rgb {
            r: 245,
            g: 224,
            b: 220,
        }),
    );
    vars.set(
        "flamingo",
        CssValue::Color(Color::Rgb {
            r: 242,
            g: 205,
            b: 205,
        }),
    );
    vars.set(
        "pink",
        CssValue::Color(Color::Rgb {
            r: 245,
            g: 194,
            b: 231,
        }),
    );
    vars.set(
        "mauve",
        CssValue::Color(Color::Rgb {
            r: 203,
            g: 166,
            b: 247,
        }),
    );
    vars.set(
        "red",
        CssValue::Color(Color::Rgb {
            r: 243,
            g: 139,
            b: 168,
        }),
    );
    vars.set(
        "maroon",
        CssValue::Color(Color::Rgb {
            r: 235,
            g: 160,
            b: 172,
        }),
    );
    vars.set(
        "peach",
        CssValue::Color(Color::Rgb {
            r: 250,
            g: 179,
            b: 135,
        }),
    );
    vars.set(
        "yellow",
        CssValue::Color(Color::Rgb {
            r: 249,
            g: 226,
            b: 175,
        }),
    );
    vars.set(
        "green",
        CssValue::Color(Color::Rgb {
            r: 166,
            g: 227,
            b: 161,
        }),
    );
    vars.set(
        "teal",
        CssValue::Color(Color::Rgb {
            r: 148,
            g: 226,
            b: 213,
        }),
    );
    vars.set(
        "sky",
        CssValue::Color(Color::Rgb {
            r: 137,
            g: 220,
            b: 235,
        }),
    );
    vars.set(
        "sapphire",
        CssValue::Color(Color::Rgb {
            r: 116,
            g: 199,
            b: 236,
        }),
    );
    vars.set(
        "blue",
        CssValue::Color(Color::Rgb {
            r: 137,
            g: 180,
            b: 250,
        }),
    );
    vars.set(
        "lavender",
        CssValue::Color(Color::Rgb {
            r: 180,
            g: 190,
            b: 254,
        }),
    );

    // Common theme variables
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 205,
            g: 214,
            b: 244,
        }),
    );
    vars.set(
        "bg",
        CssValue::Color(Color::Rgb {
            r: 30,
            g: 30,
            b: 46,
        }),
    );
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb {
            r: 49,
            g: 50,
            b: 68,
        }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 137,
            g: 180,
            b: 250,
        }),
    );
    vars.set(
        "secondary",
        CssValue::Color(Color::Rgb {
            r: 166,
            g: 227,
            b: 161,
        }),
    );
    vars.set(
        "error",
        CssValue::Color(Color::Rgb {
            r: 243,
            g: 139,
            b: 168,
        }),
    );
    vars.set(
        "warning",
        CssValue::Color(Color::Rgb {
            r: 249,
            g: 226,
            b: 175,
        }),
    );
    vars.set(
        "border",
        CssValue::Color(Color::Rgb {
            r: 88,
            g: 91,
            b: 112,
        }),
    );

    Theme::with_variables("catppuccin-mocha", vars)
}

/// Create the Catppuccin Macchiato (dark) theme.
pub fn catppuccin_macchiato() -> Theme {
    let mut vars = VariableMap::new();

    // Base colors
    vars.set(
        "base",
        CssValue::Color(Color::Rgb {
            r: 36,
            g: 39,
            b: 58,
        }),
    );
    vars.set(
        "mantle",
        CssValue::Color(Color::Rgb {
            r: 30,
            g: 32,
            b: 48,
        }),
    );
    vars.set(
        "crust",
        CssValue::Color(Color::Rgb {
            r: 24,
            g: 25,
            b: 38,
        }),
    );

    // Text colors
    vars.set(
        "text",
        CssValue::Color(Color::Rgb {
            r: 202,
            g: 211,
            b: 245,
        }),
    );
    vars.set(
        "subtext1",
        CssValue::Color(Color::Rgb {
            r: 184,
            g: 192,
            b: 224,
        }),
    );
    vars.set(
        "subtext0",
        CssValue::Color(Color::Rgb {
            r: 165,
            g: 173,
            b: 203,
        }),
    );

    // Overlay colors
    vars.set(
        "overlay2",
        CssValue::Color(Color::Rgb {
            r: 147,
            g: 154,
            b: 183,
        }),
    );
    vars.set(
        "overlay1",
        CssValue::Color(Color::Rgb {
            r: 128,
            g: 135,
            b: 162,
        }),
    );
    vars.set(
        "overlay0",
        CssValue::Color(Color::Rgb {
            r: 110,
            g: 115,
            b: 141,
        }),
    );

    // Surface colors
    vars.set(
        "surface2",
        CssValue::Color(Color::Rgb {
            r: 91,
            g: 96,
            b: 120,
        }),
    );
    vars.set(
        "surface1",
        CssValue::Color(Color::Rgb {
            r: 73,
            g: 77,
            b: 100,
        }),
    );
    vars.set(
        "surface0",
        CssValue::Color(Color::Rgb {
            r: 54,
            g: 58,
            b: 79,
        }),
    );

    // Accent colors
    vars.set(
        "rosewater",
        CssValue::Color(Color::Rgb {
            r: 244,
            g: 219,
            b: 214,
        }),
    );
    vars.set(
        "flamingo",
        CssValue::Color(Color::Rgb {
            r: 240,
            g: 198,
            b: 198,
        }),
    );
    vars.set(
        "pink",
        CssValue::Color(Color::Rgb {
            r: 245,
            g: 189,
            b: 230,
        }),
    );
    vars.set(
        "mauve",
        CssValue::Color(Color::Rgb {
            r: 198,
            g: 160,
            b: 246,
        }),
    );
    vars.set(
        "red",
        CssValue::Color(Color::Rgb {
            r: 237,
            g: 135,
            b: 150,
        }),
    );
    vars.set(
        "maroon",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 153,
            b: 160,
        }),
    );
    vars.set(
        "peach",
        CssValue::Color(Color::Rgb {
            r: 245,
            g: 169,
            b: 127,
        }),
    );
    vars.set(
        "yellow",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 212,
            b: 159,
        }),
    );
    vars.set(
        "green",
        CssValue::Color(Color::Rgb {
            r: 166,
            g: 218,
            b: 149,
        }),
    );
    vars.set(
        "teal",
        CssValue::Color(Color::Rgb {
            r: 139,
            g: 213,
            b: 202,
        }),
    );
    vars.set(
        "sky",
        CssValue::Color(Color::Rgb {
            r: 145,
            g: 215,
            b: 227,
        }),
    );
    vars.set(
        "sapphire",
        CssValue::Color(Color::Rgb {
            r: 125,
            g: 196,
            b: 228,
        }),
    );
    vars.set(
        "blue",
        CssValue::Color(Color::Rgb {
            r: 138,
            g: 173,
            b: 244,
        }),
    );
    vars.set(
        "lavender",
        CssValue::Color(Color::Rgb {
            r: 183,
            g: 189,
            b: 248,
        }),
    );

    // Common theme variables
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 202,
            g: 211,
            b: 245,
        }),
    );
    vars.set(
        "bg",
        CssValue::Color(Color::Rgb {
            r: 36,
            g: 39,
            b: 58,
        }),
    );
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb {
            r: 54,
            g: 58,
            b: 79,
        }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 138,
            g: 173,
            b: 244,
        }),
    );
    vars.set(
        "secondary",
        CssValue::Color(Color::Rgb {
            r: 166,
            g: 218,
            b: 149,
        }),
    );
    vars.set(
        "error",
        CssValue::Color(Color::Rgb {
            r: 237,
            g: 135,
            b: 150,
        }),
    );
    vars.set(
        "warning",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 212,
            b: 159,
        }),
    );
    vars.set(
        "border",
        CssValue::Color(Color::Rgb {
            r: 91,
            g: 96,
            b: 120,
        }),
    );

    Theme::with_variables("catppuccin-macchiato", vars)
}

/// Create the Catppuccin Frappe (dark) theme.
pub fn catppuccin_frappe() -> Theme {
    let mut vars = VariableMap::new();

    // Base colors
    vars.set(
        "base",
        CssValue::Color(Color::Rgb {
            r: 48,
            g: 52,
            b: 70,
        }),
    );
    vars.set(
        "mantle",
        CssValue::Color(Color::Rgb {
            r: 41,
            g: 44,
            b: 60,
        }),
    );
    vars.set(
        "crust",
        CssValue::Color(Color::Rgb {
            r: 35,
            g: 38,
            b: 52,
        }),
    );

    // Text colors
    vars.set(
        "text",
        CssValue::Color(Color::Rgb {
            r: 198,
            g: 208,
            b: 245,
        }),
    );
    vars.set(
        "subtext1",
        CssValue::Color(Color::Rgb {
            r: 181,
            g: 191,
            b: 226,
        }),
    );
    vars.set(
        "subtext0",
        CssValue::Color(Color::Rgb {
            r: 165,
            g: 173,
            b: 206,
        }),
    );

    // Overlay colors
    vars.set(
        "overlay2",
        CssValue::Color(Color::Rgb {
            r: 148,
            g: 156,
            b: 187,
        }),
    );
    vars.set(
        "overlay1",
        CssValue::Color(Color::Rgb {
            r: 131,
            g: 139,
            b: 167,
        }),
    );
    vars.set(
        "overlay0",
        CssValue::Color(Color::Rgb {
            r: 115,
            g: 121,
            b: 148,
        }),
    );

    // Surface colors
    vars.set(
        "surface2",
        CssValue::Color(Color::Rgb {
            r: 98,
            g: 104,
            b: 128,
        }),
    );
    vars.set(
        "surface1",
        CssValue::Color(Color::Rgb {
            r: 81,
            g: 87,
            b: 109,
        }),
    );
    vars.set(
        "surface0",
        CssValue::Color(Color::Rgb {
            r: 65,
            g: 69,
            b: 89,
        }),
    );

    // Accent colors
    vars.set(
        "rosewater",
        CssValue::Color(Color::Rgb {
            r: 242,
            g: 213,
            b: 207,
        }),
    );
    vars.set(
        "flamingo",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 190,
            b: 190,
        }),
    );
    vars.set(
        "pink",
        CssValue::Color(Color::Rgb {
            r: 244,
            g: 184,
            b: 228,
        }),
    );
    vars.set(
        "mauve",
        CssValue::Color(Color::Rgb {
            r: 202,
            g: 158,
            b: 230,
        }),
    );
    vars.set(
        "red",
        CssValue::Color(Color::Rgb {
            r: 231,
            g: 130,
            b: 132,
        }),
    );
    vars.set(
        "maroon",
        CssValue::Color(Color::Rgb {
            r: 234,
            g: 153,
            b: 156,
        }),
    );
    vars.set(
        "peach",
        CssValue::Color(Color::Rgb {
            r: 239,
            g: 159,
            b: 118,
        }),
    );
    vars.set(
        "yellow",
        CssValue::Color(Color::Rgb {
            r: 229,
            g: 200,
            b: 144,
        }),
    );
    vars.set(
        "green",
        CssValue::Color(Color::Rgb {
            r: 166,
            g: 209,
            b: 137,
        }),
    );
    vars.set(
        "teal",
        CssValue::Color(Color::Rgb {
            r: 129,
            g: 200,
            b: 190,
        }),
    );
    vars.set(
        "sky",
        CssValue::Color(Color::Rgb {
            r: 153,
            g: 209,
            b: 219,
        }),
    );
    vars.set(
        "sapphire",
        CssValue::Color(Color::Rgb {
            r: 133,
            g: 193,
            b: 220,
        }),
    );
    vars.set(
        "blue",
        CssValue::Color(Color::Rgb {
            r: 140,
            g: 170,
            b: 238,
        }),
    );
    vars.set(
        "lavender",
        CssValue::Color(Color::Rgb {
            r: 186,
            g: 187,
            b: 241,
        }),
    );

    // Common theme variables
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 198,
            g: 208,
            b: 245,
        }),
    );
    vars.set(
        "bg",
        CssValue::Color(Color::Rgb {
            r: 48,
            g: 52,
            b: 70,
        }),
    );
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb {
            r: 65,
            g: 69,
            b: 89,
        }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 140,
            g: 170,
            b: 238,
        }),
    );
    vars.set(
        "secondary",
        CssValue::Color(Color::Rgb {
            r: 166,
            g: 209,
            b: 137,
        }),
    );
    vars.set(
        "error",
        CssValue::Color(Color::Rgb {
            r: 231,
            g: 130,
            b: 132,
        }),
    );
    vars.set(
        "warning",
        CssValue::Color(Color::Rgb {
            r: 229,
            g: 200,
            b: 144,
        }),
    );
    vars.set(
        "border",
        CssValue::Color(Color::Rgb {
            r: 98,
            g: 104,
            b: 128,
        }),
    );

    Theme::with_variables("catppuccin-frappe", vars)
}

/// Create the Catppuccin Latte (light) theme.
pub fn catppuccin_latte() -> Theme {
    let mut vars = VariableMap::new();

    // Base colors
    vars.set(
        "base",
        CssValue::Color(Color::Rgb {
            r: 239,
            g: 241,
            b: 245,
        }),
    );
    vars.set(
        "mantle",
        CssValue::Color(Color::Rgb {
            r: 230,
            g: 233,
            b: 239,
        }),
    );
    vars.set(
        "crust",
        CssValue::Color(Color::Rgb {
            r: 220,
            g: 224,
            b: 232,
        }),
    );

    // Text colors
    vars.set(
        "text",
        CssValue::Color(Color::Rgb {
            r: 76,
            g: 79,
            b: 105,
        }),
    );
    vars.set(
        "subtext1",
        CssValue::Color(Color::Rgb {
            r: 92,
            g: 95,
            b: 119,
        }),
    );
    vars.set(
        "subtext0",
        CssValue::Color(Color::Rgb {
            r: 108,
            g: 111,
            b: 133,
        }),
    );

    // Overlay colors
    vars.set(
        "overlay2",
        CssValue::Color(Color::Rgb {
            r: 124,
            g: 127,
            b: 147,
        }),
    );
    vars.set(
        "overlay1",
        CssValue::Color(Color::Rgb {
            r: 140,
            g: 143,
            b: 161,
        }),
    );
    vars.set(
        "overlay0",
        CssValue::Color(Color::Rgb {
            r: 156,
            g: 160,
            b: 176,
        }),
    );

    // Surface colors
    vars.set(
        "surface2",
        CssValue::Color(Color::Rgb {
            r: 172,
            g: 176,
            b: 190,
        }),
    );
    vars.set(
        "surface1",
        CssValue::Color(Color::Rgb {
            r: 188,
            g: 192,
            b: 204,
        }),
    );
    vars.set(
        "surface0",
        CssValue::Color(Color::Rgb {
            r: 204,
            g: 208,
            b: 218,
        }),
    );

    // Accent colors
    vars.set(
        "rosewater",
        CssValue::Color(Color::Rgb {
            r: 220,
            g: 138,
            b: 120,
        }),
    );
    vars.set(
        "flamingo",
        CssValue::Color(Color::Rgb {
            r: 221,
            g: 120,
            b: 120,
        }),
    );
    vars.set(
        "pink",
        CssValue::Color(Color::Rgb {
            r: 234,
            g: 118,
            b: 203,
        }),
    );
    vars.set(
        "mauve",
        CssValue::Color(Color::Rgb {
            r: 136,
            g: 57,
            b: 239,
        }),
    );
    vars.set(
        "red",
        CssValue::Color(Color::Rgb {
            r: 210,
            g: 15,
            b: 57,
        }),
    );
    vars.set(
        "maroon",
        CssValue::Color(Color::Rgb {
            r: 230,
            g: 69,
            b: 83,
        }),
    );
    vars.set(
        "peach",
        CssValue::Color(Color::Rgb {
            r: 254,
            g: 100,
            b: 11,
        }),
    );
    vars.set(
        "yellow",
        CssValue::Color(Color::Rgb {
            r: 223,
            g: 142,
            b: 29,
        }),
    );
    vars.set(
        "green",
        CssValue::Color(Color::Rgb {
            r: 64,
            g: 160,
            b: 43,
        }),
    );
    vars.set(
        "teal",
        CssValue::Color(Color::Rgb {
            r: 23,
            g: 146,
            b: 153,
        }),
    );
    vars.set(
        "sky",
        CssValue::Color(Color::Rgb {
            r: 4,
            g: 165,
            b: 229,
        }),
    );
    vars.set(
        "sapphire",
        CssValue::Color(Color::Rgb {
            r: 32,
            g: 159,
            b: 181,
        }),
    );
    vars.set(
        "blue",
        CssValue::Color(Color::Rgb {
            r: 30,
            g: 102,
            b: 245,
        }),
    );
    vars.set(
        "lavender",
        CssValue::Color(Color::Rgb {
            r: 114,
            g: 135,
            b: 253,
        }),
    );

    // Common theme variables
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 76,
            g: 79,
            b: 105,
        }),
    );
    vars.set(
        "bg",
        CssValue::Color(Color::Rgb {
            r: 239,
            g: 241,
            b: 245,
        }),
    );
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb {
            r: 204,
            g: 208,
            b: 218,
        }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 30,
            g: 102,
            b: 245,
        }),
    );
    vars.set(
        "secondary",
        CssValue::Color(Color::Rgb {
            r: 64,
            g: 160,
            b: 43,
        }),
    );
    vars.set(
        "error",
        CssValue::Color(Color::Rgb {
            r: 210,
            g: 15,
            b: 57,
        }),
    );
    vars.set(
        "warning",
        CssValue::Color(Color::Rgb {
            r: 223,
            g: 142,
            b: 29,
        }),
    );
    vars.set(
        "border",
        CssValue::Color(Color::Rgb {
            r: 172,
            g: 176,
            b: 190,
        }),
    );

    Theme::with_variables("catppuccin-latte", vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mocha_theme_name() {
        let theme = catppuccin_mocha();
        assert_eq!(theme.name(), "catppuccin-mocha");
    }

    #[test]
    fn mocha_has_all_base_colors() {
        let theme = catppuccin_mocha();
        let vars = theme.variables();
        assert!(vars.contains("base"));
        assert!(vars.contains("mantle"));
        assert!(vars.contains("crust"));
    }

    #[test]
    fn mocha_has_text_colors() {
        let theme = catppuccin_mocha();
        let vars = theme.variables();
        assert!(vars.contains("text"));
        assert!(vars.contains("subtext1"));
        assert!(vars.contains("subtext0"));
    }

    #[test]
    fn mocha_has_accent_colors() {
        let theme = catppuccin_mocha();
        let vars = theme.variables();
        assert!(vars.contains("red"));
        assert!(vars.contains("green"));
        assert!(vars.contains("blue"));
        assert!(vars.contains("yellow"));
        assert!(vars.contains("mauve"));
        assert!(vars.contains("teal"));
    }

    #[test]
    fn mocha_has_common_variables() {
        let theme = catppuccin_mocha();
        let vars = theme.variables();
        assert!(vars.contains("fg"));
        assert!(vars.contains("bg"));
        assert!(vars.contains("surface"));
        assert!(vars.contains("primary"));
        assert!(vars.contains("secondary"));
        assert!(vars.contains("error"));
        assert!(vars.contains("warning"));
        assert!(vars.contains("border"));
    }

    #[test]
    fn mocha_base_color_value() {
        let theme = catppuccin_mocha();
        let vars = theme.variables();
        let base = vars.get("base");
        assert!(base.is_some());
        match base {
            Some(CssValue::Color(Color::Rgb {
                r: 30,
                g: 30,
                b: 46,
            })) => (),
            _ => panic!("incorrect base color value"),
        }
    }

    #[test]
    fn macchiato_theme_name() {
        let theme = catppuccin_macchiato();
        assert_eq!(theme.name(), "catppuccin-macchiato");
    }

    #[test]
    fn macchiato_has_all_colors() {
        let theme = catppuccin_macchiato();
        let vars = theme.variables();
        assert!(vars.contains("base"));
        assert!(vars.contains("text"));
        assert!(vars.contains("blue"));
        assert!(vars.contains("fg"));
        assert!(vars.contains("bg"));
    }

    #[test]
    fn frappe_theme_name() {
        let theme = catppuccin_frappe();
        assert_eq!(theme.name(), "catppuccin-frappe");
    }

    #[test]
    fn frappe_has_all_colors() {
        let theme = catppuccin_frappe();
        let vars = theme.variables();
        assert!(vars.contains("base"));
        assert!(vars.contains("text"));
        assert!(vars.contains("blue"));
        assert!(vars.contains("fg"));
        assert!(vars.contains("bg"));
    }

    #[test]
    fn latte_theme_name() {
        let theme = catppuccin_latte();
        assert_eq!(theme.name(), "catppuccin-latte");
    }

    #[test]
    fn latte_has_all_colors() {
        let theme = catppuccin_latte();
        let vars = theme.variables();
        assert!(vars.contains("base"));
        assert!(vars.contains("text"));
        assert!(vars.contains("blue"));
        assert!(vars.contains("fg"));
        assert!(vars.contains("bg"));
    }

    #[test]
    fn latte_is_light_theme() {
        let theme = catppuccin_latte();
        let vars = theme.variables();
        // Latte should have dark text on light background
        let text = vars.get("text");
        let bg = vars.get("bg");
        assert!(text.is_some());
        assert!(bg.is_some());
        // Text should be darker (lower RGB values)
        match (text, bg) {
            (
                Some(CssValue::Color(Color::Rgb {
                    r: tr,
                    g: tg,
                    b: tb,
                })),
                Some(CssValue::Color(Color::Rgb {
                    r: br,
                    g: bg,
                    b: bb,
                })),
            ) => {
                assert!(
                    tr < br && tg < bg && tb < bb,
                    "light theme should have dark text on light background"
                );
            }
            _ => panic!("expected RGB colors for text and bg"),
        }
    }

    #[test]
    fn all_four_flavors_unique_names() {
        let mocha = catppuccin_mocha();
        let macchiato = catppuccin_macchiato();
        let frappe = catppuccin_frappe();
        let latte = catppuccin_latte();

        let names = [mocha.name(), macchiato.name(), frappe.name(), latte.name()];
        let unique: std::collections::HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), 4, "all four flavors should have unique names");
    }

    #[test]
    fn all_flavors_have_minimum_variables() {
        let themes = vec![
            catppuccin_mocha(),
            catppuccin_macchiato(),
            catppuccin_frappe(),
            catppuccin_latte(),
        ];

        for theme in themes {
            assert!(
                theme.variables().len() >= 30,
                "{} should have at least 30 variables",
                theme.name()
            );
        }
    }
}
