//! Session tree visualization and hierarchy management.

use crate::FaeAgentError;
use crate::session::{SessionId, SessionMetadata, SessionNode, SessionStorage};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::str::FromStr;

/// A node in the session tree with metadata for rendering.
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Session ID
    pub id: SessionId,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Tree structure
    pub node: SessionNode,
    /// Child tree nodes
    pub children: Vec<TreeNode>,
    /// Message count for this session
    pub message_count: usize,
}

/// Options for tree rendering.
#[derive(Debug, Clone, Default)]
pub struct TreeRenderOptions {
    /// Highlight this session ID
    pub highlight_id: Option<SessionId>,
    /// Filter by minimum date
    pub after_date: Option<DateTime<Utc>>,
    /// Filter by maximum date
    pub before_date: Option<DateTime<Utc>>,
    /// Filter by tags
    pub tags: Vec<String>,
}

/// Build a tree of all sessions from storage.
pub fn build_session_tree(storage: &SessionStorage) -> Result<Vec<TreeNode>, FaeAgentError> {
    // Load all sessions
    let sessions = list_all_sessions_with_metadata(storage)?;

    if sessions.is_empty() {
        return Ok(Vec::new());
    }

    // Build a map of session ID -> (metadata, node, message_count)
    let mut session_map: HashMap<SessionId, (SessionMetadata, SessionNode, usize)> =
        HashMap::new();

    for (id, metadata) in sessions {
        let node = storage.load_tree(&id).unwrap_or_else(|_| SessionNode::new_root(id));

        let message_count = storage.load_messages(&id).map(|m| m.len()).unwrap_or(0);

        session_map.insert(id, (metadata, node, message_count));
    }

    // Find root nodes (no parent)
    let roots: Vec<SessionId> = session_map
        .iter()
        .filter(|(_, (_, node, _))| node.is_root())
        .map(|(id, _)| *id)
        .collect();

    // Build tree recursively
    let mut tree_nodes = Vec::new();
    for root_id in roots {
        if let Some(tree_node) = build_tree_node_recursive(root_id, &session_map) {
            tree_nodes.push(tree_node);
        }
    }

    Ok(tree_nodes)
}

/// Recursively build a tree node and its children.
fn build_tree_node_recursive(
    id: SessionId,
    session_map: &HashMap<SessionId, (SessionMetadata, SessionNode, usize)>,
) -> Option<TreeNode> {
    let (metadata, node, message_count) = session_map.get(&id)?.clone();

    let mut children = Vec::new();
    for child_id in &node.child_ids {
        if let Some(child_node) = build_tree_node_recursive(*child_id, session_map) {
            children.push(child_node);
        }
    }

    Some(TreeNode {
        id,
        metadata,
        node,
        children,
        message_count,
    })
}

/// Render the session tree as ASCII art.
pub fn render_tree(
    nodes: &[TreeNode],
    options: &TreeRenderOptions,
) -> Result<String, FaeAgentError> {
    if nodes.is_empty() {
        return Ok("No sessions found. Start a conversation to create one.".to_string());
    }

    let mut output = String::new();
    output.push_str("Session Tree\n");
    output.push_str("────────────\n\n");

    for (i, node) in nodes.iter().enumerate() {
        let is_last = i == nodes.len() - 1;
        render_node_recursive(node, "", is_last, options, &mut output);
    }

    Ok(output)
}

/// Recursively render a tree node with proper ASCII art.
fn render_node_recursive(
    node: &TreeNode,
    prefix: &str,
    is_last: bool,
    options: &TreeRenderOptions,
    output: &mut String,
) {
    // Apply filters
    if let Some(after) = options.after_date
        && node.metadata.last_active < after
    {
        return;
    }

    if let Some(before) = options.before_date
        && node.metadata.last_active > before
    {
        return;
    }

    if !options.tags.is_empty() {
        let has_tag = options
            .tags
            .iter()
            .any(|tag| node.metadata.tags.contains(tag));
        if !has_tag {
            return;
        }
    }

    // Draw the current node
    let connector = if is_last { "└──" } else { "├──" };

    let highlight = if let Some(highlight_id) = options.highlight_id {
        if highlight_id == node.id {
            "➤ "
        } else {
            ""
        }
    } else {
        ""
    };

    let title = node
        .metadata
        .title
        .as_deref()
        .unwrap_or("(untitled)");

    let last_active = node.metadata.last_active.format("%Y-%m-%d %H:%M");

    output.push_str(&format!(
        "{}{} {}{} │ {} │ {} msgs │ {}\n",
        prefix,
        connector,
        highlight,
        node.id.prefix(),
        title,
        node.message_count,
        last_active
    ));

    // Draw children
    let child_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    for (i, child) in node.children.iter().enumerate() {
        let child_is_last = i == node.children.len() - 1;
        render_node_recursive(child, &child_prefix, child_is_last, options, output);
    }
}

/// Find a specific session in the tree by ID.
pub fn find_in_tree(nodes: &[TreeNode], target_id: SessionId) -> Option<TreeNode> {
    for node in nodes {
        if node.id == target_id {
            return Some(node.clone());
        }

        if let Some(found) = find_in_tree(&node.children, target_id) {
            return Some(found);
        }
    }

    None
}

/// List all sessions with metadata (helper for tree building).
fn list_all_sessions_with_metadata(
    storage: &SessionStorage,
) -> Result<Vec<(SessionId, SessionMetadata)>, FaeAgentError> {
    let base_path = storage.base_path();

    if !base_path.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(base_path).map_err(|e| {
        FaeAgentError::Session(format!("Failed to read sessions directory: {}", e))
    })?;

    let mut sessions = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| {
            FaeAgentError::Session(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();
        if path.is_dir()
            && let Some(dir_name) = path.file_name().and_then(|s| s.to_str())
            && let Ok(session_id) = SessionId::from_str(dir_name)
            && let Ok(metadata) = storage.load_manifest(&session_id)
        {
            sessions.push((session_id, metadata));
        }
    }

    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_storage() -> (TempDir, SessionStorage) {
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(_) => panic!("Failed to create temp dir for test"),
        };
        let storage = SessionStorage::with_base_path(temp_dir.path().to_path_buf());
        (temp_dir, storage)
    }

    #[test]
    fn test_empty_tree() {
        let (_temp, storage) = test_storage();
        let tree = build_session_tree(&storage);
        assert!(tree.is_ok());
        match tree {
            Ok(nodes) => assert!(nodes.is_empty()),
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_single_session_tree() {
        let (_temp, storage) = test_storage();

        let id = SessionId::new();
        let metadata = SessionMetadata::new();
        let node = SessionNode::new_root(id);

        assert!(storage.save_manifest(&id, &metadata).is_ok());
        assert!(storage.save_tree(&id, &node).is_ok());

        let tree = build_session_tree(&storage);
        assert!(tree.is_ok());
        match tree {
            Ok(nodes) => {
                assert!(nodes.len() == 1);
                assert!(nodes[0].id == id);
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_render_empty_tree() {
        let nodes = Vec::new();
        let options = TreeRenderOptions::default();
        let result = render_tree(&nodes, &options);
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                assert!(output.contains("No sessions found"));
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_render_single_node() {
        let id = SessionId::new();
        let mut metadata = SessionMetadata::new();
        metadata.title = Some("Test Session".to_string());
        let node = SessionNode::new_root(id);

        let tree_node = TreeNode {
            id,
            metadata,
            node,
            children: Vec::new(),
            message_count: 5,
        };

        let options = TreeRenderOptions::default();
        let result = render_tree(&[tree_node], &options);
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                assert!(output.contains("Test Session"));
                assert!(output.contains("5 msgs"));
                assert!(output.contains(&id.prefix()));
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_render_with_highlight() {
        let id = SessionId::new();
        let metadata = SessionMetadata::new();
        let node = SessionNode::new_root(id);

        let tree_node = TreeNode {
            id,
            metadata,
            node,
            children: Vec::new(),
            message_count: 0,
        };

        let options = TreeRenderOptions {
            highlight_id: Some(id),
            ..Default::default()
        };

        let result = render_tree(&[tree_node], &options);
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                assert!(output.contains("➤"));
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_render_multi_level_tree() {
        let root_id = SessionId::new();
        let child_id = SessionId::new();

        let root_meta = SessionMetadata::new();
        let mut child_meta = SessionMetadata::new();
        child_meta.title = Some("Child Session".to_string());

        let mut root_node = SessionNode::new_root(root_id);
        root_node.add_child(child_id);
        let child_node = SessionNode::new_child(child_id, root_id);

        let child_tree_node = TreeNode {
            id: child_id,
            metadata: child_meta,
            node: child_node,
            children: Vec::new(),
            message_count: 3,
        };

        let root_tree_node = TreeNode {
            id: root_id,
            metadata: root_meta,
            node: root_node,
            children: vec![child_tree_node],
            message_count: 2,
        };

        let options = TreeRenderOptions::default();
        let result = render_tree(&[root_tree_node], &options);
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                assert!(output.contains("Child Session"));
                assert!(output.contains("│"));
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_filter_by_date() {
        let id = SessionId::new();
        let mut metadata = SessionMetadata::new();
        metadata.last_active = Utc::now();
        let node = SessionNode::new_root(id);

        let tree_node = TreeNode {
            id,
            metadata,
            node,
            children: Vec::new(),
            message_count: 0,
        };

        // Filter to future date - should not show
        let options = TreeRenderOptions {
            after_date: Some(Utc::now() + chrono::Duration::hours(1)),
            ..Default::default()
        };

        let result = render_tree(std::slice::from_ref(&tree_node), &options);
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                // Should only have header, no session
                assert!(!output.contains(&id.prefix()));
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_filter_by_tag() {
        let id = SessionId::new();
        let mut metadata = SessionMetadata::new();
        metadata.add_tag("important".to_string());
        let node = SessionNode::new_root(id);

        let tree_node = TreeNode {
            id,
            metadata,
            node,
            children: Vec::new(),
            message_count: 0,
        };

        // Filter for non-existent tag
        let options = TreeRenderOptions {
            tags: vec!["other".to_string()],
            ..Default::default()
        };

        let result = render_tree(std::slice::from_ref(&tree_node), &options);
        assert!(result.is_ok());
        match result {
            Ok(output) => {
                assert!(!output.contains(&id.prefix()));
            }
            Err(_) => unreachable!(),
        }

        // Filter for matching tag
        let options2 = TreeRenderOptions {
            tags: vec!["important".to_string()],
            ..Default::default()
        };

        let result2 = render_tree(&[tree_node], &options2);
        assert!(result2.is_ok());
        match result2 {
            Ok(output) => {
                assert!(output.contains(&id.prefix()));
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_find_in_tree() {
        let root_id = SessionId::new();
        let child_id = SessionId::new();

        let root_meta = SessionMetadata::new();
        let child_meta = SessionMetadata::new();

        let mut root_node = SessionNode::new_root(root_id);
        root_node.add_child(child_id);
        let child_node = SessionNode::new_child(child_id, root_id);

        let child_tree_node = TreeNode {
            id: child_id,
            metadata: child_meta,
            node: child_node,
            children: Vec::new(),
            message_count: 0,
        };

        let root_tree_node = TreeNode {
            id: root_id,
            metadata: root_meta,
            node: root_node,
            children: vec![child_tree_node],
            message_count: 0,
        };

        // Find child in tree
        let found = find_in_tree(&[root_tree_node], child_id);
        assert!(found.is_some());
        match found {
            Some(node) => assert!(node.id == child_id),
            None => unreachable!(),
        }
    }
}
