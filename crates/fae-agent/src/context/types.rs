//! Common types for context engineering.

use serde::{Deserialize, Serialize};

/// Strategy for merging multiple context files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    /// Use only the highest precedence file, ignore others.
    Replace,
    /// Merge all files in precedence order with separators.
    #[default]
    Append,
}

/// Mode for SYSTEM.md integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SystemMode {
    /// Completely replace the default system prompt.
    Replace,
    /// Append custom content after the default system prompt.
    #[default]
    Append,
}
