//! Session management types and functionality.

/// Automatic session saving with debouncing and retry logic.
pub mod autosave;
/// Filesystem path utilities.
pub mod path;
/// Session continuation and resumption.
pub mod resume;
/// Session storage and serialization.
pub mod storage;
/// Core session types (ID, metadata, messages, tree nodes).
pub mod types;

pub use autosave::{AutoSaveConfig, AutoSaveManager};
pub use resume::{find_last_active_session, find_session_by_prefix, restore_session};
pub use storage::SessionStorage;
pub use types::{Message, SessionId, SessionMetadata, SessionNode};
