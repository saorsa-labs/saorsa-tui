//! Model selector widget with fuzzy search and favorites.

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use saorsa_ai::models::{ModelInfo, lookup_model};
use saorsa_ai::provider::ProviderKind;
use saorsa_core::buffer::ScreenBuffer;
use saorsa_core::color::Color;
use saorsa_core::event::{Event, KeyCode, KeyEvent, Modifiers};
use saorsa_core::geometry::Rect;
use saorsa_core::segment::Segment;
use saorsa_core::style::Style;
use saorsa_core::widget::{BorderStyle, EventResult, InteractiveWidget, SelectList, Widget};
use std::collections::HashSet;

/// Model entry with metadata for display.
#[derive(Clone, Debug)]
pub struct ModelEntry {
    /// Model name.
    pub name: String,
    /// Model info from registry.
    pub info: Option<ModelInfo>,
    /// Whether this model is favorited.
    pub is_favorite: bool,
}

impl ModelEntry {
    /// Create a new model entry.
    pub fn new(name: String, is_favorite: bool) -> Self {
        let info = lookup_model(&name);
        Self {
            name,
            info,
            is_favorite,
        }
    }
}

/// Model selector widget with fuzzy search and favorites.
///
/// Displays a list of available models with metadata (provider, context window).
/// Supports fuzzy search, favorite toggling with 'f' key, and Ctrl+L cycling through favorites.
pub struct ModelSelector {
    /// All available models.
    models: Vec<ModelEntry>,
    /// Favorite model names.
    favorites: HashSet<String>,
    /// Current filter query.
    filter_query: String,
    /// Fuzzy matcher for filtering.
    matcher: SkimMatcherV2,
    /// Currently selected model (if confirmed).
    selected_model: Option<String>,
    /// Whether the selector is visible.
    is_visible: bool,
    /// Internal select list widget.
    select_list: Option<SelectList<ModelEntry>>,
}

impl ModelSelector {
    /// Create a new model selector with a list of model names.
    pub fn new(model_names: Vec<String>) -> Self {
        let mut models = Vec::new();
        let favorites = HashSet::new();

        for name in model_names {
            models.push(ModelEntry::new(name, false));
        }

        Self {
            models,
            favorites,
            filter_query: String::new(),
            matcher: SkimMatcherV2::default(),
            selected_model: None,
            is_visible: false,
            select_list: None,
        }
    }

    /// Add a model to favorites.
    pub fn add_favorite(&mut self, model: String) {
        self.favorites.insert(model.clone());
        if let Some(entry) = self.models.iter_mut().find(|m| m.name == model) {
            entry.is_favorite = true;
        }
    }

    /// Remove a model from favorites.
    pub fn remove_favorite(&mut self, model: &str) {
        self.favorites.remove(model);
        if let Some(entry) = self.models.iter_mut().find(|m| m.name == model) {
            entry.is_favorite = false;
        }
    }

    /// Get all favorite models.
    pub fn favorites(&self) -> &HashSet<String> {
        &self.favorites
    }

    /// Toggle favorite status of a model.
    pub fn toggle_favorite(&mut self, model: &str) {
        if self.favorites.contains(model) {
            self.remove_favorite(model);
        } else {
            self.add_favorite(model.to_string());
        }
    }

    /// Get the currently selected model.
    pub fn selected_model(&self) -> Option<&str> {
        self.selected_model.as_deref()
    }

    /// Show the selector.
    pub fn show(&mut self) {
        self.is_visible = true;
        self.rebuild_list();
    }

    /// Hide the selector.
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.select_list = None;
    }

    /// Check if the selector is visible.
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Set the filter query.
    pub fn set_filter(&mut self, query: String) {
        self.filter_query = query;
        self.rebuild_list();
    }

    /// Get filtered and sorted models.
    fn get_filtered_models(&self) -> Vec<ModelEntry> {
        let mut models: Vec<ModelEntry> = if self.filter_query.is_empty() {
            self.models.clone()
        } else {
            self.models
                .iter()
                .filter_map(|m| {
                    self.matcher
                        .fuzzy_match(&m.name, &self.filter_query)
                        .map(|score| (m.clone(), score))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|(m, _)| m)
                .collect()
        };

        // Sort: favorites first, then alphabetically
        models.sort_by(|a, b| match (a.is_favorite, b.is_favorite) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        models
    }

    /// Rebuild the internal select list.
    fn rebuild_list(&mut self) {
        let filtered = self.get_filtered_models();

        let mut list = SelectList::new(filtered)
            .with_border(BorderStyle::Rounded)
            .with_selected_style(
                Style::default()
                    .bg(Color::Rgb {
                        r: 40,
                        g: 40,
                        b: 80,
                    })
                    .bold(true),
            )
            .with_render_fn(|entry: &ModelEntry| {
                let mut segments = Vec::new();

                // Favorite indicator
                if entry.is_favorite {
                    segments.push(Segment::styled(
                        "★ ",
                        Style::default().fg(Color::Rgb {
                            r: 255,
                            g: 215,
                            b: 0,
                        }),
                    ));
                } else {
                    segments.push(Segment::new("  "));
                }

                // Model name
                segments.push(Segment::styled(&entry.name, Style::default().bold(true)));

                // Provider and metadata
                if let Some(info) = &entry.info {
                    let provider_str = match info.provider {
                        ProviderKind::Anthropic => " [Anthropic]",
                        ProviderKind::OpenAi => " [OpenAI]",
                        ProviderKind::Gemini => " [Gemini]",
                        ProviderKind::Ollama => " [Ollama]",
                        ProviderKind::OpenAiCompatible => " [Compatible]",
                        ProviderKind::LmStudio => " [LM Studio]",
                        ProviderKind::Vllm => " [vLLM]",
                        ProviderKind::OpenRouter => " [OpenRouter]",
                    };
                    segments.push(Segment::styled(
                        provider_str,
                        Style::default().fg(Color::Rgb {
                            r: 100,
                            g: 100,
                            b: 100,
                        }),
                    ));

                    // Context window
                    let ctx = if info.context_window >= 1_000_000 {
                        format!(" {}M", info.context_window / 1_000_000)
                    } else {
                        format!(" {}k", info.context_window / 1_000)
                    };
                    segments.push(Segment::styled(
                        &ctx,
                        Style::default().fg(Color::Rgb {
                            r: 150,
                            g: 150,
                            b: 150,
                        }),
                    ));

                    // Capabilities
                    let mut caps = Vec::new();
                    if info.supports_tools {
                        caps.push("tools");
                    }
                    if info.supports_vision {
                        caps.push("vision");
                    }
                    if !caps.is_empty() {
                        segments.push(Segment::styled(
                            format!(" ({})", caps.join(", ")),
                            Style::default().fg(Color::Rgb {
                                r: 120,
                                g: 120,
                                b: 120,
                            }),
                        ));
                    }
                }

                segments
            })
            .with_search_fn(|entry: &ModelEntry| entry.name.clone());

        // Set selection callback
        let selected_clone = self.selected_model.clone();
        list = list.with_on_select(move |_entry: &ModelEntry| {
            // This will be handled by external code
            let _ = &selected_clone;
        });

        self.select_list = Some(list);
    }

    /// Cycle to the next favorite model.
    pub fn cycle_favorites(&mut self) -> Option<String> {
        let fav_list: Vec<_> = self.favorites.iter().cloned().collect();
        if fav_list.is_empty() {
            return None;
        }

        let current = self.selected_model.as_ref();
        let next_idx = if let Some(current_model) = current {
            if let Some(pos) = fav_list.iter().position(|m| m == current_model) {
                (pos + 1) % fav_list.len()
            } else {
                0
            }
        } else {
            0
        };

        let next = fav_list[next_idx].clone();
        self.selected_model = Some(next.clone());
        Some(next)
    }
}

impl Widget for ModelSelector {
    fn render(&self, area: Rect, buffer: &mut ScreenBuffer) {
        if !self.is_visible {
            return;
        }

        if let Some(list) = &self.select_list {
            list.render(area, buffer);
        }
    }
}

impl InteractiveWidget for ModelSelector {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        if !self.is_visible {
            return EventResult::Ignored;
        }

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: Modifiers::NONE,
            }) => {
                // Toggle favorite for current selection
                let model_name = if let Some(list) = &self.select_list {
                    list.selected_item().map(|entry| entry.name.clone())
                } else {
                    None
                };
                if let Some(name) = model_name {
                    self.toggle_favorite(&name);
                    self.rebuild_list();
                }
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: Modifiers::CTRL,
            }) => {
                // Cycle favorites
                if let Some(next) = self.cycle_favorites() {
                    self.selected_model = Some(next);
                }
                self.hide();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Escape,
                modifiers: Modifiers::NONE,
            }) => {
                self.hide();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: Modifiers::NONE,
            }) => {
                // Confirm selection
                if let Some(list) = &self.select_list
                    && let Some(entry) = list.selected_item()
                {
                    self.selected_model = Some(entry.name.clone());
                }
                self.hide();
                EventResult::Consumed
            }
            _ => {
                // Delegate to select list
                if let Some(list) = &mut self.select_list {
                    list.handle_event(event)
                } else {
                    EventResult::Ignored
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_models() -> Vec<String> {
        vec![
            "claude-sonnet-4".to_string(),
            "claude-opus-4".to_string(),
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gemini-2.0-flash".to_string(),
        ]
    }

    #[test]
    fn new_model_selector() {
        let selector = ModelSelector::new(sample_models());
        assert_eq!(selector.models.len(), 5);
        assert!(selector.favorites.is_empty());
        assert!(!selector.is_visible);
        assert!(selector.selected_model.is_none());
    }

    #[test]
    fn add_remove_favorites() {
        let mut selector = ModelSelector::new(sample_models());
        selector.add_favorite("claude-sonnet-4".to_string());
        assert!(selector.favorites.contains("claude-sonnet-4"));

        selector.remove_favorite("claude-sonnet-4");
        assert!(!selector.favorites.contains("claude-sonnet-4"));
    }

    #[test]
    fn toggle_favorite() {
        let mut selector = ModelSelector::new(sample_models());
        selector.toggle_favorite("gpt-4o");
        assert!(selector.favorites.contains("gpt-4o"));

        selector.toggle_favorite("gpt-4o");
        assert!(!selector.favorites.contains("gpt-4o"));
    }

    #[test]
    fn show_hide() {
        let mut selector = ModelSelector::new(sample_models());
        assert!(!selector.is_visible());

        selector.show();
        assert!(selector.is_visible());

        selector.hide();
        assert!(!selector.is_visible());
    }

    #[test]
    fn cycle_favorites_empty() {
        let mut selector = ModelSelector::new(sample_models());
        assert!(selector.cycle_favorites().is_none());
    }

    #[test]
    fn cycle_favorites() {
        let mut selector = ModelSelector::new(sample_models());
        selector.add_favorite("claude-sonnet-4".to_string());
        selector.add_favorite("gpt-4o".to_string());

        let first = selector.cycle_favorites();
        assert!(first.is_some());

        let second = selector.cycle_favorites();
        assert!(second.is_some());
        assert_ne!(first, second);

        // Should wrap around
        let third = selector.cycle_favorites();
        assert_eq!(first, third);
    }

    #[test]
    fn filtered_models_empty_query() {
        let selector = ModelSelector::new(sample_models());
        let filtered = selector.get_filtered_models();
        assert_eq!(filtered.len(), 5);
    }

    #[test]
    fn filtered_models_with_query() {
        let mut selector = ModelSelector::new(sample_models());
        selector.set_filter("claude".to_string());
        let filtered = selector.get_filtered_models();
        assert!(filtered.iter().all(|m| m.name.contains("claude")));
    }

    #[test]
    fn favorites_sorted_first() {
        let mut selector = ModelSelector::new(sample_models());
        selector.add_favorite("gpt-4o".to_string());
        let filtered = selector.get_filtered_models();
        assert!(filtered.first().map(|m| m.name.as_str()) == Some("gpt-4o"));
    }

    #[test]
    fn model_entry_with_known_model() {
        let entry = ModelEntry::new("claude-sonnet-4".to_string(), false);
        assert!(entry.info.is_some());
        if let Some(info) = entry.info {
            assert_eq!(info.provider, ProviderKind::Anthropic);
            assert_eq!(info.context_window, 200_000);
        }
    }

    #[test]
    fn model_entry_with_unknown_model() {
        let entry = ModelEntry::new("unknown-model".to_string(), false);
        assert!(entry.info.is_none());
    }

    #[test]
    fn visibility_when_hidden() {
        let selector = ModelSelector::new(sample_models());
        assert!(!selector.is_visible());
    }

    #[test]
    fn visibility_when_shown() {
        let mut selector = ModelSelector::new(sample_models());
        selector.show();
        assert!(selector.is_visible());
    }

    #[test]
    fn handle_escape_hides() {
        let mut selector = ModelSelector::new(sample_models());
        selector.show();
        assert!(selector.is_visible());

        let event = Event::Key(KeyEvent {
            code: KeyCode::Escape,
            modifiers: Modifiers::NONE,
        });
        let result = selector.handle_event(&event);
        assert!(matches!(result, EventResult::Consumed));
        assert!(!selector.is_visible());
    }

    #[test]
    fn handle_event_when_hidden() {
        let mut selector = ModelSelector::new(sample_models());
        let event = Event::Key(KeyEvent {
            code: KeyCode::Enter,
            modifiers: Modifiers::NONE,
        });
        let result = selector.handle_event(&event);
        assert!(matches!(result, EventResult::Ignored));
    }
}
