//! Integration tests for the theme system with CSS variables and styling.
//!
//! Tests theme registration, retrieval, switching, variable resolution,
//! and integration with the CSS variable system.

use saorsa_tui::Color;
use saorsa_tui::tcss::theme::{
    Theme, ThemeManager, builtin_dark, builtin_light, register_all_themes,
};
use saorsa_tui::tcss::value::CssValue;
use saorsa_tui::tcss::variable::VariableMap;

/// Test: Register and retrieve all built-in themes.
#[test]
fn register_and_retrieve_all_builtin_themes() {
    let manager = ThemeManager::with_defaults();

    // All 11 built-in themes should be registered
    assert!(manager.has_theme("dark"));
    assert!(manager.has_theme("light"));
    assert!(manager.has_theme("catppuccin-mocha"));
    assert!(manager.has_theme("catppuccin-macchiato"));
    assert!(manager.has_theme("catppuccin-frappe"));
    assert!(manager.has_theme("catppuccin-latte"));
    assert!(manager.has_theme("dracula"));
    assert!(manager.has_theme("dracula-light"));
    assert!(manager.has_theme("nord"));
    assert!(manager.has_theme("solarized-dark"));
    assert!(manager.has_theme("solarized-light"));

    // Should be able to retrieve each theme by setting it active
    assert!(manager.active_theme().is_none());

    // Verify theme names are listed
    let names = manager.theme_names();
    assert!(names.contains(&"dark"));
    assert!(names.contains(&"catppuccin-mocha"));
}

/// Test: Apply theme colors to CSS variables.
#[test]
fn apply_theme_colors_to_variables() {
    let mut manager = ThemeManager::new();
    manager.register(builtin_dark());

    let result = manager.set_active("dark");
    assert!(result.is_ok());

    let global = VariableMap::new();
    let env = manager.build_environment(&global);

    // Dark theme should have specific colors
    let fg = env.resolve("fg");
    assert!(fg.is_some());
    match fg {
        Some(CssValue::Color(Color::Named(_))) => (),
        _ => panic!("expected named color for fg"),
    }

    let bg = env.resolve("bg");
    assert!(bg.is_some());
    match bg {
        Some(CssValue::Color(Color::Rgb { .. })) => (),
        _ => panic!("expected RGB color for bg"),
    }

    let primary = env.resolve("primary");
    assert!(primary.is_some());
    match primary {
        Some(CssValue::Color(Color::Rgb { .. })) => (),
        _ => panic!("expected RGB color for primary"),
    }

    let error = env.resolve("error");
    assert!(error.is_some());
    match error {
        Some(CssValue::Color(Color::Rgb { .. })) => (),
        _ => panic!("expected RGB color for error"),
    }

    let border = env.resolve("border");
    assert!(border.is_some());
    match border {
        Some(CssValue::Color(Color::Rgb { .. })) => (),
        _ => panic!("expected RGB color for border"),
    }
}

/// Test: Theme switching at runtime.
#[test]
fn theme_switching_at_runtime() {
    let mut manager = ThemeManager::new();
    manager.register(builtin_dark());
    manager.register(builtin_light());

    // Start with no active theme
    assert!(manager.active_theme().is_none());

    // Switch to dark theme
    let result = manager.set_active("dark");
    assert!(result.is_ok());
    assert_eq!(manager.active_name(), Some("dark"));
    assert!(manager.active_theme().is_some());

    // Verify dark theme is active
    let dark_theme = manager.active_theme();
    assert_eq!(dark_theme.map(|t| t.name()), Some("dark"));

    // Switch to light theme
    let result = manager.set_active("light");
    assert!(result.is_ok());
    assert_eq!(manager.active_name(), Some("light"));

    // Verify light theme is active
    let light_theme = manager.active_theme();
    assert_eq!(light_theme.map(|t| t.name()), Some("light"));

    // Attempt to switch to non-existent theme
    let result = manager.set_active("nonexistent");
    assert!(result.is_err());
    // Active theme should remain "light"
    assert_eq!(manager.active_name(), Some("light"));
}

/// Test: Light vs dark variant detection.
#[test]
fn light_vs_dark_variant_detection() {
    let mut manager = ThemeManager::with_defaults();

    // Dark themes should have light text on dark backgrounds
    assert!(manager.has_theme("dark"));
    let result = manager.set_active("dark");
    assert!(result.is_ok());
    let dark_theme = manager.active_theme();
    assert!(dark_theme.is_some());
    match dark_theme.map(|t| t.variables()) {
        Some(dark_vars) => {
            assert!(dark_vars.contains("fg"));
            assert!(dark_vars.contains("bg"));
        }
        None => panic!("expected dark theme to have variables"),
    }

    // Light theme should have dark text on light background
    assert!(manager.has_theme("light"));
    let result = manager.set_active("light");
    assert!(result.is_ok());
    let light_theme = manager.active_theme();
    assert!(light_theme.is_some());
    match light_theme.map(|t| t.variables()) {
        Some(light_vars) => {
            assert!(light_vars.contains("fg"));
            assert!(light_vars.contains("bg"));
        }
        None => panic!("expected light theme to have variables"),
    }

    // Catppuccin Latte (light) vs Mocha (dark)
    let result = manager.set_active("catppuccin-latte");
    assert!(result.is_ok());
    let latte = manager.active_theme();
    assert!(latte.is_some());
    let latte_text = latte
        .and_then(|t| t.variables().get("text"))
        .and_then(|v| match v {
            CssValue::Color(Color::Rgb { r, g, b }) => Some((*r, *g, *b)),
            _ => None,
        });

    let latte_bg = latte
        .and_then(|t| t.variables().get("base"))
        .and_then(|v| match v {
            CssValue::Color(Color::Rgb { r, g, b }) => Some((*r, *g, *b)),
            _ => None,
        });

    // Latte: text should be darker than background
    match (latte_text, latte_bg) {
        (Some((tr, tg, tb)), Some((br, bg, bb))) => {
            assert!(
                tr < br && tg < bg && tb < bb,
                "light theme should have dark text on light background"
            );
        }
        _ => panic!("expected RGB colors for latte theme"),
    }

    // Mocha (dark) should have light text on dark background
    let result = manager.set_active("catppuccin-mocha");
    assert!(result.is_ok());
    let mocha = manager.active_theme();
    assert!(mocha.is_some());
    let mocha_text = mocha
        .and_then(|t| t.variables().get("text"))
        .and_then(|v| match v {
            CssValue::Color(Color::Rgb { r, g, b }) => Some((*r, *g, *b)),
            _ => None,
        });

    let mocha_bg = mocha
        .and_then(|t| t.variables().get("base"))
        .and_then(|v| match v {
            CssValue::Color(Color::Rgb { r, g, b }) => Some((*r, *g, *b)),
            _ => None,
        });

    // Mocha: text should be lighter than background
    match (mocha_text, mocha_bg) {
        (Some((tr, tg, tb)), Some((br, bg, bb))) => {
            assert!(
                tr > br && tg > bg && tb > bb,
                "dark theme should have light text on dark background"
            );
        }
        _ => panic!("expected RGB colors for mocha theme"),
    }
}

/// Test: Custom theme registration.
#[test]
fn custom_theme_registration() {
    let mut manager = ThemeManager::new();

    // Create a custom theme
    let mut vars = VariableMap::new();
    vars.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 255,
            g: 0,
            b: 255,
        }),
    );
    vars.set("bg", CssValue::Color(Color::Rgb { r: 0, g: 0, b: 0 }));
    vars.set(
        "primary",
        CssValue::Color(Color::Rgb { r: 0, g: 255, b: 0 }),
    );

    let custom = Theme::with_variables("custom-theme", vars);

    // Register the custom theme
    manager.register(custom);
    assert!(manager.has_theme("custom-theme"));

    // Activate and verify
    let result = manager.set_active("custom-theme");
    assert!(result.is_ok());
    assert_eq!(manager.active_name(), Some("custom-theme"));

    // Verify custom colors are available
    let global = VariableMap::new();
    let env = manager.build_environment(&global);

    let fg = env.resolve("fg");
    match fg {
        Some(CssValue::Color(Color::Rgb {
            r: 255,
            g: 0,
            b: 255,
        })) => (),
        _ => panic!("expected custom fg color"),
    }

    let primary = env.resolve("primary");
    match primary {
        Some(CssValue::Color(Color::Rgb { r: 0, g: 255, b: 0 })) => (),
        _ => panic!("expected custom primary color"),
    }
}

/// Test: Theme color slot access.
#[test]
fn theme_color_slot_access() {
    let mut manager = ThemeManager::with_defaults();

    // All themes should have common color slots
    let theme_names = vec![
        "dark",
        "light",
        "catppuccin-mocha",
        "dracula",
        "nord",
        "solarized-dark",
    ];

    for name in theme_names {
        let result = manager.set_active(name);
        assert!(result.is_ok(), "theme {name} should exist");
        let theme = manager.active_theme();
        assert!(theme.is_some(), "theme {name} should be active");
        match theme.map(|t| t.variables()) {
            Some(vars) => {
                // Common slots that all themes should have
                assert!(vars.contains("fg"), "{name} should have fg");
                assert!(vars.contains("bg"), "{name} should have bg");

                // Most themes have these common semantic slots
                let common_slots = ["primary", "error", "border"];
                for slot in &common_slots {
                    let has_slot = vars.contains(slot);
                    assert!(
                        has_slot,
                        "{name} should have {slot} (or equivalent semantic color)"
                    );
                }
            }
            None => panic!("{name} should have variables"),
        }
    }
}

/// Test: Theme manager list operations.
#[test]
fn theme_manager_list_operations() {
    let manager = ThemeManager::with_defaults();

    // List all theme names
    let names = manager.theme_names();
    assert!(names.len() >= 11);

    // Should contain specific themes
    assert!(names.contains(&"dark"));
    assert!(names.contains(&"light"));
    assert!(names.contains(&"catppuccin-mocha"));
    assert!(names.contains(&"dracula"));
    assert!(names.contains(&"nord"));
    assert!(names.contains(&"solarized-dark"));

    // Check has_theme consistency
    for name in &names {
        assert!(
            manager.has_theme(name),
            "has_theme should return true for {name}"
        );
    }
}

/// Test: Theme removal and active theme management.
#[test]
fn theme_removal_and_active_management() {
    let mut manager = ThemeManager::new();
    manager.register(builtin_dark());
    manager.register(builtin_light());

    // Set dark as active
    let result = manager.set_active("dark");
    assert!(result.is_ok());
    assert_eq!(manager.active_name(), Some("dark"));

    // Remove non-active theme
    assert!(manager.remove("light"));
    assert!(!manager.has_theme("light"));
    // Active should still be dark
    assert_eq!(manager.active_name(), Some("dark"));

    // Remove active theme
    assert!(manager.remove("dark"));
    assert!(!manager.has_theme("dark"));
    // Active should be cleared
    assert!(manager.active_name().is_none());
    assert!(manager.active_theme().is_none());

    // Remove non-existent theme
    assert!(!manager.remove("nonexistent"));
}

/// Test: Variable environment layering with themes.
#[test]
fn variable_environment_layering_with_themes() {
    let mut manager = ThemeManager::new();
    manager.register(builtin_dark());

    // Set up global variables
    let mut global = VariableMap::new();
    global.set(
        "fg",
        CssValue::Color(Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        }),
    );
    global.set("custom", CssValue::Integer(42));

    // Build environment without active theme
    let env_no_theme = manager.build_environment(&global);
    assert_eq!(env_no_theme.resolve("custom"), Some(&CssValue::Integer(42)));
    // fg comes from global
    match env_no_theme.resolve("fg") {
        Some(CssValue::Color(Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        })) => (),
        _ => panic!("expected global fg color"),
    }

    // Activate dark theme
    let result = manager.set_active("dark");
    assert!(result.is_ok());

    // Build environment with active theme
    let env_with_theme = manager.build_environment(&global);

    // Theme should override global fg
    let fg = env_with_theme.resolve("fg");
    assert!(fg.is_some());
    // Should NOT be the global fg value
    match fg {
        Some(CssValue::Color(Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        })) => panic!("expected theme fg to override global"),
        Some(CssValue::Color(_)) => (),
        _ => panic!("expected color for fg"),
    }

    // Global custom variable should still be accessible
    assert_eq!(
        env_with_theme.resolve("custom"),
        Some(&CssValue::Integer(42))
    );

    // Theme variables should be accessible
    assert!(env_with_theme.resolve("primary").is_some());
    assert!(env_with_theme.resolve("error").is_some());
}

/// Test: All registered themes have minimum variable set.
#[test]
fn all_themes_have_minimum_variables() {
    let mut manager = ThemeManager::with_defaults();

    // Collect theme names first to avoid borrow checker issues
    let theme_names: Vec<String> = manager
        .theme_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    for name in theme_names {
        let result = manager.set_active(&name);
        assert!(result.is_ok(), "theme {name} should exist");
        let theme = manager.active_theme();
        assert!(theme.is_some(), "theme {name} should be active");
        match theme.map(|t| t.variables()) {
            Some(vars) => {
                // All themes should define at least the semantic color variables
                assert!(
                    vars.len() >= 3,
                    "{name} should have at least 3 variables (found {})",
                    vars.len()
                );

                // All themes should have fg and bg
                assert!(
                    vars.contains("fg") || vars.contains("foreground"),
                    "{name} should have foreground color"
                );
                assert!(
                    vars.contains("bg") || vars.contains("background"),
                    "{name} should have background color"
                );
            }
            None => panic!("{name} should have variables"),
        }
    }
}

/// Test: Theme registration is idempotent.
#[test]
fn theme_registration_is_idempotent() {
    let mut manager = ThemeManager::new();

    // Register dark theme
    manager.register(builtin_dark());
    assert!(manager.has_theme("dark"));
    let count_after_first = manager.theme_names().len();
    assert_eq!(count_after_first, 1);

    // Register dark theme again
    manager.register(builtin_dark());
    assert!(manager.has_theme("dark"));
    let count_after_second = manager.theme_names().len();
    // Should still have only 1 theme
    assert_eq!(count_after_second, 1);

    // Registering with same name should replace
    let mut vars = VariableMap::new();
    vars.set("custom", CssValue::Integer(999));
    let custom_dark = Theme::with_variables("dark", vars);
    manager.register(custom_dark);

    assert_eq!(manager.theme_names().len(), 1);
    let result = manager.set_active("dark");
    assert!(result.is_ok());
    let theme = manager.active_theme();
    assert!(theme.is_some());
    match theme {
        Some(t) => assert!(t.variables().contains("custom")),
        None => panic!("expected custom dark theme to be active"),
    }
}

/// Test: Default theme manager has all built-in themes.
#[test]
fn default_theme_manager_has_all_builtins() {
    let manager = ThemeManager::with_defaults();

    // Should have exactly 11 themes
    let names = manager.theme_names();
    assert_eq!(names.len(), 11);

    // Built-in themes
    assert!(manager.has_theme("dark"));
    assert!(manager.has_theme("light"));

    // Catppuccin (4 flavors)
    assert!(manager.has_theme("catppuccin-mocha"));
    assert!(manager.has_theme("catppuccin-macchiato"));
    assert!(manager.has_theme("catppuccin-frappe"));
    assert!(manager.has_theme("catppuccin-latte"));

    // Dracula (2 variants)
    assert!(manager.has_theme("dracula"));
    assert!(manager.has_theme("dracula-light"));

    // Nord (1 variant)
    assert!(manager.has_theme("nord"));

    // Solarized (2 variants)
    assert!(manager.has_theme("solarized-dark"));
    assert!(manager.has_theme("solarized-light"));
}

/// Test: Register all themes function.
#[test]
fn register_all_themes_function() {
    let mut manager = ThemeManager::new();
    assert!(manager.theme_names().is_empty());

    register_all_themes(&mut manager);

    // Should have all 11 themes
    assert_eq!(manager.theme_names().len(), 11);
    assert!(manager.has_theme("dark"));
    assert!(manager.has_theme("catppuccin-mocha"));
    assert!(manager.has_theme("dracula"));
    assert!(manager.has_theme("nord"));
    assert!(manager.has_theme("solarized-dark"));
}
