//! Session management types and functionality.

/// Automatic session saving with debouncing and retry logic.
pub mod autosave;
/// Session branching and forking.
pub mod branch;
/// Filesystem path utilities.
pub mod path;
/// Session continuation and resumption.
pub mod resume;
/// Session storage and serialization.
pub mod storage;
/// Session tree visualization and hierarchy.
pub mod tree;
/// Core session types (ID, metadata, messages, tree nodes).
pub mod types;

pub use autosave::{AutoSaveConfig, AutoSaveManager};
pub use branch::{auto_fork_on_edit, fork_session};
pub use resume::{find_last_active_session, find_session_by_prefix, restore_session};
pub use storage::SessionStorage;
pub use tree::{TreeNode, TreeRenderOptions, build_session_tree, find_in_tree, render_tree};
pub use types::{Message, SessionId, SessionMetadata, SessionNode};
