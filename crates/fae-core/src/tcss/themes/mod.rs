//! Theme data structures for terminal UI styling.
//!
//! This module provides strongly-typed color themes with named semantic color slots,
//! theme metadata (name, author, variant), and a registry for managing multiple themes.

pub mod catppuccin;

use std::collections::HashMap;

use crate::Color;

pub use catppuccin::{catppuccin_frappe, catppuccin_latte, catppuccin_macchiato, catppuccin_mocha};

/// Semantic color slots for a theme.
///
/// Each field represents a specific UI purpose, allowing themes to be
/// swapped without changing widget code.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeColors {
    /// Primary background color.
    pub background: Color,
    /// Primary foreground (text) color.
    pub foreground: Color,
    /// Accent color for highlights and interactive elements.
    pub accent: Color,
    /// Error/danger color.
    pub error: Color,
    /// Warning/caution color.
    pub warning: Color,
    /// Success/confirmation color.
    pub success: Color,
    /// Surface color (panels, cards, containers).
    pub surface: Color,
    /// Border color.
    pub border: Color,
    /// Muted/disabled color.
    pub muted: Color,
    /// Secondary text color (less prominent than foreground).
    pub text_secondary: Color,
}

impl ThemeColors {
    /// Create a new ThemeColors with all slots specified.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        background: Color,
        foreground: Color,
        accent: Color,
        error: Color,
        warning: Color,
        success: Color,
        surface: Color,
        border: Color,
        muted: Color,
        text_secondary: Color,
    ) -> Self {
        Self {
            background,
            foreground,
            accent,
            error,
            warning,
            success,
            surface,
            border,
            muted,
            text_secondary,
        }
    }

    /// Create the default dark theme colors.
    pub fn default_dark() -> Self {
        use crate::color::NamedColor;
        Self {
            background: Color::Rgb {
                r: 30,
                g: 30,
                b: 46,
            },
            foreground: Color::Named(NamedColor::White),
            accent: Color::Rgb {
                r: 137,
                g: 180,
                b: 250,
            },
            error: Color::Rgb {
                r: 243,
                g: 139,
                b: 168,
            },
            warning: Color::Rgb {
                r: 249,
                g: 226,
                b: 175,
            },
            success: Color::Rgb {
                r: 166,
                g: 227,
                b: 161,
            },
            surface: Color::Rgb {
                r: 49,
                g: 50,
                b: 68,
            },
            border: Color::Rgb {
                r: 88,
                g: 91,
                b: 112,
            },
            muted: Color::Rgb {
                r: 108,
                g: 112,
                b: 134,
            },
            text_secondary: Color::Rgb {
                r: 186,
                g: 194,
                b: 222,
            },
        }
    }

    /// Create the default light theme colors.
    pub fn default_light() -> Self {
        use crate::color::NamedColor;
        Self {
            background: Color::Rgb {
                r: 239,
                g: 241,
                b: 245,
            },
            foreground: Color::Rgb {
                r: 76,
                g: 79,
                b: 105,
            },
            accent: Color::Rgb {
                r: 30,
                g: 102,
                b: 245,
            },
            error: Color::Rgb {
                r: 210,
                g: 15,
                b: 57,
            },
            warning: Color::Rgb {
                r: 223,
                g: 142,
                b: 29,
            },
            success: Color::Rgb {
                r: 64,
                g: 160,
                b: 43,
            },
            surface: Color::Rgb {
                r: 204,
                g: 208,
                b: 218,
            },
            border: Color::Rgb {
                r: 156,
                g: 160,
                b: 176,
            },
            muted: Color::Rgb {
                r: 140,
                g: 143,
                b: 161,
            },
            text_secondary: Color::Named(NamedColor::BrightBlack),
        }
    }
}

/// Theme variant: light or dark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThemeVariant {
    /// Light theme (dark text on light background).
    Light,
    /// Dark theme (light text on dark background).
    Dark,
}

/// A complete theme with colors and metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// Theme name.
    pub name: String,
    /// Theme author.
    pub author: String,
    /// Theme variant (light or dark).
    pub variant: ThemeVariant,
    /// Semantic color slots.
    pub colors: ThemeColors,
}

impl Theme {
    /// Create a new theme.
    pub fn new(name: String, author: String, variant: ThemeVariant, colors: ThemeColors) -> Self {
        Self {
            name,
            author,
            variant,
            colors,
        }
    }

    /// Create the built-in default dark theme.
    pub fn default_dark() -> Self {
        Self {
            name: "Default Dark".to_string(),
            author: "Fae".to_string(),
            variant: ThemeVariant::Dark,
            colors: ThemeColors::default_dark(),
        }
    }

    /// Create the built-in default light theme.
    pub fn default_light() -> Self {
        Self {
            name: "Default Light".to_string(),
            author: "Fae".to_string(),
            variant: ThemeVariant::Light,
            colors: ThemeColors::default_light(),
        }
    }
}

/// Registry for managing multiple themes.
#[derive(Clone, Debug)]
pub struct ThemeRegistry {
    themes: HashMap<String, Theme>,
}

impl ThemeRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            themes: HashMap::new(),
        }
    }

    /// Create a registry with the default dark theme pre-registered.
    pub fn with_default() -> Self {
        let mut registry = Self::new();
        registry.register_theme(Theme::default_dark());
        registry
    }

    /// Register a theme.
    ///
    /// If a theme with the same name already exists, it will be replaced.
    pub fn register_theme(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    /// Get a theme by name.
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// List all registered theme names.
    pub fn list_themes(&self) -> Vec<&str> {
        self.themes.keys().map(String::as_str).collect()
    }

    /// Check if a theme is registered.
    pub fn has_theme(&self, name: &str) -> bool {
        self.themes.contains_key(name)
    }

    /// Remove a theme by name. Returns `true` if the theme was present.
    pub fn remove_theme(&mut self, name: &str) -> bool {
        self.themes.remove(name).is_some()
    }

    /// Get the number of registered themes.
    pub fn len(&self) -> usize {
        self.themes.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.themes.is_empty()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::with_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::NamedColor;

    #[test]
    fn theme_colors_new() {
        let colors = ThemeColors::new(
            Color::Named(NamedColor::Black),
            Color::Named(NamedColor::White),
            Color::Named(NamedColor::Blue),
            Color::Named(NamedColor::Red),
            Color::Named(NamedColor::Yellow),
            Color::Named(NamedColor::Green),
            Color::Named(NamedColor::BrightBlack),
            Color::Named(NamedColor::BrightWhite),
            Color::Named(NamedColor::BrightBlack),
            Color::Named(NamedColor::BrightWhite),
        );

        assert_eq!(colors.background, Color::Named(NamedColor::Black));
        assert_eq!(colors.foreground, Color::Named(NamedColor::White));
        assert_eq!(colors.accent, Color::Named(NamedColor::Blue));
        assert_eq!(colors.error, Color::Named(NamedColor::Red));
    }

    #[test]
    fn theme_colors_default_dark() {
        let colors = ThemeColors::default_dark();
        assert_eq!(
            colors.background,
            Color::Rgb {
                r: 30,
                g: 30,
                b: 46
            }
        );
        assert_eq!(colors.foreground, Color::Named(NamedColor::White));
    }

    #[test]
    fn theme_colors_default_light() {
        let colors = ThemeColors::default_light();
        assert_eq!(
            colors.background,
            Color::Rgb {
                r: 239,
                g: 241,
                b: 245
            }
        );
        assert_eq!(
            colors.foreground,
            Color::Rgb {
                r: 76,
                g: 79,
                b: 105
            }
        );
    }

    #[test]
    fn theme_new() {
        let colors = ThemeColors::default_dark();
        let theme = Theme::new(
            "Test".to_string(),
            "Author".to_string(),
            ThemeVariant::Dark,
            colors.clone(),
        );

        assert_eq!(theme.name, "Test");
        assert_eq!(theme.author, "Author");
        assert_eq!(theme.variant, ThemeVariant::Dark);
        assert_eq!(theme.colors, colors);
    }

    #[test]
    fn theme_default_dark() {
        let theme = Theme::default_dark();
        assert_eq!(theme.name, "Default Dark");
        assert_eq!(theme.author, "Fae");
        assert_eq!(theme.variant, ThemeVariant::Dark);
    }

    #[test]
    fn theme_default_light() {
        let theme = Theme::default_light();
        assert_eq!(theme.name, "Default Light");
        assert_eq!(theme.author, "Fae");
        assert_eq!(theme.variant, ThemeVariant::Light);
    }

    #[test]
    fn registry_new_empty() {
        let registry = ThemeRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn registry_with_default() {
        let registry = ThemeRegistry::with_default();
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
        assert!(registry.has_theme("Default Dark"));
    }

    #[test]
    fn registry_register_and_get() {
        let mut registry = ThemeRegistry::new();
        let theme = Theme::default_dark();
        registry.register_theme(theme.clone());

        assert_eq!(registry.len(), 1);
        let retrieved = registry.get_theme("Default Dark");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.map(|t| t.name.as_str()), Some("Default Dark"));
    }

    #[test]
    fn registry_list_themes() {
        let mut registry = ThemeRegistry::new();
        registry.register_theme(Theme::default_dark());
        registry.register_theme(Theme::default_light());

        let mut names = registry.list_themes();
        names.sort();
        assert_eq!(names, vec!["Default Dark", "Default Light"]);
    }

    #[test]
    fn registry_has_theme() {
        let mut registry = ThemeRegistry::new();
        registry.register_theme(Theme::default_dark());

        assert!(registry.has_theme("Default Dark"));
        assert!(!registry.has_theme("Nonexistent"));
    }

    #[test]
    fn registry_remove_theme() {
        let mut registry = ThemeRegistry::new();
        registry.register_theme(Theme::default_dark());

        assert!(registry.has_theme("Default Dark"));
        assert!(registry.remove_theme("Default Dark"));
        assert!(!registry.has_theme("Default Dark"));
        assert!(!registry.remove_theme("Default Dark"));
    }

    #[test]
    fn registry_replace_theme() {
        let mut registry = ThemeRegistry::new();
        let theme1 = Theme::default_dark();
        registry.register_theme(theme1);

        let colors = ThemeColors::default_light();
        let theme2 = Theme::new(
            "Default Dark".to_string(),
            "New Author".to_string(),
            ThemeVariant::Light,
            colors,
        );
        registry.register_theme(theme2);

        assert_eq!(registry.len(), 1);
        let retrieved = registry.get_theme("Default Dark");
        assert_eq!(retrieved.map(|t| t.author.as_str()), Some("New Author"));
    }

    #[test]
    fn registry_default() {
        let registry = ThemeRegistry::default();
        assert!(!registry.is_empty());
        assert!(registry.has_theme("Default Dark"));
    }
}
