//! Settings screen widget with tabs for configuration.

use saorsa_tui::buffer::ScreenBuffer;
use saorsa_tui::cell::Cell;
use saorsa_tui::color::{Color, NamedColor};
use saorsa_tui::event::{Event, KeyCode, KeyEvent, Modifiers};
use saorsa_tui::geometry::Rect;
use saorsa_tui::style::Style;
use saorsa_tui::widget::{Checkbox, EventResult, InteractiveWidget, Tab, Tabs, TextArea, Widget};
use std::collections::HashMap;

/// Tab identifier for settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingsTab {
    /// General settings.
    General,
    /// Model configuration.
    Models,
    /// Keybinding customization.
    Keybindings,
    /// Context engineering settings.
    Context,
}

/// Settings data structure.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Compact mode (minimal UI).
    pub compact_mode: bool,
    /// Thinking mode (show reasoning).
    pub thinking_mode: bool,
    /// Auto-save conversations.
    pub auto_save: bool,
    /// Default model.
    pub default_model: String,
    /// Temperature (0.0-2.0).
    pub temperature: f32,
    /// Max tokens per response.
    pub max_tokens: u32,
    /// Keybinding mappings (action -> key).
    pub keybindings: HashMap<String, String>,
    /// Context discovery enabled.
    pub context_discovery: bool,
    /// Max tokens per context section.
    pub max_context_tokens: u32,
}

impl Default for Settings {
    fn default() -> Self {
        let mut keybindings = HashMap::new();
        keybindings.insert("send".to_string(), "Ctrl+Enter".to_string());
        keybindings.insert("cancel".to_string(), "Escape".to_string());
        keybindings.insert("new_chat".to_string(), "Ctrl+N".to_string());
        keybindings.insert("model_selector".to_string(), "Ctrl+L".to_string());
        keybindings.insert("settings".to_string(), "Ctrl+,".to_string());

        Self {
            compact_mode: false,
            thinking_mode: false,
            auto_save: true,
            default_model: "claude-sonnet-4".to_string(),
            temperature: 1.0,
            max_tokens: 4096,
            keybindings,
            context_discovery: true,
            max_context_tokens: 8000,
        }
    }
}

/// Settings screen widget.
pub struct SettingsScreen {
    /// Current settings.
    settings: Settings,
    /// Original settings (for cancel).
    original_settings: Settings,
    /// Current tab.
    current_tab: SettingsTab,
    /// Tabs widget.
    tabs: Tabs,
    /// Whether the screen is visible.
    is_visible: bool,
    /// Whether settings have been modified.
    is_dirty: bool,
    /// General tab widgets.
    compact_checkbox: Checkbox,
    thinking_checkbox: Checkbox,
    autosave_checkbox: Checkbox,
    /// Models tab widgets.
    model_input: TextArea,
    temp_input: TextArea,
    max_tokens_input: TextArea,
    /// Context tab widgets.
    discovery_checkbox: Checkbox,
    context_tokens_input: TextArea,
}

impl SettingsScreen {
    /// Create a new settings screen with default settings.
    pub fn new() -> Self {
        let settings = Settings::default();
        let original_settings = settings.clone();

        let tabs = Tabs::new(vec![
            Tab::new("General"),
            Tab::new("Models"),
            Tab::new("Keybindings"),
            Tab::new("Context"),
        ]);

        Self {
            settings: settings.clone(),
            original_settings,
            current_tab: SettingsTab::General,
            tabs,
            is_visible: false,
            is_dirty: false,
            compact_checkbox: Checkbox::new("Compact mode").with_checked(settings.compact_mode),
            thinking_checkbox: Checkbox::new("Thinking mode").with_checked(settings.thinking_mode),
            autosave_checkbox: Checkbox::new("Auto-save").with_checked(settings.auto_save),
            model_input: TextArea::from_text(&settings.default_model),
            temp_input: TextArea::from_text(&settings.temperature.to_string()),
            max_tokens_input: TextArea::from_text(&settings.max_tokens.to_string()),
            discovery_checkbox: Checkbox::new("Context discovery")
                .with_checked(settings.context_discovery),
            context_tokens_input: TextArea::from_text(&settings.max_context_tokens.to_string()),
        }
    }

    /// Show the settings screen.
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    /// Hide the settings screen.
    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    /// Check if the screen is visible.
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Get the current settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get a mutable reference to the current settings.
    pub fn settings_mut(&mut self) -> &mut Settings {
        self.is_dirty = true;
        &mut self.settings
    }

    /// Check if settings have been modified.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Save settings and close.
    pub fn save(&mut self) {
        self.original_settings = self.settings.clone();
        self.is_dirty = false;
        self.hide();
    }

    /// Cancel changes and close.
    pub fn cancel(&mut self) {
        self.settings = self.original_settings.clone();
        self.is_dirty = false;
        self.hide();
    }

    /// Switch to a different tab.
    fn switch_tab(&mut self, tab: SettingsTab) {
        self.current_tab = tab;
        let idx = match tab {
            SettingsTab::General => 0,
            SettingsTab::Models => 1,
            SettingsTab::Keybindings => 2,
            SettingsTab::Context => 3,
        };
        self.tabs.set_active_tab(idx);
    }

    /// Render the General tab.
    fn render_general_tab(&self, area: Rect, buffer: &mut ScreenBuffer) {
        let mut y = area.position.y + 2;
        if y >= area.position.y + area.size.height {
            return;
        }

        // Render checkboxes
        self.compact_checkbox.render(
            Rect::new(area.position.x + 2, y, area.size.width - 4, 1),
            buffer,
        );
        y += 2;

        if y < area.position.y + area.size.height {
            self.thinking_checkbox.render(
                Rect::new(area.position.x + 2, y, area.size.width - 4, 1),
                buffer,
            );
            y += 2;
        }

        if y < area.position.y + area.size.height {
            self.autosave_checkbox.render(
                Rect::new(area.position.x + 2, y, area.size.width - 4, 1),
                buffer,
            );
        }
    }

    /// Render the Models tab.
    fn render_models_tab(&self, area: Rect, buffer: &mut ScreenBuffer) {
        let mut y = area.position.y + 2;
        if y >= area.position.y + area.size.height {
            return;
        }

        // Model input
        self.write_label(buffer, area.position.x + 2, y, "Default model:");
        y += 1;
        if y < area.position.y + area.size.height {
            self.model_input.render(
                Rect::new(area.position.x + 4, y, area.size.width - 6, 3),
                buffer,
            );
            y += 4;
        }

        // Temperature input
        if y < area.position.y + area.size.height {
            self.write_label(buffer, area.position.x + 2, y, "Temperature (0.0-2.0):");
            y += 1;
        }
        if y < area.position.y + area.size.height {
            self.temp_input.render(
                Rect::new(area.position.x + 4, y, area.size.width - 6, 3),
                buffer,
            );
            y += 4;
        }

        // Max tokens input
        if y < area.position.y + area.size.height {
            self.write_label(buffer, area.position.x + 2, y, "Max tokens:");
            y += 1;
        }
        if y < area.position.y + area.size.height {
            self.max_tokens_input.render(
                Rect::new(area.position.x + 4, y, area.size.width - 6, 3),
                buffer,
            );
        }
    }

    /// Helper to write a label to the buffer.
    fn write_label(&self, buffer: &mut ScreenBuffer, x: u16, y: u16, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            if let Some(cell) = buffer.get_mut(x + i as u16, y) {
                *cell = Cell::new(ch.to_string(), Style::default().bold(true));
            }
        }
    }

    /// Render the Keybindings tab.
    fn render_keybindings_tab(&self, area: Rect, buffer: &mut ScreenBuffer) {
        let y = area.position.y + 2;
        if y >= area.position.y + area.size.height {
            return;
        }

        self.write_label(
            buffer,
            area.position.x + 2,
            y,
            "Keybinding customization coming soon...",
        );
    }

    /// Render the Context tab.
    fn render_context_tab(&self, area: Rect, buffer: &mut ScreenBuffer) {
        let mut y = area.position.y + 2;
        if y >= area.position.y + area.size.height {
            return;
        }

        // Discovery checkbox
        self.discovery_checkbox.render(
            Rect::new(area.position.x + 2, y, area.size.width - 4, 1),
            buffer,
        );
        y += 2;

        // Max context tokens
        if y < area.position.y + area.size.height {
            self.write_label(buffer, area.position.x + 2, y, "Max context tokens:");
            y += 1;
        }
        if y < area.position.y + area.size.height {
            self.context_tokens_input.render(
                Rect::new(area.position.x + 4, y, area.size.width - 6, 3),
                buffer,
            );
        }
    }

    /// Render save/cancel buttons.
    fn render_buttons(&self, area: Rect, buffer: &mut ScreenBuffer) {
        let y = area.position.y + area.size.height.saturating_sub(2);
        if y < area.position.y {
            return;
        }

        // Write save button
        let mut x = area.position.x + 2;
        for (i, ch) in "[S] Save".chars().enumerate() {
            if let Some(cell) = buffer.get_mut(x + i as u16, y) {
                *cell = Cell::new(
                    ch.to_string(),
                    Style::default().fg(Color::Named(NamedColor::Green)),
                );
            }
        }

        // Write cancel button
        x += 12; // "[S] Save" + 2 spaces
        for (i, ch) in "[C] Cancel".chars().enumerate() {
            if let Some(cell) = buffer.get_mut(x + i as u16, y) {
                *cell = Cell::new(
                    ch.to_string(),
                    Style::default().fg(Color::Named(NamedColor::Red)),
                );
            }
        }
    }
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for SettingsScreen {
    fn render(&self, area: Rect, buffer: &mut ScreenBuffer) {
        if !self.is_visible {
            return;
        }

        // Render tabs
        self.tabs.render(
            Rect::new(area.position.x, area.position.y, area.size.width, 3),
            buffer,
        );

        // Render active tab content
        let content_area = Rect::new(
            area.position.x,
            area.position.y + 3,
            area.size.width,
            area.size.height.saturating_sub(3),
        );

        match self.current_tab {
            SettingsTab::General => self.render_general_tab(content_area, buffer),
            SettingsTab::Models => self.render_models_tab(content_area, buffer),
            SettingsTab::Keybindings => self.render_keybindings_tab(content_area, buffer),
            SettingsTab::Context => self.render_context_tab(content_area, buffer),
        }

        // Render buttons
        self.render_buttons(area, buffer);
    }
}

impl InteractiveWidget for SettingsScreen {
    fn handle_event(&mut self, event: &Event) -> EventResult {
        if !self.is_visible {
            return EventResult::Ignored;
        }

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: Modifiers::NONE,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('S'),
                modifiers: Modifiers::SHIFT,
            }) => {
                self.save();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: Modifiers::NONE,
            })
            | Event::Key(KeyEvent {
                code: KeyCode::Char('C'),
                modifiers: Modifiers::SHIFT,
            }) => {
                self.cancel();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Escape,
                modifiers: Modifiers::NONE,
            }) => {
                self.cancel();
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: Modifiers::NONE,
            }) => {
                // Cycle tabs
                let next = match self.current_tab {
                    SettingsTab::General => SettingsTab::Models,
                    SettingsTab::Models => SettingsTab::Keybindings,
                    SettingsTab::Keybindings => SettingsTab::Context,
                    SettingsTab::Context => SettingsTab::General,
                };
                self.switch_tab(next);
                EventResult::Consumed
            }
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: Modifiers::SHIFT,
            }) => {
                // Cycle tabs backwards (Shift+Tab)
                let prev = match self.current_tab {
                    SettingsTab::General => SettingsTab::Context,
                    SettingsTab::Models => SettingsTab::General,
                    SettingsTab::Keybindings => SettingsTab::Models,
                    SettingsTab::Context => SettingsTab::Keybindings,
                };
                self.switch_tab(prev);
                EventResult::Consumed
            }
            _ => {
                // Delegate to active tab widgets
                match self.current_tab {
                    SettingsTab::General => {
                        // Try each widget
                        if self.compact_checkbox.handle_event(event) == EventResult::Consumed {
                            return EventResult::Consumed;
                        }
                        if self.thinking_checkbox.handle_event(event) == EventResult::Consumed {
                            return EventResult::Consumed;
                        }
                        if self.autosave_checkbox.handle_event(event) == EventResult::Consumed {
                            return EventResult::Consumed;
                        }
                    }
                    SettingsTab::Models => {
                        // Try each input
                        if self.model_input.handle_event(event) == EventResult::Consumed {
                            return EventResult::Consumed;
                        }
                        if self.temp_input.handle_event(event) == EventResult::Consumed {
                            return EventResult::Consumed;
                        }
                        if self.max_tokens_input.handle_event(event) == EventResult::Consumed {
                            return EventResult::Consumed;
                        }
                    }
                    SettingsTab::Context => {
                        if self.discovery_checkbox.handle_event(event) == EventResult::Consumed {
                            return EventResult::Consumed;
                        }
                        if self.context_tokens_input.handle_event(event) == EventResult::Consumed {
                            return EventResult::Consumed;
                        }
                    }
                    SettingsTab::Keybindings => {
                        // No interactive widgets yet
                    }
                }
                EventResult::Ignored
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_settings_screen() {
        let screen = SettingsScreen::new();
        assert!(!screen.is_visible());
        assert!(!screen.is_dirty());
        assert_eq!(screen.current_tab, SettingsTab::General);
    }

    #[test]
    fn default_settings() {
        let settings = Settings::default();
        assert!(!settings.compact_mode);
        assert!(!settings.thinking_mode);
        assert!(settings.auto_save);
        assert_eq!(settings.default_model, "claude-sonnet-4");
        assert_eq!(settings.temperature, 1.0);
        assert_eq!(settings.max_tokens, 4096);
        assert!(settings.context_discovery);
        assert_eq!(settings.max_context_tokens, 8000);
    }

    #[test]
    fn show_hide() {
        let mut screen = SettingsScreen::new();
        assert!(!screen.is_visible());

        screen.show();
        assert!(screen.is_visible());

        screen.hide();
        assert!(!screen.is_visible());
    }

    #[test]
    fn save_settings() {
        let mut screen = SettingsScreen::new();
        screen.settings_mut().compact_mode = true;
        assert!(screen.is_dirty());

        screen.save();
        assert!(!screen.is_dirty());
        assert!(!screen.is_visible());
        assert!(screen.settings().compact_mode);
    }

    #[test]
    fn cancel_reverts() {
        let mut screen = SettingsScreen::new();
        let original = screen.settings().compact_mode;

        screen.settings_mut().compact_mode = !original;
        assert!(screen.is_dirty());

        screen.cancel();
        assert!(!screen.is_dirty());
        assert_eq!(screen.settings().compact_mode, original);
    }

    #[test]
    fn switch_tabs() {
        let mut screen = SettingsScreen::new();
        assert_eq!(screen.current_tab, SettingsTab::General);

        screen.switch_tab(SettingsTab::Models);
        assert_eq!(screen.current_tab, SettingsTab::Models);

        screen.switch_tab(SettingsTab::Context);
        assert_eq!(screen.current_tab, SettingsTab::Context);
    }

    #[test]
    fn tab_cycling() {
        let mut screen = SettingsScreen::new();
        screen.show();

        let event = Event::Key(KeyEvent {
            code: KeyCode::Tab,
            modifiers: Modifiers::NONE,
        });

        assert_eq!(screen.current_tab, SettingsTab::General);
        screen.handle_event(&event);
        assert_eq!(screen.current_tab, SettingsTab::Models);
        screen.handle_event(&event);
        assert_eq!(screen.current_tab, SettingsTab::Keybindings);
        screen.handle_event(&event);
        assert_eq!(screen.current_tab, SettingsTab::Context);
        screen.handle_event(&event);
        assert_eq!(screen.current_tab, SettingsTab::General);
    }

    #[test]
    fn save_with_key() {
        let mut screen = SettingsScreen::new();
        screen.show();
        screen.settings_mut().thinking_mode = true;

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::NONE,
        });
        let result = screen.handle_event(&event);

        assert!(matches!(result, EventResult::Consumed));
        assert!(!screen.is_visible());
        assert!(!screen.is_dirty());
    }

    #[test]
    fn cancel_with_key() {
        let mut screen = SettingsScreen::new();
        screen.show();
        screen.settings_mut().thinking_mode = true;

        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: Modifiers::NONE,
        });
        let result = screen.handle_event(&event);

        assert!(matches!(result, EventResult::Consumed));
        assert!(!screen.is_visible());
        assert!(!screen.is_dirty());
        assert!(!screen.settings().thinking_mode);
    }

    #[test]
    fn escape_cancels() {
        let mut screen = SettingsScreen::new();
        screen.show();

        let event = Event::Key(KeyEvent {
            code: KeyCode::Escape,
            modifiers: Modifiers::NONE,
        });
        let result = screen.handle_event(&event);

        assert!(matches!(result, EventResult::Consumed));
        assert!(!screen.is_visible());
    }

    #[test]
    fn handle_event_when_hidden() {
        let mut screen = SettingsScreen::new();
        let event = Event::Key(KeyEvent {
            code: KeyCode::Char('s'),
            modifiers: Modifiers::NONE,
        });
        let result = screen.handle_event(&event);
        assert!(matches!(result, EventResult::Ignored));
    }
}
