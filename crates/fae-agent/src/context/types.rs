//! Common types for context engineering.

use super::{AgentsContext, SystemContext};
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

/// Bundle of all context types for agent configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextBundle {
    /// Loaded AGENTS.md context.
    pub agents: String,
    /// Loaded SYSTEM.md context.
    pub system: String,
    /// User-provided context (ad-hoc).
    pub user: String,
}

impl ContextBundle {
    /// Create a new empty context bundle.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for fluent construction.
    pub fn builder() -> ContextBuilder {
        ContextBuilder::default()
    }

    /// Check if all context fields are empty.
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty() && self.system.is_empty() && self.user.is_empty()
    }
}

/// Builder for constructing `ContextBundle` fluently.
#[derive(Debug, Clone, Default)]
pub struct ContextBuilder {
    agents: Option<AgentsContext>,
    system: Option<SystemContext>,
    user: Option<String>,
}

impl ContextBuilder {
    /// Set the agents context.
    #[must_use]
    pub fn agents(mut self, ctx: AgentsContext) -> Self {
        self.agents = Some(ctx);
        self
    }

    /// Set the system context.
    #[must_use]
    pub fn system(mut self, ctx: SystemContext) -> Self {
        self.system = Some(ctx);
        self
    }

    /// Set the user context.
    #[must_use]
    pub fn user(mut self, ctx: impl Into<String>) -> Self {
        self.user = Some(ctx.into());
        self
    }

    /// Build the final `ContextBundle`.
    pub fn build(self) -> ContextBundle {
        ContextBundle {
            agents: self.agents.map(|a| a.content).unwrap_or_default(),
            system: self.system.map(|s| s.content).unwrap_or_default(),
            user: self.user.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_bundle_new() {
        let bundle = ContextBundle::new();
        assert!(bundle.agents.is_empty());
        assert!(bundle.system.is_empty());
        assert!(bundle.user.is_empty());
        assert!(bundle.is_empty());
    }

    #[test]
    fn test_context_bundle_builder() {
        let agents_ctx = AgentsContext {
            content: "Agent instructions".to_string(),
        };
        let system_ctx = SystemContext {
            content: "System instructions".to_string(),
        };

        let bundle = ContextBundle::builder()
            .agents(agents_ctx)
            .system(system_ctx)
            .user("User context")
            .build();

        assert_eq!(bundle.agents, "Agent instructions");
        assert_eq!(bundle.system, "System instructions");
        assert_eq!(bundle.user, "User context");
        assert!(!bundle.is_empty());
    }

    #[test]
    fn test_context_bundle_is_empty() {
        let empty = ContextBundle::new();
        assert!(empty.is_empty());

        let not_empty = ContextBundle {
            agents: "Something".to_string(),
            system: String::new(),
            user: String::new(),
        };
        assert!(!not_empty.is_empty());
    }

    #[test]
    fn test_context_builder_partial() {
        let bundle = ContextBundle::builder().user("Only user").build();

        assert!(bundle.agents.is_empty());
        assert!(bundle.system.is_empty());
        assert_eq!(bundle.user, "Only user");
    }

    #[test]
    fn test_context_bundle_serialization() {
        let bundle = ContextBundle {
            agents: "A".to_string(),
            system: "S".to_string(),
            user: "U".to_string(),
        };

        let json_result = serde_json::to_string(&bundle);
        assert!(json_result.is_ok());

        let json = match json_result {
            Ok(j) => j,
            Err(_) => unreachable!("Serialization should succeed"),
        };

        let deserialized_result: serde_json::Result<ContextBundle> = serde_json::from_str(&json);
        assert!(deserialized_result.is_ok());

        let deserialized = match deserialized_result {
            Ok(d) => d,
            Err(_) => unreachable!("Deserialization should succeed"),
        };

        assert_eq!(deserialized.agents, bundle.agents);
        assert_eq!(deserialized.system, bundle.system);
        assert_eq!(deserialized.user, bundle.user);
    }
}
