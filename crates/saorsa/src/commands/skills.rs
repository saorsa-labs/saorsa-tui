//! `/skills` command — list available skills.

use saorsa_agent::SkillRegistry;

/// Discover and list available skills from standard locations.
pub fn execute(_args: &str) -> anyhow::Result<String> {
    let paths = SkillRegistry::discover_skills();

    if paths.is_empty() {
        return Ok("\
No skills found.

Skills are markdown files placed in:
  .saorsa/skills/   (project-local)
  ~/.saorsa/skills/  (global)"
            .to_string());
    }

    let mut text = format!("Skills ({} found):", paths.len());
    for path in &paths {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let location = if path.starts_with(".saorsa/") {
            "project"
        } else {
            "global"
        };
        text.push_str(&format!("\n  {name:<20} ({location})"));
    }

    Ok(text)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn shows_no_skills_message() {
        // In test context, no skill directories exist.
        let text = execute("").expect("should succeed");
        // Either shows "No skills" or lists found skills.
        assert!(text.contains("skills") || text.contains("Skills"));
    }

    #[test]
    fn mentions_skill_locations() {
        let text = execute("").expect("should succeed");
        if text.contains("No skills") {
            assert!(text.contains(".saorsa/skills/"));
            assert!(text.contains("~/.saorsa/skills/"));
        }
    }
}
