//! Nord color theme.
//!
//! Official palette: <https://www.nordtheme.com>
//!
//! Nord is an arctic, north-bluish color palette with a dark variant.

use crate::Color;
use crate::tcss::theme::Theme;
use crate::tcss::value::CssValue;
use crate::tcss::variable::VariableMap;

/// Create the Nord dark theme.
pub fn nord_dark() -> Theme {
    let mut vars = VariableMap::new();

    // Polar Night (dark backgrounds)
    vars.set(
        "nord0",
        CssValue::Color(Color::Rgb {
            r: 46,
            g: 52,
            b: 64,
        }),
    );
    vars.set(
        "nord1",
        CssValue::Color(Color::Rgb {
            r: 59,
            g: 66,
            b: 82,
        }),
    );
    vars.set(
        "nord2",
        CssValue::Color(Color::Rgb {
            r: 67,
            g: 76,
            b: 94,
        }),
    );
    vars.set(
        "nord3",
        CssValue::Color(Color::Rgb {
            r: 76,
            g: 86,
            b: 106,
        }),
    );

    // Snow Storm (light foregrounds)
    vars.set(
        "nord4",
        CssValue::Color(Color::Rgb {
            r: 216,
            g: 222,
            b: 233,
        }),
    );
    vars.set(
        "nord5",
        CssValue::Color(Color::Rgb {
            r: 229,
            g: 233,
            b: 240,
        }),
    );
    vars.set(
        "nord6",
        CssValue::Color(Color::Rgb {
            r: 236,
            g: 239,
            b: 244,
        }),
    );

    // Frost (blues/cyans)
    vars.set(
        "nord7",
        CssValue::Color(Color::Rgb {
            r: 143,
            g: 188,
            b: 187,
        }),
    );
    vars.set(
        "nord8",
        CssValue::Color(Color::Rgb {
            r: 136,
            g: 192,
            b: 208,
        }),
    );
    vars.set(
        "nord9",
        CssValue::Color(Color::Rgb {
            r: 129,
            g: 161,
            b: 193,
        }),
    );
    vars.set(
        "nord10",
        CssValue::Color(Color::Rgb {
            r: 94,
            g: 129,
            b: 172,
        }),
    );

    // Aurora (accent colors)
    vars.set(
        "nord11",
        CssValue::Color(Color::Rgb {
            r: 191,
            g: 97,
            b: 106,
        }),
    );
    vars.set(
        "nord12",
        CssValue::Color(Color::Rgb {
            r: 208,
            g: 135,
            b: 112,
        }),
    );
    vars.set(
        "nord13",
        CssValue::Color(Color::Rgb {
            r: 235,
            g: 203,
            b: 139,
        }),
    );
    vars.set(
        "nord14",
        CssValue::Color(Color::Rgb {
            r: 163,
            g: 190,
            b: 140,
        }),
    );
    vars.set(
        "nord15",
        CssValue::Color(Color::Rgb {
            r: 180,
            g: 142,
            b: 173,
        }),
    );

    // Common theme variables
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 216,
            g: 222,
            b: 233,
        }),
    );
    vars.set(
        "bg",
        CssValue::Color(Color::Rgb {
            r: 46,
            g: 52,
            b: 64,
        }),
    );
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb {
            r: 59,
            g: 66,
            b: 82,
        }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 136,
            g: 192,
            b: 208,
        }),
    );
    vars.set(
        "secondary",
        CssValue::Color(Color::Rgb {
            r: 163,
            g: 190,
            b: 140,
        }),
    );
    vars.set(
        "error",
        CssValue::Color(Color::Rgb {
            r: 191,
            g: 97,
            b: 106,
        }),
    );
    vars.set(
        "warning",
        CssValue::Color(Color::Rgb {
            r: 235,
            g: 203,
            b: 139,
        }),
    );
    vars.set(
        "border",
        CssValue::Color(Color::Rgb {
            r: 76,
            g: 86,
            b: 106,
        }),
    );

    Theme::with_variables("nord", vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nord_theme_name() {
        let theme = nord_dark();
        assert_eq!(theme.name(), "nord");
    }

    #[test]
    fn nord_has_all_palette_colors() {
        let theme = nord_dark();
        let vars = theme.variables();
        assert!(vars.contains("nord0"));
        assert!(vars.contains("nord1"));
        assert!(vars.contains("nord2"));
        assert!(vars.contains("nord3"));
        assert!(vars.contains("nord4"));
        assert!(vars.contains("nord5"));
        assert!(vars.contains("nord6"));
        assert!(vars.contains("nord7"));
        assert!(vars.contains("nord8"));
        assert!(vars.contains("nord9"));
        assert!(vars.contains("nord10"));
        assert!(vars.contains("nord11"));
        assert!(vars.contains("nord12"));
        assert!(vars.contains("nord13"));
        assert!(vars.contains("nord14"));
        assert!(vars.contains("nord15"));
    }

    #[test]
    fn nord_has_common_variables() {
        let theme = nord_dark();
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
    fn nord_bg_color_value() {
        let theme = nord_dark();
        let vars = theme.variables();
        let bg = vars.get("bg");
        assert!(bg.is_some());
        match bg {
            Some(CssValue::Color(Color::Rgb {
                r: 46,
                g: 52,
                b: 64,
            })) => (),
            _ => panic!("incorrect bg color value"),
        }
    }

    #[test]
    fn nord_fg_color_value() {
        let theme = nord_dark();
        let vars = theme.variables();
        let fg = vars.get("fg");
        assert!(fg.is_some());
        match fg {
            Some(CssValue::Color(Color::Rgb {
                r: 216,
                g: 222,
                b: 233,
            })) => (),
            _ => panic!("incorrect fg color value"),
        }
    }

    #[test]
    fn nord_frost_blue() {
        let theme = nord_dark();
        let vars = theme.variables();
        let nord8 = vars.get("nord8");
        assert!(nord8.is_some());
        match nord8 {
            Some(CssValue::Color(Color::Rgb {
                r: 136,
                g: 192,
                b: 208,
            })) => (),
            _ => panic!("incorrect nord8 (frost blue) color value"),
        }
    }

    #[test]
    fn nord_aurora_red() {
        let theme = nord_dark();
        let vars = theme.variables();
        let nord11 = vars.get("nord11");
        assert!(nord11.is_some());
        match nord11 {
            Some(CssValue::Color(Color::Rgb {
                r: 191,
                g: 97,
                b: 106,
            })) => (),
            _ => panic!("incorrect nord11 (aurora red) color value"),
        }
    }

    #[test]
    fn nord_has_minimum_variables() {
        let theme = nord_dark();
        assert!(
            theme.variables().len() >= 24,
            "nord should have at least 24 variables (16 palette + 8 common)"
        );
    }
}
