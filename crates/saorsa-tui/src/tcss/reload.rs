//! Live stylesheet file watching and reloading.
//!
//! Provides [`StylesheetLoader`] for loading, parsing, and reloading
//! TCSS stylesheets from files, plus a file watcher for live updates.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use notify::{Event, EventKind, RecommendedWatcher, Watcher};

use crate::tcss::ast::Stylesheet;
use crate::tcss::error::TcssError;
use crate::tcss::parser::parse_stylesheet;
use crate::tcss::theme::{Theme, extract_themes};
use crate::tcss::variable::VariableMap;

/// A stylesheet loader that manages parsing and optional file watching.
#[derive(Clone, Debug)]
pub struct StylesheetLoader {
    /// Parsed stylesheet.
    stylesheet: Stylesheet,
    /// Extracted global variables.
    globals: VariableMap,
    /// Extracted themes.
    themes: Vec<Theme>,
    /// File path being watched (if loaded from file).
    path: Option<PathBuf>,
    /// Generation counter (incremented on each reload).
    generation: u64,
}

/// Events emitted by the stylesheet loader.
#[derive(Clone, Debug)]
pub enum StylesheetEvent {
    /// Stylesheet was reloaded successfully.
    Reloaded {
        /// The new generation number.
        generation: u64,
    },
    /// Stylesheet reload failed.
    Error(String),
}

impl StylesheetLoader {
    /// Create a new empty loader.
    pub fn new() -> Self {
        Self {
            stylesheet: Stylesheet::new(),
            globals: VariableMap::new(),
            themes: Vec::new(),
            path: None,
            generation: 0,
        }
    }

    /// Load and parse a stylesheet from a CSS string.
    pub fn load_string(css: &str) -> Result<Self, TcssError> {
        let stylesheet = parse_stylesheet(css)?;
        let (globals, themes) = extract_themes(&stylesheet);
        Ok(Self {
            stylesheet,
            globals,
            themes,
            path: None,
            generation: 1,
        })
    }

    /// Load and parse a stylesheet from a file.
    pub fn load_file(path: &Path) -> Result<Self, TcssError> {
        let css = std::fs::read_to_string(path).map_err(|e| TcssError::Parse(e.to_string()))?;
        let stylesheet = parse_stylesheet(&css)?;
        let (globals, themes) = extract_themes(&stylesheet);
        Ok(Self {
            stylesheet,
            globals,
            themes,
            path: Some(path.to_path_buf()),
            generation: 1,
        })
    }

    /// Reload the stylesheet from the associated file.
    ///
    /// Returns an error if no file path is set or if parsing fails.
    pub fn reload(&mut self) -> Result<StylesheetEvent, TcssError> {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| TcssError::Parse("no file path set for reload".into()))?
            .clone();

        let css = std::fs::read_to_string(&path).map_err(|e| TcssError::Parse(e.to_string()))?;
        let stylesheet = parse_stylesheet(&css)?;
        let (globals, themes) = extract_themes(&stylesheet);

        self.stylesheet = stylesheet;
        self.globals = globals;
        self.themes = themes;
        self.generation += 1;

        Ok(StylesheetEvent::Reloaded {
            generation: self.generation,
        })
    }

    /// Reload from a new CSS string (useful for testing or in-memory updates).
    pub fn reload_string(&mut self, css: &str) -> Result<StylesheetEvent, TcssError> {
        let stylesheet = parse_stylesheet(css)?;
        let (globals, themes) = extract_themes(&stylesheet);

        self.stylesheet = stylesheet;
        self.globals = globals;
        self.themes = themes;
        self.generation += 1;

        Ok(StylesheetEvent::Reloaded {
            generation: self.generation,
        })
    }

    /// Access the parsed stylesheet.
    pub fn stylesheet(&self) -> &Stylesheet {
        &self.stylesheet
    }

    /// Access the extracted global variables.
    pub fn globals(&self) -> &VariableMap {
        &self.globals
    }

    /// Access the extracted themes.
    pub fn themes(&self) -> &[Theme] {
        &self.themes
    }

    /// Get the current generation counter.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Get the file path (if loaded from a file).
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}

impl Default for StylesheetLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Start watching a stylesheet file for changes.
///
/// Returns a `(watcher, receiver)` pair. The watcher must be kept alive
/// for events to be delivered. Events arrive on the receiver channel
/// when the file is modified.
///
/// # Errors
///
/// Returns an error if the watcher cannot be created or the path
/// cannot be watched.
pub fn watch_stylesheet(
    path: &Path,
) -> Result<(RecommendedWatcher, mpsc::Receiver<StylesheetEvent>), TcssError> {
    let (tx, rx) = mpsc::channel();
    let watched_path = path.to_path_buf();

    let watcher_tx = tx.clone();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        match res {
            Ok(event) => {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    // Try to reload the file.
                    match std::fs::read_to_string(&watched_path) {
                        Ok(css) => match parse_stylesheet(&css) {
                            Ok(_) => {
                                let _ = watcher_tx.send(StylesheetEvent::Reloaded {
                                    generation: 0, // Caller tracks actual generation.
                                });
                            }
                            Err(e) => {
                                let _ = watcher_tx.send(StylesheetEvent::Error(e.to_string()));
                            }
                        },
                        Err(e) => {
                            let _ = watcher_tx.send(StylesheetEvent::Error(e.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                let _ = watcher_tx.send(StylesheetEvent::Error(e.to_string()));
            }
        }
    })
    .map_err(|e| TcssError::Parse(format!("failed to create watcher: {e}")))?;

    watcher
        .watch(path, notify::RecursiveMode::NonRecursive)
        .map_err(|e| TcssError::Parse(format!("failed to watch path: {e}")))?;

    Ok((watcher, rx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_loader() {
        let loader = StylesheetLoader::new();
        assert!(loader.stylesheet().is_empty());
        assert!(loader.globals().is_empty());
        assert!(loader.themes().is_empty());
        assert_eq!(loader.generation(), 0);
        assert!(loader.path().is_none());
    }

    #[test]
    fn load_from_string() {
        let css = "Label { color: red; }";
        let loader = StylesheetLoader::load_string(css);
        assert!(loader.is_ok());
        let loader = match loader {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        assert_eq!(loader.stylesheet().len(), 1);
        assert_eq!(loader.generation(), 1);
    }

    #[test]
    fn load_extracts_globals() {
        let css = ":root { $fg: white; $bg: #1e1e2e; }";
        let loader = StylesheetLoader::load_string(css);
        assert!(loader.is_ok());
        let loader = match loader {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        assert_eq!(loader.globals().len(), 2);
        assert!(loader.globals().contains("fg"));
    }

    #[test]
    fn load_extracts_themes() {
        let css = r#"
            .dark { $fg: white; }
            .light { $fg: #4c4f69; }
        "#;
        let loader = StylesheetLoader::load_string(css);
        assert!(loader.is_ok());
        let loader = match loader {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        assert_eq!(loader.themes().len(), 2);
    }

    #[test]
    fn generation_increments() {
        let css1 = "Label { color: red; }";
        let result = StylesheetLoader::load_string(css1);
        assert!(result.is_ok());
        let mut loader = match result {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        assert_eq!(loader.generation(), 1);

        let css2 = "Label { color: blue; }";
        let event = loader.reload_string(css2);
        assert!(event.is_ok());
        assert_eq!(loader.generation(), 2);
    }

    #[test]
    fn reload_from_string() {
        let css1 = "Label { color: red; }";
        let result = StylesheetLoader::load_string(css1);
        assert!(result.is_ok());
        let mut loader = match result {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };

        let css2 = r#"
            :root { $fg: white; }
            .dark { $bg: #1e1e2e; }
            Label { color: $fg; }
        "#;
        let event = loader.reload_string(css2);
        assert!(event.is_ok());
        assert_eq!(loader.globals().len(), 1);
        assert_eq!(loader.themes().len(), 1);
        assert_eq!(loader.generation(), 2);
    }

    #[test]
    fn loader_accessors() {
        let css = r#"
            :root { $fg: white; }
            .dark { $bg: black; }
            Label { color: $fg; }
        "#;
        let result = StylesheetLoader::load_string(css);
        assert!(result.is_ok());
        let loader = match result {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };

        assert!(!loader.stylesheet().is_empty());
        assert!(!loader.globals().is_empty());
        assert!(!loader.themes().is_empty());
        assert_eq!(loader.generation(), 1);
        assert!(loader.path().is_none());
    }

    #[test]
    fn load_file_not_found() {
        let result = StylesheetLoader::load_file(Path::new("/nonexistent/file.tcss"));
        assert!(result.is_err());
    }

    #[test]
    fn reload_without_path_errors() {
        let css = "Label { color: red; }";
        let result = StylesheetLoader::load_string(css);
        assert!(result.is_ok());
        let mut loader = match result {
            Ok(l) => l,
            Err(_) => unreachable!(),
        };
        // No file path set, reload should error.
        let result = loader.reload();
        assert!(result.is_err());
    }
}
