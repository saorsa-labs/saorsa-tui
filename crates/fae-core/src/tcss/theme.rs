//! Theme system for TCSS.
//!
//! Manages named themes (sets of variable overrides) with built-in
//! dark and light themes. Themes integrate with the [`VariableEnvironment`]
//! for scoped variable resolution.

use std::collections::HashMap;

use crate::Color;
use crate::tcss::error::TcssError;
use crate::tcss::value::CssValue;
use crate::tcss::variable::{VariableEnvironment, VariableMap};

/// A named theme — a set of variable overrides.
#[derive(Clone, Debug)]
pub struct Theme {
    /// Theme name (e.g., "dark", "light", "catppuccin").
    name: String,
    /// Variable overrides for this theme.
    variables: VariableMap,
}

impl Theme {
    /// Create a new empty theme.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            variables: VariableMap::new(),
        }
    }

    /// Create a theme with pre-defined variables.
    pub fn with_variables(name: &str, variables: VariableMap) -> Self {
        Self {
            name: name.into(),
            variables,
        }
    }

    /// Return the theme name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the theme's variables.
    pub fn variables(&self) -> &VariableMap {
        &self.variables
    }

    /// Set a variable in this theme.
    pub fn set_variable(&mut self, name: &str, value: CssValue) {
        self.variables.set(name, value);
    }
}

/// Manages multiple themes and the active theme.
#[derive(Clone, Debug, Default)]
pub struct ThemeManager {
    /// Available themes keyed by name.
    themes: HashMap<String, Theme>,
    /// Currently active theme name.
    active: Option<String>,
}

impl ThemeManager {
    /// Create an empty theme manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a theme.
    pub fn register(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    /// Set the active theme by name.
    ///
    /// Returns an error if the theme is not registered.
    pub fn set_active(&mut self, name: &str) -> Result<(), TcssError> {
        if self.themes.contains_key(name) {
            self.active = Some(name.into());
            Ok(())
        } else {
            Err(TcssError::InvalidValue {
                property: "theme".into(),
                value: format!("unknown theme: {name}"),
            })
        }
    }

    /// Get the active theme.
    pub fn active_theme(&self) -> Option<&Theme> {
        self.active.as_ref().and_then(|name| self.themes.get(name))
    }

    /// Get the active theme name.
    pub fn active_name(&self) -> Option<&str> {
        self.active.as_deref()
    }

    /// List all registered theme names.
    pub fn theme_names(&self) -> Vec<&str> {
        self.themes.keys().map(String::as_str).collect()
    }

    /// Check if a theme is registered.
    pub fn has_theme(&self, name: &str) -> bool {
        self.themes.contains_key(name)
    }

    /// Remove a theme by name. Returns true if it was present.
    pub fn remove(&mut self, name: &str) -> bool {
        let removed = self.themes.remove(name).is_some();
        if self.active.as_deref() == Some(name) {
            self.active = None;
        }
        removed
    }

    /// Build a [`VariableEnvironment`] from global variables and the active theme.
    pub fn build_environment(&self, global: &VariableMap) -> VariableEnvironment {
        let mut env = VariableEnvironment::with_global(global.clone());
        if let Some(theme) = self.active_theme() {
            env.set_theme_layer(theme.variables.clone());
        }
        env
    }
}

/// Create the built-in dark theme.
pub fn builtin_dark() -> Theme {
    let mut vars = VariableMap::new();
    vars.set(
        "fg",
        CssValue::Color(Color::Named(crate::color::NamedColor::White)),
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
    Theme::with_variables("dark", vars)
}

/// Create the built-in light theme.
pub fn builtin_light() -> Theme {
    let mut vars = VariableMap::new();
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
            r: 156,
            g: 160,
            b: 176,
        }),
    );
    Theme::with_variables("light", vars)
}

/// Extract themes from a parsed stylesheet.
///
/// Rules with a single class selector (`.dark`, `.light`) that contain
/// variable definitions are treated as theme definitions.
/// The `:root` rule provides global defaults (extracted separately via
/// [`crate::tcss::parser::extract_root_variables`]).
///
/// Returns (global_variables, themes).
pub fn extract_themes(stylesheet: &crate::tcss::ast::Stylesheet) -> (VariableMap, Vec<Theme>) {
    use crate::tcss::selector::{PseudoClass, SimpleSelector};

    let mut globals = VariableMap::new();
    let mut themes: Vec<Theme> = Vec::new();

    for rule in stylesheet.rules() {
        // Skip rules with no variable definitions.
        if rule.variables.is_empty() {
            continue;
        }

        let selectors = &rule.selectors.selectors;

        // Check for :root — contributes to global variables.
        let is_root = selectors.len() == 1
            && selectors[0].chain.is_empty()
            && selectors[0].head.components.len() == 1
            && matches!(
                &selectors[0].head.components[0],
                SimpleSelector::PseudoClass(PseudoClass::Root)
            );

        if is_root {
            for vardef in &rule.variables {
                globals.set(&vardef.name, vardef.value.clone());
            }
            continue;
        }

        // Check for single class selector — contributes to a theme.
        let is_theme_class = selectors.len() == 1
            && selectors[0].chain.is_empty()
            && selectors[0].head.components.len() == 1
            && matches!(&selectors[0].head.components[0], SimpleSelector::Class(_));

        if is_theme_class
            && let SimpleSelector::Class(class_name) = &selectors[0].head.components[0]
        {
            // Find or create the theme for this class name.
            let existing = themes.iter_mut().find(|t| t.name() == class_name.as_str());
            let theme = match existing {
                Some(t) => t,
                None => {
                    themes.push(Theme::new(class_name));
                    let idx = themes.len() - 1;
                    &mut themes[idx]
                }
            };
            for vardef in &rule.variables {
                theme.set_variable(&vardef.name, vardef.value.clone());
            }
        }
    }

    (globals, themes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::NamedColor;

    #[test]
    fn empty_theme() {
        let theme = Theme::new("test");
        assert_eq!(theme.name(), "test");
        assert!(theme.variables().is_empty());
    }

    #[test]
    fn theme_with_variables() {
        let mut vars = VariableMap::new();
        vars.set("fg", CssValue::Color(Color::Named(NamedColor::White)));
        let theme = Theme::with_variables("dark", vars);
        assert_eq!(theme.name(), "dark");
        assert_eq!(theme.variables().len(), 1);
    }

    #[test]
    fn theme_set_variable() {
        let mut theme = Theme::new("test");
        theme.set_variable("fg", CssValue::Color(Color::Named(NamedColor::Red)));
        assert_eq!(theme.variables().len(), 1);
        assert!(theme.variables().contains("fg"));
    }

    #[test]
    fn manager_register() {
        let mut mgr = ThemeManager::new();
        mgr.register(Theme::new("dark"));
        assert!(mgr.has_theme("dark"));
        assert!(!mgr.has_theme("light"));
    }

    #[test]
    fn manager_set_active() {
        let mut mgr = ThemeManager::new();
        mgr.register(Theme::new("dark"));
        assert!(mgr.set_active("dark").is_ok());
        assert_eq!(mgr.active_name(), Some("dark"));
    }

    #[test]
    fn manager_set_active_missing() {
        let mut mgr = ThemeManager::new();
        assert!(mgr.set_active("nonexistent").is_err());
    }

    #[test]
    fn manager_theme_names() {
        let mut mgr = ThemeManager::new();
        mgr.register(Theme::new("dark"));
        mgr.register(Theme::new("light"));
        let mut names = mgr.theme_names();
        names.sort();
        assert_eq!(names, vec!["dark", "light"]);
    }

    #[test]
    fn manager_build_environment() {
        let mut global = VariableMap::new();
        global.set("fg", CssValue::Color(Color::Named(NamedColor::White)));

        let mut dark = Theme::new("dark");
        dark.set_variable("fg", CssValue::Color(Color::Named(NamedColor::Red)));

        let mut mgr = ThemeManager::new();
        mgr.register(dark);
        let result = mgr.set_active("dark");
        assert!(result.is_ok());

        let env = mgr.build_environment(&global);
        // Theme overrides global.
        assert_eq!(
            env.resolve("fg"),
            Some(&CssValue::Color(Color::Named(NamedColor::Red)))
        );
    }

    #[test]
    fn builtin_dark_theme() {
        let dark = builtin_dark();
        assert_eq!(dark.name(), "dark");
        assert!(dark.variables().contains("fg"));
        assert!(dark.variables().contains("bg"));
        assert!(dark.variables().contains("primary"));
        assert!(dark.variables().contains("error"));
        assert!(dark.variables().contains("border"));
        assert!(dark.variables().len() >= 8);
    }

    #[test]
    fn builtin_light_theme() {
        let light = builtin_light();
        assert_eq!(light.name(), "light");
        assert!(light.variables().contains("fg"));
        assert!(light.variables().contains("bg"));
        assert!(light.variables().contains("primary"));
        assert!(light.variables().contains("error"));
        assert!(light.variables().contains("border"));
        assert!(light.variables().len() >= 8);
    }

    #[test]
    fn theme_switch() {
        let mut mgr = ThemeManager::new();
        mgr.register(builtin_dark());
        mgr.register(builtin_light());

        let result = mgr.set_active("dark");
        assert!(result.is_ok());
        assert_eq!(mgr.active_name(), Some("dark"));

        let result = mgr.set_active("light");
        assert!(result.is_ok());
        assert_eq!(mgr.active_name(), Some("light"));
    }

    #[test]
    fn manager_remove_theme() {
        let mut mgr = ThemeManager::new();
        mgr.register(Theme::new("dark"));
        let result = mgr.set_active("dark");
        assert!(result.is_ok());

        assert!(mgr.remove("dark"));
        assert!(!mgr.has_theme("dark"));
        // Active cleared when removing active theme.
        assert!(mgr.active_name().is_none());
    }

    // --- Theme extraction tests ---

    fn parse_sheet(css: &str) -> crate::tcss::ast::Stylesheet {
        let result = crate::tcss::parser::parse_stylesheet(css);
        assert!(result.is_ok(), "parse failed: {result:?}");
        match result {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn extract_no_themes() {
        let sheet = parse_sheet("Label { color: red; }");
        let (globals, themes) = extract_themes(&sheet);
        assert!(globals.is_empty());
        assert!(themes.is_empty());
    }

    #[test]
    fn extract_root_globals() {
        let sheet = parse_sheet(":root { $fg: white; $bg: #1e1e2e; }");
        let (globals, themes) = extract_themes(&sheet);
        assert_eq!(globals.len(), 2);
        assert!(globals.contains("fg"));
        assert!(globals.contains("bg"));
        assert!(themes.is_empty());
    }

    #[test]
    fn extract_dark_theme() {
        let sheet = parse_sheet(".dark { $fg: white; $bg: #1e1e2e; }");
        let (globals, themes) = extract_themes(&sheet);
        assert!(globals.is_empty());
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name(), "dark");
        assert_eq!(themes[0].variables().len(), 2);
    }

    #[test]
    fn extract_multiple_themes() {
        let css = r#"
            .dark { $fg: white; $bg: #1e1e2e; }
            .light { $fg: #4c4f69; $bg: #eff1f5; }
        "#;
        let sheet = parse_sheet(css);
        let (_, themes) = extract_themes(&sheet);
        assert_eq!(themes.len(), 2);
    }

    #[test]
    fn extract_ignores_property_rules() {
        let css = r#"
            .dark { $fg: white; }
            Label { color: red; }
        "#;
        let sheet = parse_sheet(css);
        let (_, themes) = extract_themes(&sheet);
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name(), "dark");
    }

    #[test]
    fn extract_full_themed_stylesheet() {
        let css = r#"
            :root { $fg: white; $bg: #1e1e2e; }
            .dark { $fg: white; $bg: #1e1e2e; }
            .light { $fg: #4c4f69; $bg: #eff1f5; }
            Label { color: $fg; }
            Container { background: $bg; }
        "#;
        let sheet = parse_sheet(css);
        let (globals, themes) = extract_themes(&sheet);
        assert_eq!(globals.len(), 2);
        assert_eq!(themes.len(), 2);
    }

    #[test]
    fn manager_build_environment_no_active() {
        let mut global = VariableMap::new();
        global.set("fg", CssValue::Color(Color::Named(NamedColor::White)));

        let mgr = ThemeManager::new();
        let env = mgr.build_environment(&global);
        // Only global layer.
        assert_eq!(
            env.resolve("fg"),
            Some(&CssValue::Color(Color::Named(NamedColor::White)))
        );
    }
}
