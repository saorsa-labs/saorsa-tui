//! CSS variable storage and scoped resolution.
//!
//! Variables use `$name` syntax and are resolved through a layered
//! environment: local → theme → global.

use std::collections::HashMap;

use crate::tcss::value::CssValue;

/// A set of CSS variable definitions.
#[derive(Clone, Debug, Default)]
pub struct VariableMap {
    vars: HashMap<String, CssValue>,
}

impl VariableMap {
    /// Create an empty variable map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a variable.
    pub fn set(&mut self, name: &str, value: CssValue) {
        self.vars.insert(name.into(), value);
    }

    /// Look up a variable by name.
    pub fn get(&self, name: &str) -> Option<&CssValue> {
        self.vars.get(name)
    }

    /// Remove a variable definition.
    pub fn remove(&mut self, name: &str) {
        self.vars.remove(name);
    }

    /// Check if a variable is defined.
    pub fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    /// Return the number of defined variables.
    pub fn len(&self) -> usize {
        self.vars.len()
    }

    /// Return whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty()
    }

    /// Iterate over all variable name-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &CssValue)> {
        self.vars.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Merge all variables from `other` into self.
    ///
    /// Variables in `other` override those in self on conflict.
    pub fn merge(&mut self, other: &VariableMap) {
        for (k, v) in &other.vars {
            self.vars.insert(k.clone(), v.clone());
        }
    }
}

/// A scoped variable environment with layered lookups.
///
/// Resolution order: local → theme → global.
#[derive(Clone, Debug)]
pub struct VariableEnvironment {
    /// Global variables (from :root rules).
    global: VariableMap,
    /// Active theme variables (override global).
    theme: VariableMap,
    /// Local overrides (per-widget inline).
    local: VariableMap,
}

impl Default for VariableEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl VariableEnvironment {
    /// Create an empty variable environment.
    pub fn new() -> Self {
        Self {
            global: VariableMap::new(),
            theme: VariableMap::new(),
            local: VariableMap::new(),
        }
    }

    /// Create an environment with global variables.
    pub fn with_global(global: VariableMap) -> Self {
        Self {
            global,
            theme: VariableMap::new(),
            local: VariableMap::new(),
        }
    }

    /// Resolve a variable by name.
    ///
    /// Lookup order: local → theme → global.
    pub fn resolve(&self, name: &str) -> Option<&CssValue> {
        self.local
            .get(name)
            .or_else(|| self.theme.get(name))
            .or_else(|| self.global.get(name))
    }

    /// Set a global variable.
    pub fn set_global(&mut self, name: &str, value: CssValue) {
        self.global.set(name, value);
    }

    /// Set a theme variable.
    pub fn set_theme(&mut self, name: &str, value: CssValue) {
        self.theme.set(name, value);
    }

    /// Set a local variable override.
    pub fn set_local(&mut self, name: &str, value: CssValue) {
        self.local.set(name, value);
    }

    /// Replace the entire theme layer.
    pub fn set_theme_layer(&mut self, theme: VariableMap) {
        self.theme = theme;
    }

    /// Clear all local overrides.
    pub fn clear_local(&mut self) {
        self.local = VariableMap::new();
    }

    /// Access the global variable map.
    pub fn global(&self) -> &VariableMap {
        &self.global
    }

    /// Access the theme variable map.
    pub fn theme(&self) -> &VariableMap {
        &self.theme
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;
    use crate::color::NamedColor;

    #[test]
    fn empty_variable_map() {
        let map = VariableMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn set_and_get() {
        let mut map = VariableMap::new();
        map.set("fg", CssValue::Color(Color::Named(NamedColor::White)));
        assert_eq!(
            map.get("fg"),
            Some(&CssValue::Color(Color::Named(NamedColor::White)))
        );
    }

    #[test]
    fn get_missing() {
        let map = VariableMap::new();
        assert!(map.get("nonexistent").is_none());
    }

    #[test]
    fn remove_variable() {
        let mut map = VariableMap::new();
        map.set("fg", CssValue::Keyword("red".into()));
        assert!(map.contains("fg"));
        map.remove("fg");
        assert!(!map.contains("fg"));
        assert!(map.get("fg").is_none());
    }

    #[test]
    fn contains_check() {
        let mut map = VariableMap::new();
        assert!(!map.contains("fg"));
        map.set("fg", CssValue::Keyword("white".into()));
        assert!(map.contains("fg"));
    }

    #[test]
    fn merge_maps() {
        let mut base = VariableMap::new();
        base.set("fg", CssValue::Keyword("white".into()));
        base.set("bg", CssValue::Keyword("black".into()));

        let mut other = VariableMap::new();
        other.set("fg", CssValue::Keyword("red".into()));
        other.set("accent", CssValue::Keyword("blue".into()));

        base.merge(&other);
        assert_eq!(base.len(), 3);
        // other's "fg" overrides base's "fg".
        assert_eq!(base.get("fg"), Some(&CssValue::Keyword("red".into())));
        assert_eq!(base.get("bg"), Some(&CssValue::Keyword("black".into())));
        assert_eq!(base.get("accent"), Some(&CssValue::Keyword("blue".into())));
    }

    #[test]
    fn environment_resolve_global() {
        let mut global = VariableMap::new();
        global.set("fg", CssValue::Color(Color::Named(NamedColor::White)));

        let env = VariableEnvironment::with_global(global);
        assert_eq!(
            env.resolve("fg"),
            Some(&CssValue::Color(Color::Named(NamedColor::White)))
        );
    }

    #[test]
    fn environment_resolve_theme_overrides_global() {
        let mut global = VariableMap::new();
        global.set("fg", CssValue::Color(Color::Named(NamedColor::White)));

        let mut env = VariableEnvironment::with_global(global);
        env.set_theme("fg", CssValue::Color(Color::Named(NamedColor::Red)));

        assert_eq!(
            env.resolve("fg"),
            Some(&CssValue::Color(Color::Named(NamedColor::Red)))
        );
    }

    #[test]
    fn environment_resolve_local_overrides_all() {
        let mut global = VariableMap::new();
        global.set("fg", CssValue::Color(Color::Named(NamedColor::White)));

        let mut env = VariableEnvironment::with_global(global);
        env.set_theme("fg", CssValue::Color(Color::Named(NamedColor::Red)));
        env.set_local("fg", CssValue::Color(Color::Named(NamedColor::Blue)));

        assert_eq!(
            env.resolve("fg"),
            Some(&CssValue::Color(Color::Named(NamedColor::Blue)))
        );
    }

    #[test]
    fn environment_set_theme_layer() {
        let mut env = VariableEnvironment::new();
        env.set_global("fg", CssValue::Keyword("white".into()));

        let mut theme = VariableMap::new();
        theme.set("fg", CssValue::Keyword("red".into()));
        theme.set("bg", CssValue::Keyword("black".into()));

        env.set_theme_layer(theme);
        assert_eq!(env.resolve("fg"), Some(&CssValue::Keyword("red".into())));
        assert_eq!(env.resolve("bg"), Some(&CssValue::Keyword("black".into())));
    }

    #[test]
    fn environment_clear_local() {
        let mut env = VariableEnvironment::new();
        env.set_global("fg", CssValue::Keyword("white".into()));
        env.set_local("fg", CssValue::Keyword("green".into()));

        assert_eq!(env.resolve("fg"), Some(&CssValue::Keyword("green".into())));
        env.clear_local();
        assert_eq!(env.resolve("fg"), Some(&CssValue::Keyword("white".into())));
    }

    #[test]
    fn variable_map_iteration() {
        let mut map = VariableMap::new();
        map.set("a", CssValue::Integer(1));
        map.set("b", CssValue::Integer(2));
        let pairs: Vec<_> = map.iter().collect();
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn environment_resolve_missing() {
        let env = VariableEnvironment::new();
        assert!(env.resolve("nonexistent").is_none());
    }

    #[test]
    fn variable_map_len_after_operations() {
        let mut map = VariableMap::new();
        map.set("a", CssValue::Integer(1));
        map.set("b", CssValue::Integer(2));
        assert_eq!(map.len(), 2);
        map.remove("a");
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        map.remove("b");
        assert!(map.is_empty());
    }

    #[test]
    fn environment_global_and_theme_accessors() {
        let mut env = VariableEnvironment::new();
        env.set_global("fg", CssValue::Keyword("white".into()));
        env.set_theme("bg", CssValue::Keyword("black".into()));

        assert_eq!(env.global().len(), 1);
        assert_eq!(env.theme().len(), 1);
        assert!(env.global().contains("fg"));
        assert!(env.theme().contains("bg"));
    }
}
