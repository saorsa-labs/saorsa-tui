//! Focus management for widget navigation.

/// Unique identifier for a focusable widget.
pub type WidgetId = u64;

/// Whether a widget currently has focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusState {
    /// The widget has focus.
    Focused,
    /// The widget does not have focus.
    Unfocused,
}

/// Manages focus among a set of widgets.
///
/// Supports Tab / Shift-Tab navigation with wraparound.
#[derive(Clone, Debug)]
pub struct FocusManager {
    /// Ordered list of focusable widget IDs.
    order: Vec<WidgetId>,
    /// Index of the currently focused widget, or None if nothing is focused.
    current: Option<usize>,
}

impl FocusManager {
    /// Create a new focus manager with no widgets.
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            current: None,
        }
    }

    /// Register a widget as focusable. Order of registration determines tab order.
    pub fn register(&mut self, id: WidgetId) {
        if !self.order.contains(&id) {
            self.order.push(id);
            // Auto-focus the first widget
            if self.current.is_none() {
                self.current = Some(0);
            }
        }
    }

    /// Unregister a widget.
    pub fn unregister(&mut self, id: WidgetId) {
        if let Some(pos) = self.order.iter().position(|&w| w == id) {
            self.order.remove(pos);
            if self.order.is_empty() {
                self.current = None;
            } else if let Some(current) = self.current {
                if current >= self.order.len() {
                    self.current = Some(self.order.len() - 1);
                } else if current > pos {
                    self.current = Some(current - 1);
                }
            }
        }
    }

    /// Get the currently focused widget ID.
    pub fn focused(&self) -> Option<WidgetId> {
        self.current.and_then(|i| self.order.get(i).copied())
    }

    /// Check if a specific widget has focus.
    pub fn focus_state(&self, id: WidgetId) -> FocusState {
        if self.focused() == Some(id) {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        }
    }

    /// Move focus to the next widget (Tab).
    pub fn focus_next(&mut self) {
        if self.order.is_empty() {
            return;
        }
        match self.current {
            Some(i) => {
                self.current = Some((i + 1) % self.order.len());
            }
            None => {
                self.current = Some(0);
            }
        }
    }

    /// Move focus to the previous widget (Shift-Tab).
    pub fn focus_previous(&mut self) {
        if self.order.is_empty() {
            return;
        }
        match self.current {
            Some(i) => {
                if i == 0 {
                    self.current = Some(self.order.len() - 1);
                } else {
                    self.current = Some(i - 1);
                }
            }
            None => {
                self.current = Some(self.order.len() - 1);
            }
        }
    }

    /// Set focus directly to a specific widget.
    pub fn set_focus(&mut self, id: WidgetId) {
        if let Some(pos) = self.order.iter().position(|&w| w == id) {
            self.current = Some(pos);
        }
    }

    /// Get the number of registered focusable widgets.
    pub fn count(&self) -> usize {
        self.order.len()
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_focus_manager() {
        let fm = FocusManager::new();
        assert!(fm.focused().is_none());
        assert_eq!(fm.count(), 0);
    }

    #[test]
    fn register_auto_focuses_first() {
        let mut fm = FocusManager::new();
        fm.register(1);
        assert_eq!(fm.focused(), Some(1));
    }

    #[test]
    fn focus_next_cycles() {
        let mut fm = FocusManager::new();
        fm.register(1);
        fm.register(2);
        fm.register(3);

        assert_eq!(fm.focused(), Some(1));
        fm.focus_next();
        assert_eq!(fm.focused(), Some(2));
        fm.focus_next();
        assert_eq!(fm.focused(), Some(3));
        fm.focus_next();
        assert_eq!(fm.focused(), Some(1)); // wraps around
    }

    #[test]
    fn focus_previous_cycles() {
        let mut fm = FocusManager::new();
        fm.register(1);
        fm.register(2);
        fm.register(3);

        assert_eq!(fm.focused(), Some(1));
        fm.focus_previous();
        assert_eq!(fm.focused(), Some(3)); // wraps to end
        fm.focus_previous();
        assert_eq!(fm.focused(), Some(2));
    }

    #[test]
    fn focus_state_query() {
        let mut fm = FocusManager::new();
        fm.register(1);
        fm.register(2);

        assert_eq!(fm.focus_state(1), FocusState::Focused);
        assert_eq!(fm.focus_state(2), FocusState::Unfocused);
    }

    #[test]
    fn set_focus_directly() {
        let mut fm = FocusManager::new();
        fm.register(10);
        fm.register(20);
        fm.register(30);

        fm.set_focus(30);
        assert_eq!(fm.focused(), Some(30));
    }

    #[test]
    fn unregister_adjusts_focus() {
        let mut fm = FocusManager::new();
        fm.register(1);
        fm.register(2);
        fm.register(3);
        fm.set_focus(2);

        fm.unregister(2);
        // Focus should stay at same index (now pointing to 3)
        assert_eq!(fm.count(), 2);
        assert!(fm.focused().is_some());
    }

    #[test]
    fn unregister_last_clears_focus() {
        let mut fm = FocusManager::new();
        fm.register(1);
        fm.unregister(1);
        assert!(fm.focused().is_none());
    }

    #[test]
    fn duplicate_register_ignored() {
        let mut fm = FocusManager::new();
        fm.register(1);
        fm.register(1);
        assert_eq!(fm.count(), 1);
    }

    #[test]
    fn focus_next_on_empty_is_noop() {
        let mut fm = FocusManager::new();
        fm.focus_next(); // Should not crash
        assert!(fm.focused().is_none());
    }
}
