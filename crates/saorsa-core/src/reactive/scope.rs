//! Reactive scope for automatic cleanup of effects.
//!
//! A [`ReactiveScope`] owns effects and cleanup callbacks. When the
//! scope is dropped, all its effects are disposed and all cleanup
//! callbacks are run. This ties reactive lifetimes to widget lifetimes.

use super::computed::Computed;
use super::effect::Effect;
use super::signal::Signal;

/// A scope that owns reactive effects and cleanup callbacks.
///
/// When a scope is dropped, all effects it owns are disposed and
/// all cleanup callbacks are run in reverse registration order.
/// Child scopes are dropped before their parent's cleanups run.
///
/// # Examples
///
/// ```ignore
/// let mut scope = ReactiveScope::new();
/// let count = Signal::new(0);
///
/// scope.create_effect({
///     let count = count.clone();
///     move || println!("count = {}", count.get())
/// });
///
/// // Effect runs on creation and when count changes.
/// count.set(1);
///
/// drop(scope); // Effect is disposed, cleanup runs.
/// count.set(2); // Effect does NOT run.
/// ```
pub struct ReactiveScope {
    /// Owned effects that will be disposed on drop.
    effects: Vec<Effect>,
    /// Nested child scopes (dropped before parent cleanups).
    children: Vec<ReactiveScope>,
    /// Cleanup callbacks run on drop (in reverse order).
    cleanups: Vec<Box<dyn FnOnce()>>,
}

impl ReactiveScope {
    /// Create a new empty reactive scope.
    #[must_use]
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            children: Vec::new(),
            cleanups: Vec::new(),
        }
    }

    /// Create a signal. Convenience method — signals are not owned
    /// by the scope since they are shared handles.
    pub fn create_signal<T>(&self, value: T) -> Signal<T> {
        Signal::new(value)
    }

    /// Create a computed value. Convenience method — computed values
    /// are shared handles and not owned by the scope.
    pub fn create_computed<T: Clone + 'static>(&self, f: impl Fn() -> T + 'static) -> Computed<T> {
        Computed::new(f)
    }

    /// Create an effect owned by this scope.
    ///
    /// The effect will be automatically disposed when the scope is dropped.
    pub fn create_effect(&mut self, f: impl FnMut() + 'static) -> Effect {
        let effect = Effect::new(f);
        self.effects.push(effect.clone());
        effect
    }

    /// Register a cleanup callback to run when this scope is dropped.
    ///
    /// Callbacks are run in reverse registration order.
    pub fn on_cleanup(&mut self, f: impl FnOnce() + 'static) {
        self.cleanups.push(Box::new(f));
    }

    /// Create a nested child scope.
    ///
    /// The child scope will be dropped before the parent's cleanups run.
    pub fn child(&mut self) -> &mut ReactiveScope {
        self.children.push(ReactiveScope::new());
        let last = self.children.len() - 1;
        &mut self.children[last]
    }

    /// Get the number of effects owned by this scope.
    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    /// Get the number of child scopes.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

impl Default for ReactiveScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ReactiveScope {
    fn drop(&mut self) {
        // Drop children first (nested scopes).
        self.children.clear();

        // Dispose all effects.
        for effect in &self.effects {
            effect.dispose();
        }

        // Run cleanups in reverse order.
        while let Some(cleanup) = self.cleanups.pop() {
            cleanup();
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn scope_disposes_effects_on_drop() {
        let sig = Signal::new(0);
        let count = Rc::new(Cell::new(0u32));

        let effect;
        {
            let mut scope = ReactiveScope::new();
            effect = scope.create_effect({
                let sig = sig.clone();
                let count = Rc::clone(&count);
                move || {
                    let _ = sig.get();
                    count.set(count.get() + 1);
                }
            });

            sig.subscribe(effect.as_subscriber());
            assert_eq!(count.get(), 1);

            sig.set(1);
            assert_eq!(count.get(), 2);
        }
        // Scope dropped — effect should be disposed.
        assert!(!effect.is_active());

        sig.set(2);
        assert_eq!(count.get(), 2); // No further runs.
    }

    #[test]
    fn scope_runs_cleanups_on_drop() {
        let order = Rc::new(RefCell::new(Vec::new()));

        {
            let mut scope = ReactiveScope::new();
            scope.on_cleanup({
                let order = Rc::clone(&order);
                move || order.borrow_mut().push(1)
            });
            scope.on_cleanup({
                let order = Rc::clone(&order);
                move || order.borrow_mut().push(2)
            });
        }

        // Reverse order.
        assert_eq!(*order.borrow(), vec![2, 1]);
    }

    #[test]
    fn nested_scope_dropped_before_parent_cleanup() {
        let order = Rc::new(RefCell::new(Vec::new()));

        {
            let mut scope = ReactiveScope::new();
            scope.on_cleanup({
                let order = Rc::clone(&order);
                move || order.borrow_mut().push("parent")
            });

            let child = scope.child();
            child.on_cleanup({
                let order = Rc::clone(&order);
                move || order.borrow_mut().push("child")
            });
        }

        // Child cleanup runs before parent cleanup.
        assert_eq!(*order.borrow(), vec!["child", "parent"]);
    }

    #[test]
    fn effect_in_dropped_scope_does_not_fire() {
        let sig = Signal::new(0);
        let count = Rc::new(Cell::new(0u32));

        {
            let mut scope = ReactiveScope::new();
            let effect = scope.create_effect({
                let sig = sig.clone();
                let count = Rc::clone(&count);
                move || {
                    let _ = sig.get();
                    count.set(count.get() + 1);
                }
            });
            sig.subscribe(effect.as_subscriber());
        }

        sig.set(1);
        // Effect was disposed, so count stays at 1 (just the initial run).
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn create_signal_through_scope() {
        let scope = ReactiveScope::new();
        let sig = scope.create_signal(42);
        assert_eq!(sig.get(), 42);
    }

    #[test]
    fn create_computed_through_scope() {
        let scope = ReactiveScope::new();
        let sig = Signal::new(3);
        let doubled = scope.create_computed({
            let sig = sig.clone();
            move || sig.get() * 2
        });
        assert_eq!(doubled.get(), 6);
    }

    #[test]
    fn scope_counts() {
        let mut scope = ReactiveScope::new();
        assert_eq!(scope.effect_count(), 0);
        assert_eq!(scope.child_count(), 0);

        scope.create_effect(|| {});
        assert_eq!(scope.effect_count(), 1);

        scope.child();
        assert_eq!(scope.child_count(), 1);
    }

    #[test]
    fn scope_can_be_moved() {
        let mut scope = ReactiveScope::new();
        scope.create_effect(|| {});

        let scope2 = scope;
        assert_eq!(scope2.effect_count(), 1);
    }

    #[test]
    fn multiple_nested_scopes() {
        let order = Rc::new(RefCell::new(Vec::new()));

        {
            let mut scope = ReactiveScope::new();
            scope.on_cleanup({
                let order = Rc::clone(&order);
                move || order.borrow_mut().push("root")
            });

            let child1 = scope.child();
            child1.on_cleanup({
                let order = Rc::clone(&order);
                move || order.borrow_mut().push("child1")
            });

            let child2 = scope.child();
            child2.on_cleanup({
                let order = Rc::clone(&order);
                move || order.borrow_mut().push("child2")
            });
        }

        // Both children cleanup before root.
        let v = order.borrow().clone();
        assert_eq!(v.len(), 3);
        // Children dropped in order (Vec::clear drops in order).
        assert_eq!(v[0], "child1");
        assert_eq!(v[1], "child2");
        assert_eq!(v[2], "root");
    }

    use std::cell::RefCell;
}
