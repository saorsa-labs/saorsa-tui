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
    /// Available commands with descriptions.
    commands: Vec<(&'static str, &'static str)>,
}

impl Autocomplete {
    /// Create a new autocomplete provider.
    pub fn new() -> Self {
        Self {
            file_paths: Vec::new(),
            commands: vec![
                ("/help", "Show available commands"),
                ("/model", "Show or switch AI model"),
                ("/thinking", "Set extended-thinking level"),
                ("/compact", "Toggle compact display mode"),
                ("/clear", "Clear conversation history"),
                ("/hotkeys", "Show keyboard shortcuts"),
                ("/settings", "Show current settings"),
                ("/providers", "List configured LLM providers"),
                ("/cost", "Show session cost breakdown"),
                ("/agents", "List available agent tools"),
                ("/skills", "List available skills"),
                ("/status", "Show session information"),
                ("/tree", "Show conversation tree"),
                ("/bookmark", "Manage bookmarks"),
                ("/export", "Export conversation"),
                ("/share", "Share conversation link"),
                ("/fork", "Fork conversation"),
                ("/login", "Configure API keys"),
                ("/logout", "Remove API keys"),
                ("/config", "View or change settings"),
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

    /// Suggest commands matching a prefix.
    ///
    /// When the input is exactly `/`, all commands are returned.
    fn suggest_commands(&self, prefix: &str) -> Vec<Suggestion> {
        self.commands
            .iter()
            .filter(|(name, _)| name.starts_with(prefix))
            .map(|(name, desc)| Suggestion {
                text: (*name).to_string(),
                description: Some((*desc).to_string()),
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
    fn suggest_commands_have_descriptions() {
        let ac = Autocomplete::new();
        let suggestions = ac.suggest("/mod");
        assert!(suggestions.iter().all(|s| s.description.is_some()));
    }

    #[test]
    fn suggest_all_commands_on_slash() {
        let ac = Autocomplete::new();
        let suggestions = ac.suggest("/");
        assert_eq!(suggestions.len(), ac.commands.len());
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
