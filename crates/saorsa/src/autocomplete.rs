//! Autocomplete functionality for @files and /commands.

use std::path::PathBuf;

/// Autocomplete suggestion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    /// The text to insert.
    pub text: String,
    /// Description or additional info.
    pub description: Option<String>,
}

/// Autocomplete provider.
pub struct Autocomplete {
    /// Available file paths.
    file_paths: Vec<PathBuf>,
    /// Available commands.
    commands: Vec<String>,
}

impl Autocomplete {
    /// Create a new autocomplete provider.
    pub fn new() -> Self {
        Self {
            file_paths: Vec::new(),
            commands: vec![
                "/help".to_string(),
                "/model".to_string(),
                "/thinking".to_string(),
                "/compact".to_string(),
                "/clear".to_string(),
                "/hotkeys".to_string(),
                "/settings".to_string(),
                "/providers".to_string(),
                "/cost".to_string(),
                "/agents".to_string(),
                "/skills".to_string(),
                "/status".to_string(),
                "/tree".to_string(),
                "/bookmark".to_string(),
                "/export".to_string(),
                "/share".to_string(),
                "/fork".to_string(),
                "/login".to_string(),
                "/logout".to_string(),
            ],
        }
    }

    /// Set available file paths.
    pub fn set_file_paths(&mut self, paths: Vec<PathBuf>) {
        self.file_paths = paths;
    }

    /// Get suggestions for a prefix.
    pub fn suggest(&self, text: &str) -> Vec<Suggestion> {
        if let Some(stripped) = text.strip_prefix('@') {
            self.suggest_files(stripped)
        } else if text.starts_with('/') {
            self.suggest_commands(text)
        } else {
            Vec::new()
        }
    }

    /// Suggest file paths.
    fn suggest_files(&self, prefix: &str) -> Vec<Suggestion> {
        self.file_paths
            .iter()
            .filter_map(|path| {
                path.to_str().and_then(|s| {
                    if s.contains(prefix) {
                        Some(Suggestion {
                            text: format!("@{}", s),
                            description: None,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    /// Suggest commands.
    fn suggest_commands(&self, prefix: &str) -> Vec<Suggestion> {
        self.commands
            .iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .map(|cmd| Suggestion {
                text: cmd.clone(),
                description: None,
            })
            .collect()
    }
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_autocomplete() {
        let ac = Autocomplete::new();
        assert!(ac.file_paths.is_empty());
        assert!(!ac.commands.is_empty());
    }

    #[test]
    fn suggest_commands() {
        let ac = Autocomplete::new();
        let suggestions = ac.suggest("/mod");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.text == "/model"));
    }

    #[test]
    fn suggest_files() {
        let mut ac = Autocomplete::new();
        ac.set_file_paths(vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/lib.rs"),
        ]);
        let suggestions = ac.suggest("@src");
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn no_suggestions_for_plain_text() {
        let ac = Autocomplete::new();
        let suggestions = ac.suggest("hello");
        assert!(suggestions.is_empty());
    }
}
