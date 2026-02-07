//! Session management types and functionality.

/// Core session types (ID, metadata, messages, tree nodes).
pub mod types;

pub use types::{Message, SessionId, SessionMetadata, SessionNode};
