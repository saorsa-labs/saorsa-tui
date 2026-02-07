//! Session management types and functionality.

/// Filesystem path utilities.
pub mod path;
/// Session storage and serialization.
pub mod storage;
/// Core session types (ID, metadata, messages, tree nodes).
pub mod types;

pub use storage::SessionStorage;
pub use types::{Message, SessionId, SessionMetadata, SessionNode};
