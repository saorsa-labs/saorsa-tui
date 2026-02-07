//! Prompt template system with variable substitution and conditionals.
//!
//! Provides built-in templates for common tasks and supports user-defined
//! templates from ~/.saorsa/templates/.

pub mod builtins;
pub mod engine;

pub use builtins::{get_builtin, list_builtins};
pub use engine::{TemplateContext, TemplateEngine, render_simple};

use crate::error::Result;
use std::path::PathBuf;

/// Discover user template files.
///
/// Searches ~/.saorsa/templates/*.md for user-defined templates.
pub fn discover_user_templates() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let template_dir = home.join(".saorsa/templates");
        if template_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&template_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    paths.push(path);
                }
            }
        }
    }

    paths
}

/// Load a user template from a file.
pub fn load_user_template(path: &PathBuf) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_user_templates_no_error() {
        // Should not panic even if directory doesn't exist
        let _templates = discover_user_templates();
    }

    #[test]
    fn test_render_simple_basic() {
        let template = "Hello {{name}}";
        let mut context = TemplateContext::new();
        context.insert("name".to_string(), "Test".to_string());

        let result = render_simple(template, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => assert_eq!(r, "Hello Test"),
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_template_engine_new() {
        let _engine = TemplateEngine::new();
    }
}
