//! Solarized color themes (Light and Dark).
//!
//! Official palette: <https://ethanschoonover.com/solarized/>
//!
//! Solarized is a sixteen color palette (eight monotones, eight accent colors)
//! designed for use with both terminal and GUI applications.
//!
//! Provides:
//! - Solarized Dark
//! - Solarized Light

use crate::Color;
use crate::tcss::theme::Theme;
use crate::tcss::value::CssValue;
use crate::tcss::variable::VariableMap;

/// Create the Solarized Dark theme.
pub fn solarized_dark() -> Theme {
    let mut vars = VariableMap::new();

    // Base colors (dark)
    vars.set("base03", CssValue::Color(Color::Rgb { r: 0, g: 43, b: 54 }));
    vars.set("base02", CssValue::Color(Color::Rgb { r: 7, g: 54, b: 66 }));
    vars.set(
        "base01",
        CssValue::Color(Color::Rgb {
            r: 88,
            g: 110,
            b: 117,
        }),
    );
    vars.set(
        "base00",
        CssValue::Color(Color::Rgb {
            r: 101,
            g: 123,
            b: 131,
        }),
    );
    vars.set(
        "base0",
        CssValue::Color(Color::Rgb {
            r: 131,
            g: 148,
            b: 150,
        }),
    );
    vars.set(
        "base1",
        CssValue::Color(Color::Rgb {
            r: 147,
            g: 161,
            b: 161,
        }),
    );
    vars.set(
        "base2",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 232,
            b: 213,
        }),
    );
    vars.set(
        "base3",
        CssValue::Color(Color::Rgb {
            r: 253,
            g: 246,
            b: 227,
        }),
    );

    // Accent colors
    vars.set(
        "yellow",
        CssValue::Color(Color::Rgb {
            r: 181,
            g: 137,
            b: 0,
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
        "red",
        CssValue::Color(Color::Rgb {
            r: 220,
            g: 50,
            b: 47,
        }),
    );
    vars.set(
        "magenta",
        CssValue::Color(Color::Rgb {
            r: 211,
            g: 54,
            b: 130,
        }),
    );
    vars.set(
        "violet",
        CssValue::Color(Color::Rgb {
            r: 108,
            g: 113,
            b: 196,
        }),
    );
    vars.set(
        "blue",
        CssValue::Color(Color::Rgb {
            r: 38,
            g: 139,
            b: 210,
        }),
    );
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
            r: 133,
            g: 153,
            b: 0,
        }),
    );

    // Common theme variables (dark)
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 131,
            g: 148,
            b: 150,
        }),
    );
    vars.set("bg", CssValue::Color(Color::Rgb { r: 0, g: 43, b: 54 }));
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb { r: 7, g: 54, b: 66 }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 38,
            g: 139,
            b: 210,
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
            r: 88,
            g: 110,
            b: 117,
        }),
    );

    Theme::with_variables("solarized-dark", vars)
}

/// Create the Solarized Light theme.
pub fn solarized_light() -> Theme {
    let mut vars = VariableMap::new();

    // Base colors (light)
    vars.set("base03", CssValue::Color(Color::Rgb { r: 0, g: 43, b: 54 }));
    vars.set("base02", CssValue::Color(Color::Rgb { r: 7, g: 54, b: 66 }));
    vars.set(
        "base01",
        CssValue::Color(Color::Rgb {
            r: 88,
            g: 110,
            b: 117,
        }),
    );
    vars.set(
        "base00",
        CssValue::Color(Color::Rgb {
            r: 101,
            g: 123,
            b: 131,
        }),
    );
    vars.set(
        "base0",
        CssValue::Color(Color::Rgb {
            r: 131,
            g: 148,
            b: 150,
        }),
    );
    vars.set(
        "base1",
        CssValue::Color(Color::Rgb {
            r: 147,
            g: 161,
            b: 161,
        }),
    );
    vars.set(
        "base2",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 232,
            b: 213,
        }),
    );
    vars.set(
        "base3",
        CssValue::Color(Color::Rgb {
            r: 253,
            g: 246,
            b: 227,
        }),
    );

    // Accent colors (same as dark)
    vars.set(
        "yellow",
        CssValue::Color(Color::Rgb {
            r: 181,
            g: 137,
            b: 0,
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
        "red",
        CssValue::Color(Color::Rgb {
            r: 220,
            g: 50,
            b: 47,
        }),
    );
    vars.set(
        "magenta",
        CssValue::Color(Color::Rgb {
            r: 211,
            g: 54,
            b: 130,
        }),
    );
    vars.set(
        "violet",
        CssValue::Color(Color::Rgb {
            r: 108,
            g: 113,
            b: 196,
        }),
    );
    vars.set(
        "blue",
        CssValue::Color(Color::Rgb {
            r: 38,
            g: 139,
            b: 210,
        }),
    );
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
            r: 133,
            g: 153,
            b: 0,
        }),
    );

    // Common theme variables (light)
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 101,
            g: 123,
            b: 131,
        }),
    );
    vars.set(
        "bg",
        CssValue::Color(Color::Rgb {
            r: 253,
            g: 246,
            b: 227,
        }),
    );
    vars.set(
        "surface",
        CssValue::Color(Color::Rgb {
            r: 238,
            g: 232,
            b: 213,
        }),
    );
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb {
            r: 38,
            g: 139,
            b: 210,
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
            r: 147,
            g: 161,
            b: 161,
        }),
    );

    Theme::with_variables("solarized-light", vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solarized_dark_theme_name() {
        let theme = solarized_dark();
        assert_eq!(theme.name(), "solarized-dark");
    }

    #[test]
    fn solarized_light_theme_name() {
        let theme = solarized_light();
        assert_eq!(theme.name(), "solarized-light");
    }

    #[test]
    fn solarized_dark_has_base_colors() {
        let theme = solarized_dark();
        let vars = theme.variables();
        assert!(vars.contains("base03"));
        assert!(vars.contains("base02"));
        assert!(vars.contains("base01"));
        assert!(vars.contains("base00"));
        assert!(vars.contains("base0"));
        assert!(vars.contains("base1"));
        assert!(vars.contains("base2"));
        assert!(vars.contains("base3"));
    }

    #[test]
    fn solarized_light_has_base_colors() {
        let theme = solarized_light();
        let vars = theme.variables();
        assert!(vars.contains("base03"));
        assert!(vars.contains("base02"));
        assert!(vars.contains("base01"));
        assert!(vars.contains("base00"));
        assert!(vars.contains("base0"));
        assert!(vars.contains("base1"));
        assert!(vars.contains("base2"));
        assert!(vars.contains("base3"));
    }

    #[test]
    fn solarized_dark_has_accent_colors() {
        let theme = solarized_dark();
        let vars = theme.variables();
        assert!(vars.contains("yellow"));
        assert!(vars.contains("orange"));
        assert!(vars.contains("red"));
        assert!(vars.contains("magenta"));
        assert!(vars.contains("violet"));
        assert!(vars.contains("blue"));
        assert!(vars.contains("cyan"));
        assert!(vars.contains("green"));
    }

    #[test]
    fn solarized_light_has_accent_colors() {
        let theme = solarized_light();
        let vars = theme.variables();
        assert!(vars.contains("yellow"));
        assert!(vars.contains("orange"));
        assert!(vars.contains("red"));
        assert!(vars.contains("magenta"));
        assert!(vars.contains("violet"));
        assert!(vars.contains("blue"));
        assert!(vars.contains("cyan"));
        assert!(vars.contains("green"));
    }

    #[test]
    fn solarized_dark_has_common_variables() {
        let theme = solarized_dark();
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
    fn solarized_light_has_common_variables() {
        let theme = solarized_light();
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
    fn solarized_dark_background_color_value() {
        let theme = solarized_dark();
        let vars = theme.variables();
        let bg = vars.get("bg");
        assert!(bg.is_some());
        match bg {
            Some(CssValue::Color(Color::Rgb { r: 0, g: 43, b: 54 })) => (),
            _ => panic!("incorrect background color value"),
        }
    }

    #[test]
    fn solarized_light_background_color_value() {
        let theme = solarized_light();
        let vars = theme.variables();
        let bg = vars.get("bg");
        assert!(bg.is_some());
        match bg {
            Some(CssValue::Color(Color::Rgb {
                r: 253,
                g: 246,
                b: 227,
            })) => (),
            _ => panic!("incorrect background color value"),
        }
    }

    #[test]
    fn solarized_dark_blue_color_value() {
        let theme = solarized_dark();
        let vars = theme.variables();
        let blue = vars.get("blue");
        assert!(blue.is_some());
        match blue {
            Some(CssValue::Color(Color::Rgb {
                r: 38,
                g: 139,
                b: 210,
            })) => (),
            _ => panic!("incorrect blue color value"),
        }
    }

    #[test]
    fn solarized_light_is_light_theme() {
        let theme = solarized_light();
        let vars = theme.variables();
        let fg = vars.get("fg");
        let bg = vars.get("bg");
        assert!(fg.is_some());
        assert!(bg.is_some());
        // Light theme should have darker text on lighter background
        match (fg, bg) {
            (
                Some(CssValue::Color(Color::Rgb {
                    r: fr,
                    g: fg,
                    b: fb,
                })),
                Some(CssValue::Color(Color::Rgb {
                    r: br,
                    g: bg,
                    b: bb,
                })),
            ) => {
                assert!(
                    fr < br && fg < bg && fb < bb,
                    "light theme should have dark text on light background"
                );
            }
            _ => panic!("expected RGB colors for fg and bg"),
        }
    }

    #[test]
    fn solarized_dark_is_dark_theme() {
        let theme = solarized_dark();
        let vars = theme.variables();
        let fg = vars.get("fg");
        let bg = vars.get("bg");
        assert!(fg.is_some());
        assert!(bg.is_some());
        // Dark theme should have lighter text on darker background
        match (fg, bg) {
            (
                Some(CssValue::Color(Color::Rgb {
                    r: fr,
                    g: fg,
                    b: fb,
                })),
                Some(CssValue::Color(Color::Rgb {
                    r: br,
                    g: bg,
                    b: bb,
                })),
            ) => {
                assert!(
                    fr > br && fg > bg && fb > bb,
                    "dark theme should have light text on dark background"
                );
            }
            _ => panic!("expected RGB colors for fg and bg"),
        }
    }

    #[test]
    fn both_variants_unique_names() {
        let dark = solarized_dark();
        let light = solarized_light();

        assert_ne!(
            dark.name(),
            light.name(),
            "dark and light variants should have unique names"
        );
    }

    #[test]
    fn both_variants_have_minimum_variables() {
        let themes = vec![solarized_dark(), solarized_light()];

        for theme in themes {
            assert!(
                theme.variables().len() >= 20,
                "{} should have at least 20 variables",
                theme.name()
            );
        }
    }
}
