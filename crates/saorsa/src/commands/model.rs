//! `/model` command — list models, switch, enable, or disable.

use crate::app::AppState;

/// List available models with the current one highlighted.
pub fn list_models(state: &AppState) -> anyhow::Result<String> {
    let mut text = format!("Models (current: {})", state.model);
    if state.enabled_models.is_empty() {
        text.push_str("\n  (none configured — add models to ~/.saorsa/settings.json)");
    } else {
        for (i, m) in state.enabled_models.iter().enumerate() {
            let marker = if i == state.model_index { " *" } else { "" };
            text.push_str(&format!("\n  {m}{marker}"));
        }
    }
    text.push_str("\n\nSubcommands: /model enable <name>, /model disable <name>");
    Ok(text)
}

/// Switch to a model by name or partial name.
///
/// First checks `enabled_models` for a partial match, otherwise sets the
/// model name directly.
pub fn switch_model(name: &str, state: &mut AppState) -> anyhow::Result<String> {
    let target = name.trim();

    // Try partial match in enabled_models.
    if let Some(pos) = state.enabled_models.iter().position(|m| m.contains(target)) {
        state.model_index = pos;
        state.model = state.enabled_models[pos].clone();
        return Ok(format!("Switched to: {}", state.model));
    }

    // Accept the name directly (user may specify a model not in the list).
    state.model = target.to_string();
    Ok(format!("Switched to: {}", state.model))
}

/// Enable a model for Ctrl+P cycling.
///
/// Adds the model to `enabled_models` if not already present.
pub fn enable_model(name: &str, state: &mut AppState) -> anyhow::Result<String> {
    let target = name.trim();
    if target.is_empty() {
        return Ok("Usage: /model enable <model-name>".into());
    }
    if state.enabled_models.iter().any(|m| m == target) {
        return Ok(format!("{target} is already enabled."));
    }
    state.enabled_models.push(target.to_string());
    Ok(format!(
        "Enabled: {target} (now {} models in rotation)",
        state.enabled_models.len()
    ))
}

/// Disable a model from Ctrl+P cycling.
///
/// Removes the model from `enabled_models`. Adjusts `model_index` if needed.
pub fn disable_model(name: &str, state: &mut AppState) -> anyhow::Result<String> {
    let target = name.trim();
    if target.is_empty() {
        return Ok("Usage: /model disable <model-name>".into());
    }
    if let Some(pos) = state.enabled_models.iter().position(|m| m == target) {
        state.enabled_models.remove(pos);
        // Adjust index if it pointed at or past the removed entry.
        if !state.enabled_models.is_empty() {
            if state.model_index >= state.enabled_models.len() {
                state.model_index = state.enabled_models.len() - 1;
            }
        } else {
            state.model_index = 0;
        }
        Ok(format!(
            "Disabled: {target} ({} models in rotation)",
            state.enabled_models.len()
        ))
    } else {
        Ok(format!("{target} is not in the enabled list."))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn list_models_shows_marker() {
        let mut state = AppState::new("model-a");
        state.enabled_models = vec!["model-a".into(), "model-b".into()];
        state.model_index = 0;
        let text = list_models(&state).expect("should succeed");
        assert!(text.contains("model-a *"));
        assert!(text.contains("model-b"));
        assert!(!text.contains("model-b *"));
    }

    #[test]
    fn switch_to_existing_model() {
        let mut state = AppState::new("model-a");
        state.enabled_models = vec!["model-a".into(), "model-b".into()];
        let text = switch_model("model-b", &mut state).expect("should succeed");
        assert!(text.contains("model-b"));
        assert_eq!(state.model, "model-b");
        assert_eq!(state.model_index, 1);
    }

    #[test]
    fn switch_to_nonexistent_model() {
        let mut state = AppState::new("model-a");
        state.enabled_models = vec!["model-a".into()];
        let text = switch_model("gpt-4o", &mut state).expect("should succeed");
        assert!(text.contains("gpt-4o"));
        assert_eq!(state.model, "gpt-4o");
    }

    #[test]
    fn switch_updates_index() {
        let mut state = AppState::new("a");
        state.enabled_models = vec!["alpha".into(), "beta".into(), "gamma".into()];
        state.model_index = 0;
        switch_model("beta", &mut state).expect("should succeed");
        assert_eq!(state.model_index, 1);
    }

    #[test]
    fn enable_adds_model() {
        let mut state = AppState::new("test");
        let text = enable_model("gpt-4o", &mut state).expect("should succeed");
        assert!(text.contains("Enabled"));
        assert_eq!(state.enabled_models, vec!["gpt-4o"]);
    }

    #[test]
    fn enable_duplicate_is_noop() {
        let mut state = AppState::new("test");
        state.enabled_models = vec!["gpt-4o".into()];
        let text = enable_model("gpt-4o", &mut state).expect("should succeed");
        assert!(text.contains("already enabled"));
        assert_eq!(state.enabled_models.len(), 1);
    }

    #[test]
    fn disable_removes_model() {
        let mut state = AppState::new("test");
        state.enabled_models = vec!["model-a".into(), "model-b".into()];
        state.model_index = 1;
        let text = disable_model("model-b", &mut state).expect("should succeed");
        assert!(text.contains("Disabled"));
        assert_eq!(state.enabled_models, vec!["model-a"]);
        assert_eq!(state.model_index, 0);
    }

    #[test]
    fn disable_nonexistent_is_noop() {
        let mut state = AppState::new("test");
        let text = disable_model("nope", &mut state).expect("should succeed");
        assert!(text.contains("not in the enabled list"));
    }

    #[test]
    fn enable_empty_shows_usage() {
        let mut state = AppState::new("test");
        let text = enable_model("", &mut state).expect("should succeed");
        assert!(text.contains("Usage"));
    }
}
