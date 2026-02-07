//! Skill registry for loading and managing skills.

use crate::error::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::types::Skill;

/// Registry of loaded skills.
#[derive(Debug, Clone, Default)]
pub struct SkillRegistry {
    /// Skills indexed by name.
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    /// Create a new empty skill registry.
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Discover skill files in the standard locations.
    ///
    /// Searches:
    /// - Project: .saorsa-tui/skills/*.md
    /// - Global: ~/.saorsa-tui/skills/*.md
    pub fn discover_skills() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Project skills (.saorsa-tui/skills/)
        let project_dir = PathBuf::from(".saorsa-tui/skills");
        if project_dir.exists()
            && let Ok(entries) = std::fs::read_dir(&project_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    paths.push(path);
                }
            }
        }

        // Global skills (~/.saorsa-tui/skills/)
        if let Some(home) = dirs::home_dir() {
            let global_dir = home.join(".saorsa-tui/skills");
            if global_dir.exists()
                && let Ok(entries) = std::fs::read_dir(&global_dir)
            {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        paths.push(path);
                    }
                }
            }
        }

        paths
    }

    /// Load a skill from a file.
    ///
    /// Skill file format:
    /// ```markdown
    /// ---
    /// name: skill_name
    /// description: What this skill does
    /// triggers:
    ///   - keyword1
    ///   - keyword2
    /// ---
    /// Skill content here in markdown...
    /// ```
    pub fn load_skill(path: &Path) -> Result<Skill> {
        let content = std::fs::read_to_string(path)?;
        parse_skill_file(&content)
    }

    /// Add a skill to the registry.
    pub fn add_skill(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    /// Get a skill by name.
    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// List all loaded skills.
    pub fn list_skills(&self) -> Vec<&Skill> {
        self.skills.values().collect()
    }

    /// Activate a skill, returning its content for injection into context.
    pub fn activate_skill(&self, name: &str) -> Option<String> {
        self.skills.get(name).map(|skill| skill.content.clone())
    }

    /// Load all discovered skills into the registry.
    pub fn load_all_discovered(&mut self) -> Result<()> {
        let paths = Self::discover_skills();
        for path in paths {
            match Self::load_skill(&path) {
                Ok(skill) => self.add_skill(skill),
                Err(_) => {
                    // Skip invalid skill files
                    continue;
                }
            }
        }
        Ok(())
    }
}

/// Parse a skill file with front matter.
fn parse_skill_file(content: &str) -> Result<Skill> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(crate::error::SaorsaAgentError::Context(
            "Skill file must start with front matter".into(),
        ));
    }

    let after_first = &trimmed[3..];
    let end_pos = after_first.find("---").ok_or_else(|| {
        crate::error::SaorsaAgentError::Context("Front matter not properly closed".into())
    })?;

    let front_matter = &after_first[..end_pos];
    let body = after_first[end_pos + 3..].trim_start();

    // Parse front matter (simple YAML-like parsing)
    let mut name = String::new();
    let mut description = String::new();
    let mut triggers = Vec::new();
    let mut in_triggers = false;

    for line in front_matter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(trigger_item) = line.strip_prefix("- ") {
            // Trigger item
            if in_triggers {
                triggers.push(trigger_item.trim().to_string());
            }
        } else if let Some(value) = line.strip_prefix("name:") {
            name = value.trim().to_string();
            in_triggers = false;
        } else if let Some(value) = line.strip_prefix("description:") {
            description = value.trim().to_string();
            in_triggers = false;
        } else if line.starts_with("triggers:") {
            in_triggers = true;
        }
    }

    if name.is_empty() {
        return Err(crate::error::SaorsaAgentError::Context(
            "Skill must have a name".into(),
        ));
    }

    Ok(Skill::new(name, description, triggers, body.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_temp_dir() -> TempDir {
        match TempDir::new() {
            Ok(t) => t,
            Err(e) => unreachable!("Failed to create temp dir: {e}"),
        }
    }

    fn create_skill_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(format!("{}.md", name));
        assert!(fs::write(&path, content).is_ok());
        path
    }

    #[test]
    fn test_parse_skill_file_valid() {
        let content = r#"---
name: test_skill
description: A test skill
triggers:
  - test
  - example
---
Skill content here"#;

        let result = parse_skill_file(content);
        assert!(result.is_ok());

        let skill = match result {
            Ok(s) => s,
            Err(_) => unreachable!("Should parse successfully"),
        };

        assert_eq!(skill.name, "test_skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.triggers.len(), 2);
        assert_eq!(skill.triggers[0], "test");
        assert_eq!(skill.triggers[1], "example");
        assert_eq!(skill.content, "Skill content here");
    }

    #[test]
    fn test_parse_skill_file_no_front_matter() {
        let content = "Just content without front matter";
        let result = parse_skill_file(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_skill_file_missing_name() {
        let content = r#"---
description: Missing name
---
Content"#;
        let result = parse_skill_file(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_add_and_get() {
        let mut registry = SkillRegistry::new();
        let skill = Skill::new("test", "Test skill", vec![], "Content");

        registry.add_skill(skill.clone());

        let retrieved = registry.get_skill("test");
        assert!(retrieved.is_some());

        match retrieved {
            Some(s) => assert_eq!(s.name, "test"),
            None => unreachable!("Should find skill"),
        }
    }

    #[test]
    fn test_registry_list_skills() {
        let mut registry = SkillRegistry::new();
        registry.add_skill(Skill::new("skill1", "First", vec![], "Content1"));
        registry.add_skill(Skill::new("skill2", "Second", vec![], "Content2"));

        let skills = registry.list_skills();
        assert_eq!(skills.len(), 2);
    }

    #[test]
    fn test_activate_skill() {
        let mut registry = SkillRegistry::new();
        let skill = Skill::new("test", "Test", vec![], "Skill content");
        registry.add_skill(skill);

        let content = registry.activate_skill("test");
        assert!(content.is_some());

        match content {
            Some(c) => assert_eq!(c, "Skill content"),
            None => unreachable!("Should activate skill"),
        }
    }

    #[test]
    fn test_activate_nonexistent_skill() {
        let registry = SkillRegistry::new();
        let content = registry.activate_skill("nonexistent");
        assert!(content.is_none());
    }

    #[test]
    fn test_load_skill_from_file() {
        let temp = make_temp_dir();
        let content = r#"---
name: file_skill
description: Loaded from file
triggers:
  - load
---
File content"#;

        let path = create_skill_file(temp.path(), "test", content);
        let result = SkillRegistry::load_skill(&path);
        assert!(result.is_ok());

        let skill = match result {
            Ok(s) => s,
            Err(_) => unreachable!("Should load skill"),
        };

        assert_eq!(skill.name, "file_skill");
        assert_eq!(skill.content, "File content");
    }

    #[test]
    fn test_load_invalid_skill_file() {
        let temp = make_temp_dir();
        let content = "Invalid skill file";
        let path = create_skill_file(temp.path(), "invalid", content);

        let result = SkillRegistry::load_skill(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new("name", "desc", vec!["trigger".to_string()], "content");
        assert_eq!(skill.name, "name");
        assert_eq!(skill.description, "desc");
        assert_eq!(skill.triggers.len(), 1);
        assert_eq!(skill.content, "content");
    }
}
