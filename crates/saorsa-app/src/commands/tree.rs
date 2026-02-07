//! /tree command implementation for displaying session hierarchy.

use saorsa_agent::{
    SaorsaAgentError, SessionId, SessionStorage, TreeRenderOptions, build_session_tree,
    find_in_tree, render_tree,
};
use std::str::FromStr;

/// The /tree command for displaying session hierarchy.
pub struct TreeCommand;

impl TreeCommand {
    /// Execute the tree command with optional arguments.
    ///
    /// - No args: Show full hierarchy
    /// - `<id>`: Show specific session subtree
    pub fn execute(args: &str) -> Result<String, SaorsaAgentError> {
        let storage = SessionStorage::new()?;
        let tree = build_session_tree(&storage)?;

        let options = TreeRenderOptions::default();

        // If an ID is provided, filter to that subtree
        if !args.trim().is_empty() {
            let session_id = SessionId::from_str(args.trim()).map_err(|_| {
                SaorsaAgentError::Session(format!("Invalid session ID: {}", args.trim()))
            })?;

            if let Some(subtree_root) = find_in_tree(&tree, session_id) {
                render_tree(&[subtree_root], &options)
            } else {
                Ok(format!("Session {} not found", session_id.prefix()))
            }
        } else {
            render_tree(&tree, &options)
        }
    }

    /// Execute with custom options (for filtering).
    pub fn execute_with_options(options: TreeRenderOptions) -> Result<String, SaorsaAgentError> {
        let storage = SessionStorage::new()?;
        let tree = build_session_tree(&storage)?;
        render_tree(&tree, &options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_command_empty_args() {
        // TreeCommand::execute requires access to filesystem storage
        // Integration test would require proper setup
        // Verify TreeCommand exists and has execute method
        let _ = TreeCommand;
    }

    #[test]
    fn test_tree_command_with_options() {
        // Test that TreeRenderOptions can be constructed
        let options = TreeRenderOptions::default();
        assert!(options.highlight_id.is_none());
        assert!(options.after_date.is_none());
        assert!(options.before_date.is_none());
        assert!(options.tags.is_empty());
    }
}
