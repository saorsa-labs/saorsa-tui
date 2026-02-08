//! App runtime integrating DOM, TCSS, layout, and rendering.

use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;

use crate::RenderContext;
use crate::buffer::ScreenBuffer;
use crate::error::{Result, SaorsaTuiError};
use crate::event::{Event, KeyCode, KeyEvent, Modifiers, MouseEventKind};
use crate::geometry::{Rect, Size};
use crate::layout::{LayoutEngine, computed_to_taffy};
use crate::tcss::{
    CascadeResolver, ComputedStyle, MatchCache, StyleMatcher, StylesheetEvent, StylesheetLoader,
    ThemeManager, VariableEnvironment, WidgetTree,
};
use crate::widget::EventResult;

use super::dom::{Dom, NodeId};

type AppAction = Box<dyn FnMut(&mut App) -> Result<EventResult>>;

#[derive(Clone, Debug)]
struct KeyBinding {
    code: KeyCode,
    modifiers: Modifiers,
    action: String,
}

/// Retained-mode application runtime.
pub struct App {
    dom: Dom,
    matcher: StyleMatcher,
    match_cache: MatchCache,
    vars: VariableEnvironment,
    theme_mgr: ThemeManager,
    active_theme: Option<String>,
    stylesheet_loader: StylesheetLoader,
    stylesheet_watcher: Option<notify::RecommendedWatcher>,
    stylesheet_rx: Option<mpsc::Receiver<StylesheetEvent>>,
    actions: HashMap<String, AppAction>,
    bindings: Vec<KeyBinding>,

    layout: LayoutEngine,
    rects: HashMap<NodeId, Rect>,
    render_order: Vec<NodeId>,
    computed: HashMap<NodeId, ComputedStyle>,

    render: RenderContext,
    dirty: bool,
    last_focused: Option<NodeId>,
}

impl App {
    /// Create a new app runtime from a DOM and a stylesheet loader.
    ///
    /// The app will compute styles and layout on demand and render via
    /// [`RenderContext`].
    pub fn new(
        terminal: &dyn crate::terminal::Terminal,
        dom: Dom,
        loader: StylesheetLoader,
    ) -> Result<Self> {
        let render = RenderContext::new(terminal)?;

        let matcher = StyleMatcher::new(loader.stylesheet());

        let mut theme_mgr = ThemeManager::new();
        for theme in loader.themes() {
            theme_mgr.register(theme.clone());
        }

        let vars = VariableEnvironment::with_global(loader.globals().clone());

        let mut app = Self {
            dom,
            matcher,
            match_cache: MatchCache::new(),
            vars,
            theme_mgr,
            active_theme: None,
            stylesheet_loader: loader,
            stylesheet_watcher: None,
            stylesheet_rx: None,
            actions: HashMap::new(),
            bindings: Vec::new(),
            layout: LayoutEngine::new(),
            rects: HashMap::new(),
            render_order: Vec::new(),
            computed: HashMap::new(),
            render,
            dirty: true,
            last_focused: None,
        };

        app.build_layout_tree()?;
        Ok(app)
    }

    /// Convenience: build from a TCSS string.
    pub fn from_tcss_string(
        terminal: &dyn crate::terminal::Terminal,
        dom: Dom,
        tcss: &str,
    ) -> Result<Self> {
        let loader = StylesheetLoader::load_string(tcss)
            .map_err(|e| SaorsaTuiError::Style(e.to_string()))?;
        Self::new(terminal, dom, loader)
    }

    /// Convenience: build from a TCSS file and enable hot reload.
    ///
    /// To apply updates, call [`Self::poll_stylesheet_reload`] regularly
    /// (e.g. once per event loop tick). Successful reloads mark the app dirty.
    pub fn from_tcss_file(
        terminal: &dyn crate::terminal::Terminal,
        dom: Dom,
        path: impl AsRef<Path>,
    ) -> Result<Self> {
        let loader = StylesheetLoader::load_file(path.as_ref())
            .map_err(|e| SaorsaTuiError::Style(e.to_string()))?;
        let mut app = Self::new(terminal, dom, loader)?;

        let (watcher, rx) = crate::tcss::reload::watch_stylesheet(path.as_ref())
            .map_err(|e| SaorsaTuiError::Style(e.to_string()))?;
        app.stylesheet_watcher = Some(watcher);
        app.stylesheet_rx = Some(rx);
        Ok(app)
    }

    /// Access the DOM.
    pub fn dom(&self) -> &Dom {
        &self.dom
    }

    /// Access the DOM mutably.
    pub fn dom_mut(&mut self) -> &mut Dom {
        &mut self.dom
    }

    /// Get the last computed layout rectangle for a node (if layout ran).
    pub fn rect_of(&self, node: super::dom::NodeRef) -> Option<Rect> {
        self.rects.get(&node.0).copied()
    }

    /// Register an application-level action handler.
    ///
    /// Actions can be invoked via key bindings (see [`Self::bind_key`]).
    /// Registering a handler replaces any existing handler with the same name.
    pub fn register_action(&mut self, name: impl Into<String>, action: AppAction) {
        self.actions.insert(name.into(), action);
    }

    /// Bind a key chord to an action name.
    ///
    /// If multiple bindings match, the first registered binding wins.
    pub fn bind_key(&mut self, key: KeyEvent, action: impl Into<String>) {
        self.bindings.push(KeyBinding {
            code: key.code,
            modifiers: key.modifiers,
            action: action.into(),
        });
    }

    /// Query nodes using a TCSS selector string, in DOM pre-order.
    pub fn query(&self, selector: &str) -> Result<Vec<super::dom::NodeRef>> {
        let selectors = crate::tcss::SelectorList::parse(selector)
            .map_err(|e| SaorsaTuiError::Style(e.to_string()))?;
        let mut out = Vec::new();
        let root = match self.dom.root() {
            Some(r) => r,
            None => return Ok(out),
        };

        let mut order = Vec::new();
        pre_order(self.dom.widget_tree(), root, &mut order);

        for id in order {
            if crate::tcss::StyleMatcher::matches_any(self.dom.widget_tree(), id, &selectors)
                .is_some()
            {
                out.push(super::dom::NodeRef(id));
            }
        }
        Ok(out)
    }

    /// Query the first matching node using a TCSS selector string.
    pub fn query_one(&self, selector: &str) -> Result<Option<super::dom::NodeRef>> {
        Ok(self.query(selector)?.into_iter().next())
    }

    /// Attach `child` to `parent` and update layout/style caches.
    pub fn mount(&mut self, parent: super::dom::NodeRef, child: super::dom::NodeRef) -> Result<()> {
        if !self.dom.contains(parent.0) || !self.dom.contains(child.0) {
            return Err(SaorsaTuiError::Widget("mount: unknown node".into()));
        }

        // Prevent double-parenting.
        if self
            .dom
            .widget_tree()
            .get(child.0)
            .and_then(|n| n.parent)
            .is_some()
        {
            return Err(SaorsaTuiError::Widget(
                "mount: child already has a parent".into(),
            ));
        }

        self.dom.append_child(parent, child);

        // Lifecycle: mount subtree in pre-order.
        let ids = self.dom.subtree_pre_order(child);
        for id in ids {
            if let Some(n) = self.dom.node_mut(id) {
                n.widget.on_mount();
            }
        }

        self.ensure_layout_nodes(child.0)?;
        self.sync_layout_edges(parent)?;

        let tree = self.dom.widget_tree();
        self.match_cache.invalidate_subtree(tree, parent.0);
        self.dirty = true;
        Ok(())
    }

    /// Remove a node and all descendants from the DOM and layout tree.
    pub fn remove_subtree(&mut self, node: super::dom::NodeRef) -> Result<()> {
        if !self.dom.contains(node.0) {
            return Ok(());
        }
        if self.dom.root() == Some(node.0) {
            return Err(SaorsaTuiError::Widget("cannot remove root node".into()));
        }

        // Capture parent before mutation.
        let parent = self
            .dom
            .widget_tree()
            .get(node.0)
            .and_then(|n| n.parent)
            .map(super::dom::NodeRef);

        // Lifecycle: unmount subtree (pre-order).
        for id in self.dom.subtree_pre_order(node) {
            if let Some(n) = self.dom.node_mut(id) {
                n.widget.on_unmount();
            }
        }

        // Remove from DOM, then remove from layout.
        let removed_ids = self.dom.remove_subtree(node);
        for id in removed_ids {
            if self.layout.has_node(id) {
                self.layout
                    .remove_node(id)
                    .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;
            }
        }

        // Update the parent's children edges after removal.
        if let Some(p) = parent
            && self.dom.contains(p.0)
        {
            self.sync_layout_edges(p)?;
            let tree = self.dom.widget_tree();
            self.match_cache.invalidate_subtree(tree, p.0);
        } else {
            self.match_cache.invalidate_all();
        }

        self.rects.remove(&node.0);
        self.dirty = true;
        Ok(())
    }

    /// Mark the app as needing a re-render.
    pub fn request_render(&mut self) {
        self.dirty = true;
    }

    /// Poll for TCSS hot reload events (if enabled).
    ///
    /// Returns `Ok(Some(event))` if a reload or error event was processed.
    /// A successful reload updates the internal matcher and variable environment
    /// and marks the app dirty.
    pub fn poll_stylesheet_reload(&mut self) -> Result<Option<StylesheetEvent>> {
        let Some(rx) = self.stylesheet_rx.as_ref() else {
            return Ok(None);
        };

        // Drain all queued events; the last one wins.
        let mut last: Option<StylesheetEvent> = None;
        while let Ok(ev) = rx.try_recv() {
            last = Some(ev);
        }

        let Some(ev) = last else {
            return Ok(None);
        };

        match ev {
            StylesheetEvent::Reloaded { .. } => {
                let reloaded = self
                    .stylesheet_loader
                    .reload()
                    .map_err(|e| SaorsaTuiError::Style(e.to_string()))?;
                self.apply_stylesheet_loader_state();
                Ok(Some(reloaded))
            }
            StylesheetEvent::Error(msg) => Ok(Some(StylesheetEvent::Error(msg))),
        }
    }

    /// Replace the stylesheet loader state (in-memory).
    ///
    /// This updates the matcher, globals/themes, invalidates the match cache,
    /// and marks the app dirty.
    pub fn set_stylesheet_loader(&mut self, loader: StylesheetLoader) {
        self.stylesheet_loader = loader;
        self.apply_stylesheet_loader_state();
    }

    /// Reload stylesheet content from an in-memory string.
    ///
    /// This updates the matcher, globals/themes, invalidates the match cache,
    /// and marks the app dirty.
    pub fn reload_stylesheet_string(&mut self, css: &str) -> Result<StylesheetEvent> {
        let ev = self
            .stylesheet_loader
            .reload_string(css)
            .map_err(|e| SaorsaTuiError::Style(e.to_string()))?;
        self.apply_stylesheet_loader_state();
        Ok(ev)
    }

    /// Set the active theme by name (a theme is a class selector with variables).
    pub fn set_active_theme(&mut self, name: Option<&str>) {
        self.active_theme = name.map(str::to_string);
        let mut layer = crate::tcss::VariableMap::new();
        if let Some(name) = self.active_theme.as_deref() {
            // Ignore invalid theme names; they simply result in an empty theme layer.
            let _ = self.theme_mgr.set_active(name);
            if let Some(theme) = self.theme_mgr.active_theme() {
                layer = theme.variables().clone();
            }
        }
        self.vars.set_theme_layer(layer);
        self.match_cache.invalidate_all();
        self.dirty = true;
    }

    /// Handle an input event (focus management + dispatch).
    ///
    /// Returns whether the event was consumed.
    pub fn handle_event(&mut self, event: &Event) -> Result<EventResult> {
        // Key bindings to application actions.
        if let Event::Key(key) = event
            && let Some(action_name) = self.lookup_binding(key)
        {
            // Temporarily remove to avoid borrowing self.actions while calling action(&mut self).
            if let Some(mut action) = self.actions.remove(&action_name) {
                let res = action(self);
                self.actions.insert(action_name.clone(), action);
                let res = res?;
                if matches!(res, EventResult::Consumed) {
                    self.dirty = true;
                }
                return Ok(res);
            }
        }

        match event {
            Event::Resize(w, h) => {
                self.handle_resize(Size::new(*w, *h));
                return Ok(EventResult::Consumed);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers,
            }) => {
                // Shift+Tab = previous, Tab = next.
                if modifiers.contains(Modifiers::SHIFT) {
                    self.dom.focus_mut().focus_previous();
                } else {
                    self.dom.focus_mut().focus_next();
                }
                self.sync_focus_state();
                self.dirty = true;
                return Ok(EventResult::Consumed);
            }
            Event::Mouse(me) if matches!(me.kind, MouseEventKind::Press) => {
                // Click-to-focus based on hit test.
                if let Some(id) = self.hit_test(me.x, me.y) {
                    self.dom.focus_mut().set_focus(id);
                    self.sync_focus_state();
                    self.dirty = true;
                }
            }
            _ => {}
        }

        // Dispatch to focused widget if it exists.
        let focused = self.dom.focus().focused();
        if let Some(id) = focused
            && let Some(node) = self.dom.node_mut(id)
        {
            let res = node.widget.handle_event(event);
            if matches!(res, EventResult::Consumed) {
                self.dirty = true;
            }
            return Ok(res);
        }
        Ok(EventResult::Ignored)
    }

    /// Render if dirty.
    pub fn render_if_needed(
        &mut self,
        terminal: &mut dyn crate::terminal::Terminal,
    ) -> Result<bool> {
        if !self.dirty {
            return Ok(false);
        }
        self.render_frame(terminal)?;
        Ok(true)
    }

    /// Force a render.
    pub fn render_frame(&mut self, terminal: &mut dyn crate::terminal::Terminal) -> Result<()> {
        self.sync_focus_state();
        self.compute_styles()?;
        self.compute_layout()?;

        self.render.begin_frame();
        let buf: &mut ScreenBuffer = self.render.buffer_mut();
        for &id in &self.render_order {
            let area = self
                .rects
                .get(&id)
                .copied()
                .unwrap_or(Rect::new(0, 0, 0, 0));
            if let Some(node) = self.dom.node_mut(id) {
                node.widget.render(area, buf);
            }
        }
        self.render.end_frame(terminal)?;
        self.dirty = false;
        Ok(())
    }

    /// Handle terminal resize.
    pub fn handle_resize(&mut self, new_size: Size) {
        self.render.handle_resize(new_size);
        self.rects.clear();
        self.dirty = true;
    }

    // ---------------------------
    // Internal: style + layout
    // ---------------------------

    fn sync_focus_state(&mut self) {
        let focused = self.dom.focus().focused();
        if focused == self.last_focused {
            return;
        }

        // Clear previous focus.
        if let Some(prev) = self.last_focused {
            if let Some(n) = self.dom.widget_tree_mut().get_mut(prev) {
                n.state.focused = false;
            }
            let tree = self.dom.widget_tree();
            self.match_cache.invalidate_subtree(tree, prev);
        }

        // Set new focus.
        if let Some(now) = focused {
            if let Some(n) = self.dom.widget_tree_mut().get_mut(now) {
                n.state.focused = true;
            }
            let tree = self.dom.widget_tree();
            self.match_cache.invalidate_subtree(tree, now);
        }

        self.last_focused = focused;
    }

    fn compute_styles(&mut self) -> Result<()> {
        self.computed.clear();
        for id in self.dom.node_ids() {
            let matches = if let Some(cached) = self.match_cache.get(id) {
                cached.clone()
            } else {
                let m = self.matcher.match_widget(self.dom.widget_tree(), id);
                self.match_cache.insert(id, m.clone());
                m
            };

            let computed = CascadeResolver::resolve_with_variables(&matches, &self.vars);
            self.computed.insert(id, computed);
        }

        // Apply computed style into widgets.
        for (id, style) in self.computed.iter() {
            if let Some(node) = self.dom.node_mut(*id) {
                node.widget.apply_computed_style(style);
            }
        }

        Ok(())
    }

    fn lookup_binding(&self, key: &KeyEvent) -> Option<String> {
        self.bindings
            .iter()
            .find(|b| b.code == key.code && b.modifiers == key.modifiers)
            .map(|b| b.action.clone())
    }

    fn ensure_layout_nodes(&mut self, subtree_root: NodeId) -> Result<()> {
        let ids = self
            .dom
            .subtree_post_order(super::dom::NodeRef(subtree_root));
        for id in ids {
            if !self.layout.has_node(id) {
                self.layout
                    .add_node(id, taffy::Style::default())
                    .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn sync_layout_edges(&mut self, subtree_root: super::dom::NodeRef) -> Result<()> {
        let ids = self.dom.subtree_pre_order(subtree_root);
        for id in ids {
            if !self.layout.has_node(id) {
                // Can happen if user calls sync directly; ensure node exists.
                self.layout
                    .add_node(id, taffy::Style::default())
                    .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;
            }

            let children = self.dom.widget_tree().children(id).to_vec();
            self.layout
                .set_children(id, &children)
                .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;
        }
        Ok(())
    }

    fn apply_stylesheet_loader_state(&mut self) {
        self.matcher = StyleMatcher::new(self.stylesheet_loader.stylesheet());

        // Rebuild theme manager from current loader themes.
        let mut mgr = ThemeManager::new();
        for theme in self.stylesheet_loader.themes() {
            mgr.register(theme.clone());
        }
        self.theme_mgr = mgr;

        // Reset globals and re-apply current theme selection.
        let globals = self.stylesheet_loader.globals().clone();
        self.vars = VariableEnvironment::with_global(globals);

        let active = self.active_theme.clone();
        self.active_theme = None;
        self.set_active_theme(active.as_deref());

        self.match_cache.invalidate_all();
        self.dirty = true;
    }

    fn build_layout_tree(&mut self) -> Result<()> {
        let root = self
            .dom
            .root()
            .ok_or_else(|| SaorsaTuiError::Widget("DOM has no root".into()))?;

        // Post-order traversal to ensure children are added before parents.
        let mut order = Vec::new();
        post_order(self.dom.widget_tree(), root, &mut order);

        for id in order {
            let children = self.dom.widget_tree().children(id).to_vec();
            if children.is_empty() {
                self.layout
                    .add_node(id, taffy::Style::default())
                    .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;
            } else {
                self.layout
                    .add_node_with_children(id, taffy::Style::default(), &children)
                    .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;
            }
        }

        self.layout
            .set_root(root)
            .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;

        Ok(())
    }

    fn compute_layout(&mut self) -> Result<()> {
        let size = self.render.size();
        let root = self
            .dom
            .root()
            .ok_or_else(|| SaorsaTuiError::Widget("DOM has no root".into()))?;

        // Update per-node styles in layout engine.
        for id in self.dom.node_ids() {
            let computed = self.computed.get(&id).cloned().unwrap_or_default();
            let mut taffy_style = computed_to_taffy(&computed);

            // Root is always the full terminal viewport.
            if id == root {
                taffy_style.size.width = taffy::Dimension::Length(f32::from(size.width));
                taffy_style.size.height = taffy::Dimension::Length(f32::from(size.height));
            }

            self.layout
                .update_style(id, taffy_style)
                .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;
        }

        self.layout
            .compute(size.width, size.height)
            .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;

        self.rects.clear();
        for id in self.dom.node_ids() {
            let rect = self
                .layout
                .layout_rect(id)
                .map_err(|e| SaorsaTuiError::Layout(e.to_string()))?;
            self.rects.insert(id, rect);
        }

        self.render_order.clear();
        pre_order(self.dom.widget_tree(), root, &mut self.render_order);

        Ok(())
    }

    fn hit_test(&self, x: u16, y: u16) -> Option<NodeId> {
        // Prefer the deepest node whose rect contains the point.
        // This is a simple approximation without z-index or clipping.
        let mut best: Option<(usize, NodeId)> = None;
        for (&id, rect) in &self.rects {
            if contains(*rect, x, y) {
                let depth = self.dom.widget_tree().ancestors(id).len();
                best = Some(match best {
                    Some((d, bid)) if d >= depth => (d, bid),
                    _ => (depth, id),
                });
            }
        }
        best.map(|(_, id)| id)
    }
}

fn post_order(tree: &WidgetTree, id: NodeId, out: &mut Vec<NodeId>) {
    for &child in tree.children(id) {
        post_order(tree, child, out);
    }
    out.push(id);
}

fn pre_order(tree: &WidgetTree, id: NodeId, out: &mut Vec<NodeId>) {
    out.push(id);
    for &child in tree.children(id) {
        pre_order(tree, child, out);
    }
}

fn contains(rect: Rect, x: u16, y: u16) -> bool {
    let x2 = rect.position.x.saturating_add(rect.size.width);
    let y2 = rect.position.y.saturating_add(rect.size.height);
    x >= rect.position.x && x < x2 && y >= rect.position.y && y < y2
}
