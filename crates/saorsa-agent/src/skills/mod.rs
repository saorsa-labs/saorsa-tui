//! Skills system for on-demand capability injection.
//!
//! Skills are markdown files with front matter that can be discovered and
//! activated to inject specialized knowledge into the agent context.

pub mod registry;
pub mod types;

pub use registry::SkillRegistry;
pub use types::Skill;
