//! Dracula color theme.
//!
//! Official palette: <https://draculatheme.com/contribute>
//!
//! The Dracula theme is a dark theme with vibrant accent colors
//! designed to reduce eye strain while providing good contrast.

use crate::Color;
use crate::tcss::theme::Theme;
use crate::tcss::value::CssValue;
use crate::tcss::variable::VariableMap;

/// Create the Dracula dark theme.
pub fn dracula_dark() -> Theme {
    let mut vars = VariableMap::new();

    // Base colors
    vars.set(
        "background",
        CssValue::Color(Color::Rgb {
            r: 40,
            g: 42,
            b: 54,
        }),
    );
    vars.set(
        "current-line",
        CssValue::Color(Color::Rgb {
            r: 68,
            g: 71,
            b: 90,
        }),
    );
    vars.set(
        "selection",
        CssValue::Color(Color::Rgb {
            r: 68,
            g: 71,
            b: 90,
        }),
    );
    vars.set(
        "foreground",
        CssValue::Color(Color::Rgb {
            r: 248,
            g: 248,
            b: 242,
        }),
    );
    vars.set(
        "comment",
        CssValue::Color(Color::Rgb {
            r: 98,
            g: 114,
            b: 164,
        }),
    );

    // Accent colors
    vars.set(
        "cyan",
        CssValue::Color(Color::Rgb {
            r: 139,
            g: 233,
            b: 253,
        }),
    );
    vars.set(
        "green",
        CssValue::Color(Color::Rgb {
            r: 80,
            g: 250,
            b: 123,
        }),
    );
    vars.set(
        "orange",
        CssValue::Color(Color::Rgb {
            r: 255,
            g: 184,
            b: 108,
        }),
    );
    vars.set(
        "pink",
        CssValue::Color(Color::Rgb {
            r: 255,
            g: 121,
            b: 198,
        }),
    );
    vars.set(
        "purple",
        CssValue::Color(Color::Rgb {
            r: 189,
            g: 147,
            b: 249,
        }),
    );
    vars.set(
        "red",
        CssValue::Color(Color::Rgb {
            r: 255,
            g: 85,
            b: 85,
        }),
    );
    vars.set(
        "yellow",
        CssValue::Color(Color::Rgb {
            r: 241,
            g: 250,
            b: 140,
        }),
    );

    // Common theme variables
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 248,
            g: 248,
            b: 242,
        }),
    );
    vars.set(
        "bg",
        CssValue::Color(Color::Rgb {
            r: 40,
            g: 42,
            b: 54,
        }),
    );
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb {
            r: 68,
            g: 71,
            b: 90,
        }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 189,
            g: 147,
            b: 249,
        }),
    );
    vars.set(
        "secondary",
        CssValue::Color(Color::Rgb {
            r: 139,
            g: 233,
            b: 253,
        }),
    );
    vars.set(
        "error",
        CssValue::Color(Color::Rgb {
            r: 255,
            g: 85,
            b: 85,
        }),
    );
    vars.set(
        "warning",
        CssValue::Color(Color::Rgb {
            r: 255,
            g: 184,
            b: 108,
        }),
    );
    vars.set(
        "border",
        CssValue::Color(Color::Rgb {
            r: 98,
            g: 114,
            b: 164,
        }),
    );

    Theme::with_variables("dracula", vars)
}

/// Create the Dracula light theme.
///
/// Note: Dracula is traditionally a dark theme. This light variant
/// inverts the colors for light backgrounds.
pub fn dracula_light() -> Theme {
    let mut vars = VariableMap::new();

    // Base colors (inverted for light theme)
    vars.set(
        "background",
        CssValue::Color(Color::Rgb {
            r: 248,
            g: 248,
            b: 242,
        }),
    );
    vars.set(
        "current-line",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 238,
            b: 232,
        }),
    );
    vars.set(
        "selection",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 238,
            b: 232,
        }),
    );
    vars.set(
        "foreground",
        CssValue::Color(Color::Rgb {
            r: 40,
            g: 42,
            b: 54,
        }),
    );
    vars.set(
        "comment",
        CssValue::Color(Color::Rgb {
            r: 98,
            g: 114,
            b: 164,
        }),
    );

    // Accent colors (same as dark - these work on light backgrounds too)
    vars.set(
        "cyan",
        CssValue::Color(Color::Rgb {
            r: 42,
            g: 161,
            b: 152,
        }),
    );
    vars.set(
        "green",
        CssValue::Color(Color::Rgb {
            r: 40,
            g: 180,
            b: 80,
        }),
    );
    vars.set(
        "orange",
        CssValue::Color(Color::Rgb {
            r: 203,
            g: 75,
            b: 22,
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
        "purple",
        CssValue::Color(Color::Rgb {
            r: 108,
            g: 113,
            b: 196,
        }),
    );
    vars.set(
        "red",
        CssValue::Color(Color::Rgb {
            r: 220,
            g: 50,
            b: 47,
        }),
    );
    vars.set(
        "yellow",
        CssValue::Color(Color::Rgb {
            r: 181,
            g: 137,
            b: 0,
        }),
    );

    // Common theme variables
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 40,
            g: 42,
            b: 54,
        }),
    );
    vars.set(
        "bg",
        CssValue::Color(Color::Rgb {
            r: 248,
            g: 248,
            b: 242,
        }),
    );
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 238,
            b: 232,
        }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 108,
            g: 113,
            b: 196,
        }),
    );
    vars.set(
        "secondary",
        CssValue::Color(Color::Rgb {
            r: 42,
            g: 161,
            b: 152,
        }),
    );
    vars.set(
        "error",
        CssValue::Color(Color::Rgb {
            r: 220,
            g: 50,
            b: 47,
        }),
    );
    vars.set(
        "warning",
        CssValue::Color(Color::Rgb {
            r: 203,
            g: 75,
            b: 22,
        }),
    );
    vars.set(
        "border",
        CssValue::Color(Color::Rgb {
            r: 187,
            g: 184,
            b: 165,
        }),
    );

    Theme::with_variables("dracula-light", vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dracula_dark_theme_name() {
        let theme = dracula_dark();
        assert_eq!(theme.name(), "dracula");
    }

    #[test]
    fn dracula_light_theme_name() {
        let theme = dracula_light();
        assert_eq!(theme.name(), "dracula-light");
    }

    #[test]
    fn dracula_dark_has_base_colors() {
        let theme = dracula_dark();
        let vars = theme.variables();
        assert!(vars.contains("background"));
        assert!(vars.contains("foreground"));
        assert!(vars.contains("current-line"));
        assert!(vars.contains("selection"));
        assert!(vars.contains("comment"));
    }

    #[test]
    fn dracula_light_has_base_colors() {
        let theme = dracula_light();
        let vars = theme.variables();
        assert!(vars.contains("background"));
        assert!(vars.contains("foreground"));
        assert!(vars.contains("current-line"));
        assert!(vars.contains("selection"));
        assert!(vars.contains("comment"));
    }

    #[test]
    fn dracula_dark_has_accent_colors() {
        let theme = dracula_dark();
        let vars = theme.variables();
        assert!(vars.contains("cyan"));
        assert!(vars.contains("green"));
        assert!(vars.contains("orange"));
        assert!(vars.contains("pink"));
        assert!(vars.contains("purple"));
        assert!(vars.contains("red"));
        assert!(vars.contains("yellow"));
    }

    #[test]
    fn dracula_light_has_accent_colors() {
        let theme = dracula_light();
        let vars = theme.variables();
        assert!(vars.contains("cyan"));
        assert!(vars.contains("green"));
        assert!(vars.contains("orange"));
        assert!(vars.contains("pink"));
        assert!(vars.contains("purple"));
        assert!(vars.contains("red"));
        assert!(vars.contains("yellow"));
    }

    #[test]
    fn dracula_dark_has_common_variables() {
        let theme = dracula_dark();
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
    fn dracula_light_has_common_variables() {
        let theme = dracula_light();
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
    fn dracula_dark_background_color_value() {
        let theme = dracula_dark();
        let vars = theme.variables();
        let bg = vars.get("background");
        assert!(bg.is_some());
        match bg {
            Some(CssValue::Color(Color::Rgb {
                r: 40,
                g: 42,
                b: 54,
            })) => (),
            _ => panic!("incorrect background color value"),
        }
    }

    #[test]
    fn dracula_light_background_color_value() {
        let theme = dracula_light();
        let vars = theme.variables();
        let bg = vars.get("background");
        assert!(bg.is_some());
        match bg {
            Some(CssValue::Color(Color::Rgb {
                r: 248,
                g: 248,
                b: 242,
            })) => (),
            _ => panic!("incorrect background color value"),
        }
    }

    #[test]
    fn dracula_dark_purple_color_value() {
        let theme = dracula_dark();
        let vars = theme.variables();
        let purple = vars.get("purple");
        assert!(purple.is_some());
        match purple {
            Some(CssValue::Color(Color::Rgb {
                r: 189,
                g: 147,
                b: 249,
            })) => (),
            _ => panic!("incorrect purple color value"),
        }
    }

    #[test]
    fn both_variants_unique_names() {
        let dark = dracula_dark();
        let light = dracula_light();

        assert_ne!(
            dark.name(),
            light.name(),
            "dark and light variants should have unique names"
        );
    }

    #[test]
    fn both_variants_have_minimum_variables() {
        let themes = vec![dracula_dark(), dracula_light()];

        for theme in themes {
            assert!(
                theme.variables().len() >= 15,
                "{} should have at least 15 variables",
                theme.name()
            );
        }
    }
}
