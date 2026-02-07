//! Skill type definitions.

use serde::{Deserialize, Serialize};

/// A skill represents on-demand capability that can be activated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique skill name.
    pub name: String,
    /// Human-readable description of what this skill does.
    pub description: String,
    /// Keywords or patterns that might trigger this skill.
    pub triggers: Vec<String>,
    /// The skill content (markdown) to inject into context.
    pub content: String,
}

impl Skill {
    /// Create a new skill.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        triggers: Vec<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            triggers,
            content: content.into(),
        }
    }
}
