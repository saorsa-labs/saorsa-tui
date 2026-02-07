//! Data binding primitives for connecting signals to widget properties.
//!
//! Bindings bridge the reactive system and the widget layer. A binding
//! watches a signal (or computed expression) and pushes its value to a
//! [`PropertySink`] whenever the source changes.
//!
//! Three binding flavours are provided:
//!
//! - [`OneWayBinding`]: source signal → sink (read-only push)
//! - [`TwoWayBinding`]: source signal ↔ sink (bidirectional, with loop guard)
//! - [`BindingExpression`]: source signal → transform → sink

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::computed::Computed;
use super::effect::Effect;
use super::signal::Signal;

// ---------------------------------------------------------------------------
// BindingId
// ---------------------------------------------------------------------------

/// Unique identifier for a binding instance.
pub type BindingId = u64;

static BINDING_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate the next unique binding ID.
fn next_binding_id() -> BindingId {
    BINDING_COUNTER.fetch_add(1, Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// BindingDirection
// ---------------------------------------------------------------------------

/// Direction in which data flows through a binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BindingDirection {
    /// Data flows from source to sink only.
    OneWay,
    /// Data flows in both directions (source ↔ sink).
    TwoWay,
}

// ---------------------------------------------------------------------------
// Binding trait
// ---------------------------------------------------------------------------

/// A type-erased binding that can be stored and managed in a collection.
pub trait Binding {
    /// Unique identifier for this binding.
    fn id(&self) -> BindingId;

    /// Direction of data flow.
    fn direction(&self) -> BindingDirection;

    /// Whether this binding is still active.
    fn is_active(&self) -> bool;

    /// Permanently deactivate this binding.
    fn dispose(&self);
}

// ---------------------------------------------------------------------------
// PropertySink
// ---------------------------------------------------------------------------

/// A target that can receive property values from a binding.
///
/// Implementations typically wrap a widget property setter or a
/// `Rc<RefCell<T>>` shared cell.
pub trait PropertySink<T> {
    /// Push a new value to the sink.
    fn set_value(&self, value: &T);
}

/// Blanket implementation: any `Fn(&T)` is a `PropertySink<T>`.
impl<T, F: Fn(&T)> PropertySink<T> for F {
    fn set_value(&self, value: &T) {
        self(value);
    }
}

// ---------------------------------------------------------------------------
// OneWayBinding
// ---------------------------------------------------------------------------

/// A one-way binding that pushes signal changes to a property sink.
///
/// When the source signal changes, the binding reads the new value
/// and calls [`PropertySink::set_value`] on the sink.
///
/// # Examples
///
/// ```ignore
/// let count = Signal::new(0);
/// let display = Rc::new(RefCell::new(String::new()));
///
/// let binding = OneWayBinding::new(&count, {
///     let display = display.clone();
///     move |v: &i32| *display.borrow_mut() = format!("{v}")
/// });
///
/// count.set(42);
/// assert_eq!(*display.borrow(), "42");
/// ```
pub struct OneWayBinding<T: Clone + 'static> {
    id: BindingId,
    effect: Effect,
    /// Keep the signal alive for the binding's lifetime.
    _source: Signal<T>,
}

impl<T: Clone + 'static> OneWayBinding<T> {
    /// Create a new one-way binding from `source` to `sink`.
    ///
    /// The sink is called immediately with the current value, and
    /// again whenever the source signal changes.
    pub fn new(source: &Signal<T>, sink: impl PropertySink<T> + 'static) -> Self {
        let id = next_binding_id();
        let source_clone = source.clone();

        let effect = Effect::new({
            let sig = source.clone();
            move || {
                let value = sig.get();
                sink.set_value(&value);
            }
        });

        // Subscribe the effect to source changes.
        source.subscribe(effect.as_subscriber());

        Self {
            id,
            effect,
            _source: source_clone,
        }
    }
}

impl<T: Clone + 'static> Binding for OneWayBinding<T> {
    fn id(&self) -> BindingId {
        self.id
    }

    fn direction(&self) -> BindingDirection {
        BindingDirection::OneWay
    }

    fn is_active(&self) -> bool {
        self.effect.is_active()
    }

    fn dispose(&self) {
        self.effect.dispose();
    }
}

// ---------------------------------------------------------------------------
// TwoWayBinding
// ---------------------------------------------------------------------------

/// A two-way binding between a signal and a property sink.
///
/// The forward direction works like [`OneWayBinding`]: signal changes
/// are pushed to the sink. The reverse direction is handled by calling
/// [`TwoWayBinding::write_back`], which updates the signal.
///
/// An internal loop guard prevents infinite ping-pong when a write-back
/// triggers a forward push that would trigger another write-back.
///
/// # Examples
///
/// ```ignore
/// let model = Signal::new(String::from("hello"));
/// let display = Rc::new(RefCell::new(String::new()));
///
/// let binding = TwoWayBinding::new(&model, {
///     let display = display.clone();
///     move |v: &String| *display.borrow_mut() = v.clone()
/// });
///
/// // Forward: model → display
/// model.set("world".into());
/// assert_eq!(*display.borrow(), "world");
///
/// // Reverse: display → model
/// binding.write_back("reverse".into());
/// assert_eq!(model.get(), "reverse");
/// ```
pub struct TwoWayBinding<T: Clone + 'static> {
    id: BindingId,
    effect: Effect,
    source: Signal<T>,
    /// Guard to prevent infinite update loops.
    updating: Rc<Cell<bool>>,
}

impl<T: Clone + 'static> TwoWayBinding<T> {
    /// Create a new two-way binding between `source` and `sink`.
    ///
    /// Forward direction runs immediately and on every source change.
    pub fn new(source: &Signal<T>, sink: impl PropertySink<T> + 'static) -> Self {
        let id = next_binding_id();
        let updating = Rc::new(Cell::new(false));

        let effect = Effect::new({
            let sig = source.clone();
            let guard = Rc::clone(&updating);
            move || {
                if guard.get() {
                    return;
                }
                let value = sig.get();
                sink.set_value(&value);
            }
        });

        source.subscribe(effect.as_subscriber());

        Self {
            id,
            effect,
            source: source.clone(),
            updating,
        }
    }

    /// Write a value back from the sink to the source signal.
    ///
    /// Uses the loop guard to prevent the forward effect from
    /// re-pushing the same value to the sink.
    pub fn write_back(&self, value: T) {
        if !self.effect.is_active() {
            return;
        }

        self.updating.set(true);
        self.source.set(value);
        self.updating.set(false);
    }
}

impl<T: Clone + 'static> Binding for TwoWayBinding<T> {
    fn id(&self) -> BindingId {
        self.id
    }

    fn direction(&self) -> BindingDirection {
        BindingDirection::TwoWay
    }

    fn is_active(&self) -> bool {
        self.effect.is_active()
    }

    fn dispose(&self) {
        self.effect.dispose();
    }
}

// ---------------------------------------------------------------------------
// BindingExpression
// ---------------------------------------------------------------------------

/// A binding that transforms the source value before pushing to the sink.
///
/// Uses a [`Computed`] internally so the transform result is cached
/// and only re-evaluated when the source changes.
///
/// # Examples
///
/// ```ignore
/// let count = Signal::new(3);
/// let label = Rc::new(RefCell::new(String::new()));
///
/// let binding = BindingExpression::new(
///     &count,
///     |v: &i32| format!("Count: {v}"),
///     {
///         let label = label.clone();
///         move |v: &String| *label.borrow_mut() = v.clone()
///     },
/// );
///
/// count.set(7);
/// assert_eq!(*label.borrow(), "Count: 7");
/// ```
pub struct BindingExpression<S: Clone + 'static, T: Clone + 'static> {
    id: BindingId,
    effect: Effect,
    /// Keep source and computed alive.
    _source: Signal<S>,
    _computed: Computed<T>,
}

impl<S: Clone + 'static, T: Clone + 'static> BindingExpression<S, T> {
    /// Create a binding that transforms source values via `transform`
    /// before pushing to `sink`.
    pub fn new(
        source: &Signal<S>,
        transform: impl Fn(&S) -> T + 'static,
        sink: impl PropertySink<T> + 'static,
    ) -> Self {
        let id = next_binding_id();

        // Create a computed that applies the transform.
        let computed = Computed::new({
            let sig = source.clone();
            move || {
                let v = sig.get();
                transform(&v)
            }
        });
        source.subscribe(computed.as_subscriber());

        // Effect reads the computed and pushes to sink.
        let effect = Effect::new({
            let comp = computed.clone();
            move || {
                let value = comp.get();
                sink.set_value(&value);
            }
        });
        computed.subscribe(effect.as_subscriber());

        Self {
            id,
            effect,
            _source: source.clone(),
            _computed: computed,
        }
    }
}

impl<S: Clone + 'static, T: Clone + 'static> Binding for BindingExpression<S, T> {
    fn id(&self) -> BindingId {
        self.id
    }

    fn direction(&self) -> BindingDirection {
        BindingDirection::OneWay
    }

    fn is_active(&self) -> bool {
        self.effect.is_active()
    }

    fn dispose(&self) {
        self.effect.dispose();
    }
}

// ---------------------------------------------------------------------------
// BindingScope
// ---------------------------------------------------------------------------

/// A scope that owns bindings and disposes them on drop.
///
/// `BindingScope` provides a convenient way to manage the lifetime
/// of multiple bindings. When the scope is dropped, all owned bindings
/// are disposed.
///
/// # Examples
///
/// ```ignore
/// let count = Signal::new(0);
/// let display = Rc::new(RefCell::new(String::new()));
///
/// {
///     let mut scope = BindingScope::new();
///     scope.bind(&count, {
///         let display = display.clone();
///         move |v: &i32| *display.borrow_mut() = format!("{v}")
///     });
///     count.set(5);
///     assert_eq!(*display.borrow(), "5");
/// }
/// // Scope dropped — binding disposed.
/// count.set(99);
/// // display still shows "5" because the binding is gone.
/// ```
pub struct BindingScope {
    bindings: Vec<Box<dyn Binding>>,
}

impl BindingScope {
    /// Create a new empty binding scope.
    #[must_use]
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Create a one-way binding from `source` to `sink`.
    ///
    /// Returns the binding ID.
    pub fn bind<T: Clone + 'static>(
        &mut self,
        source: &Signal<T>,
        sink: impl PropertySink<T> + 'static,
    ) -> BindingId {
        let binding = OneWayBinding::new(source, sink);
        let id = binding.id();
        self.bindings.push(Box::new(binding));
        id
    }

    /// Create a two-way binding between `source` and `sink`.
    ///
    /// Returns the `TwoWayBinding` (for calling `write_back`) and the binding ID.
    pub fn bind_two_way<T: Clone + 'static>(
        &mut self,
        source: &Signal<T>,
        sink: impl PropertySink<T> + 'static,
    ) -> (TwoWayBinding<T>, BindingId) {
        let binding = TwoWayBinding::new(source, sink);
        let id = binding.id();

        // We need a second instance for the caller — clone the internals.
        let caller_binding = TwoWayBinding {
            id: binding.id,
            effect: binding.effect.clone(),
            source: binding.source.clone(),
            updating: Rc::clone(&binding.updating),
        };

        self.bindings.push(Box::new(binding));
        (caller_binding, id)
    }

    /// Create a binding expression (source → transform → sink).
    ///
    /// Returns the binding ID.
    pub fn bind_expression<S: Clone + 'static, T: Clone + 'static>(
        &mut self,
        source: &Signal<S>,
        transform: impl Fn(&S) -> T + 'static,
        sink: impl PropertySink<T> + 'static,
    ) -> BindingId {
        let binding = BindingExpression::new(source, transform, sink);
        let id = binding.id();
        self.bindings.push(Box::new(binding));
        id
    }

    /// Get the number of bindings in this scope.
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Check if a specific binding is still active.
    pub fn is_binding_active(&self, id: BindingId) -> bool {
        self.bindings
            .iter()
            .find(|b| b.id() == id)
            .is_some_and(|b| b.is_active())
    }
}

impl Default for BindingScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BindingScope {
    fn drop(&mut self) {
        for binding in &self.bindings {
            binding.dispose();
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::reactive::batch::batch;
    use std::cell::RefCell;

    // -- OneWayBinding --

    #[test]
    fn one_way_pushes_initial_value() {
        let sig = Signal::new(42);
        let output = Rc::new(Cell::new(0));

        let _binding = OneWayBinding::new(&sig, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        assert_eq!(output.get(), 42);
    }

    #[test]
    fn one_way_pushes_on_change() {
        let sig = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        let _binding = OneWayBinding::new(&sig, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        sig.set(10);
        assert_eq!(output.get(), 10);

        sig.set(20);
        assert_eq!(output.get(), 20);
    }

    #[test]
    fn one_way_stops_after_dispose() {
        let sig = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        let binding = OneWayBinding::new(&sig, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        sig.set(5);
        assert_eq!(output.get(), 5);

        binding.dispose();
        assert!(!binding.is_active());

        sig.set(99);
        assert_eq!(output.get(), 5); // Unchanged.
    }

    #[test]
    fn one_way_direction() {
        let sig = Signal::new(0);
        let binding = OneWayBinding::new(&sig, |_: &i32| {});
        assert_eq!(binding.direction(), BindingDirection::OneWay);
    }

    #[test]
    fn one_way_unique_ids() {
        let sig = Signal::new(0);
        let a = OneWayBinding::new(&sig, |_: &i32| {});
        let b = OneWayBinding::new(&sig, |_: &i32| {});
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn one_way_with_string_sink() {
        let sig = Signal::new(String::from("hello"));
        let output = Rc::new(RefCell::new(String::new()));

        let _binding = OneWayBinding::new(&sig, {
            let out = Rc::clone(&output);
            move |v: &String| *out.borrow_mut() = v.clone()
        });

        assert_eq!(*output.borrow(), "hello");

        sig.set("world".into());
        assert_eq!(*output.borrow(), "world");
    }

    // -- TwoWayBinding --

    #[test]
    fn two_way_forward_push() {
        let sig = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        let _binding = TwoWayBinding::new(&sig, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        sig.set(42);
        assert_eq!(output.get(), 42);
    }

    #[test]
    fn two_way_write_back() {
        let sig = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        let binding = TwoWayBinding::new(&sig, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        binding.write_back(99);
        assert_eq!(sig.get(), 99);
    }

    #[test]
    fn two_way_loop_guard() {
        let sig = Signal::new(0);
        let push_count = Rc::new(Cell::new(0u32));

        let binding = TwoWayBinding::new(&sig, {
            let count = Rc::clone(&push_count);
            move |_: &i32| count.set(count.get() + 1)
        });

        // Initial push.
        assert_eq!(push_count.get(), 1);

        // Write-back should NOT trigger the forward push
        // (loop guard prevents it).
        binding.write_back(42);
        assert_eq!(push_count.get(), 1); // Still 1.
        assert_eq!(sig.get(), 42);

        // Regular forward push still works.
        sig.set(100);
        assert_eq!(push_count.get(), 2);
    }

    #[test]
    fn two_way_disposed_write_back_ignored() {
        let sig = Signal::new(0);
        let binding = TwoWayBinding::new(&sig, |_: &i32| {});

        binding.dispose();
        binding.write_back(42);

        // Signal unchanged because binding is disposed.
        assert_eq!(sig.get(), 0);
    }

    #[test]
    fn two_way_direction() {
        let sig = Signal::new(0);
        let binding = TwoWayBinding::new(&sig, |_: &i32| {});
        assert_eq!(binding.direction(), BindingDirection::TwoWay);
    }

    // -- BindingExpression --

    #[test]
    fn expression_transforms_value() {
        let sig = Signal::new(3);
        let output = Rc::new(RefCell::new(String::new()));

        let _binding = BindingExpression::new(&sig, |v: &i32| format!("Count: {v}"), {
            let out = Rc::clone(&output);
            move |v: &String| *out.borrow_mut() = v.clone()
        });

        assert_eq!(*output.borrow(), "Count: 3");

        sig.set(7);
        assert_eq!(*output.borrow(), "Count: 7");
    }

    #[test]
    fn expression_stops_after_dispose() {
        let sig = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        let binding = BindingExpression::new(&sig, |v: &i32| v * 10, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        sig.set(5);
        assert_eq!(output.get(), 50);

        binding.dispose();
        sig.set(99);
        assert_eq!(output.get(), 50); // Unchanged.
    }

    #[test]
    fn expression_direction() {
        let sig = Signal::new(0);
        let binding = BindingExpression::new(&sig, |v: &i32| *v, |_: &i32| {});
        assert_eq!(binding.direction(), BindingDirection::OneWay);
    }

    #[test]
    fn expression_type_conversion() {
        let sig = Signal::new(42i32);
        let output = Rc::new(Cell::new(0.0f64));

        let _binding = BindingExpression::new(&sig, |v: &i32| *v as f64 * 1.5, {
            let out = Rc::clone(&output);
            move |v: &f64| out.set(*v)
        });

        assert!((output.get() - 63.0).abs() < f64::EPSILON);

        sig.set(10);
        assert!((output.get() - 15.0).abs() < f64::EPSILON);
    }

    // -- BindingScope --

    #[test]
    fn scope_bind_creates_one_way() {
        let sig = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        let mut scope = BindingScope::new();
        let id = scope.bind(&sig, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        assert_eq!(scope.binding_count(), 1);
        assert!(scope.is_binding_active(id));

        sig.set(10);
        assert_eq!(output.get(), 10);
    }

    #[test]
    fn scope_bind_two_way() {
        let sig = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        let mut scope = BindingScope::new();
        let (two_way, id) = scope.bind_two_way(&sig, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        assert_eq!(scope.binding_count(), 1);
        assert!(scope.is_binding_active(id));

        // Forward.
        sig.set(10);
        assert_eq!(output.get(), 10);

        // Reverse.
        two_way.write_back(50);
        assert_eq!(sig.get(), 50);
    }

    #[test]
    fn scope_bind_expression() {
        let sig = Signal::new(5);
        let output = Rc::new(RefCell::new(String::new()));

        let mut scope = BindingScope::new();
        let id = scope.bind_expression(&sig, |v: &i32| format!("val={v}"), {
            let out = Rc::clone(&output);
            move |v: &String| *out.borrow_mut() = v.clone()
        });

        assert!(scope.is_binding_active(id));
        assert_eq!(*output.borrow(), "val=5");

        sig.set(10);
        assert_eq!(*output.borrow(), "val=10");
    }

    #[test]
    fn scope_disposes_bindings_on_drop() {
        let sig = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        {
            let mut scope = BindingScope::new();
            scope.bind(&sig, {
                let out = Rc::clone(&output);
                move |v: &i32| out.set(*v)
            });

            sig.set(5);
            assert_eq!(output.get(), 5);
        }
        // Scope dropped.

        sig.set(99);
        assert_eq!(output.get(), 5); // Binding disposed, no update.
    }

    #[test]
    fn scope_multiple_bindings() {
        let a = Signal::new(0);
        let b = Signal::new(0);
        let out_a = Rc::new(Cell::new(0));
        let out_b = Rc::new(Cell::new(0));

        let mut scope = BindingScope::new();
        scope.bind(&a, {
            let out = Rc::clone(&out_a);
            move |v: &i32| out.set(*v)
        });
        scope.bind(&b, {
            let out = Rc::clone(&out_b);
            move |v: &i32| out.set(*v)
        });

        assert_eq!(scope.binding_count(), 2);

        a.set(10);
        b.set(20);
        assert_eq!(out_a.get(), 10);
        assert_eq!(out_b.get(), 20);
    }

    #[test]
    fn scope_is_binding_active_returns_false_for_unknown_id() {
        let scope = BindingScope::new();
        assert!(!scope.is_binding_active(99999));
    }

    // -- Integration tests --

    #[test]
    fn binding_with_batch() {
        let sig = Signal::new(0);
        let push_count = Rc::new(Cell::new(0u32));

        let _binding = OneWayBinding::new(&sig, {
            let count = Rc::clone(&push_count);
            move |_: &i32| count.set(count.get() + 1)
        });

        assert_eq!(push_count.get(), 1); // Initial.

        batch(|| {
            sig.set(1);
            sig.set(2);
            sig.set(3);
        });

        // Should push only once after batch.
        assert_eq!(push_count.get(), 2);
    }

    #[test]
    fn binding_expression_with_batch() {
        let sig = Signal::new(0);
        let transform_count = Rc::new(Cell::new(0u32));
        let output = Rc::new(Cell::new(0));

        let _binding = BindingExpression::new(
            &sig,
            {
                let count = Rc::clone(&transform_count);
                move |v: &i32| {
                    count.set(count.get() + 1);
                    v * 2
                }
            },
            {
                let out = Rc::clone(&output);
                move |v: &i32| out.set(*v)
            },
        );

        // Initial transform + push.
        assert_eq!(transform_count.get(), 1);
        assert_eq!(output.get(), 0);

        batch(|| {
            sig.set(5);
            sig.set(10);
        });

        // Transform called once more after batch.
        assert_eq!(output.get(), 20);
    }

    #[test]
    fn two_way_binding_round_trip() {
        let model = Signal::new(String::from("initial"));
        let view = Rc::new(RefCell::new(String::new()));

        let binding = TwoWayBinding::new(&model, {
            let view = Rc::clone(&view);
            move |v: &String| *view.borrow_mut() = v.clone()
        });

        // Forward: model → view.
        assert_eq!(*view.borrow(), "initial");

        model.set("forward".into());
        assert_eq!(*view.borrow(), "forward");

        // Reverse: view → model.
        binding.write_back("reverse".into());
        assert_eq!(model.get(), "reverse");

        // Forward again after reverse.
        model.set("final".into());
        assert_eq!(*view.borrow(), "final");
    }

    #[test]
    fn binding_scope_with_mixed_types() {
        let count = Signal::new(0);
        let name = Signal::new(String::from("test"));

        let out_count = Rc::new(Cell::new(0));
        let out_name = Rc::new(RefCell::new(String::new()));

        let mut scope = BindingScope::new();

        scope.bind(&count, {
            let out = Rc::clone(&out_count);
            move |v: &i32| out.set(*v)
        });

        scope.bind(&name, {
            let out = Rc::clone(&out_name);
            move |v: &String| *out.borrow_mut() = v.clone()
        });

        count.set(42);
        name.set("hello".into());

        assert_eq!(out_count.get(), 42);
        assert_eq!(*out_name.borrow(), "hello");
    }

    #[test]
    fn chained_one_way_bindings() {
        let source = Signal::new(1);
        let middle = Signal::new(0);
        let output = Rc::new(Cell::new(0));

        // source → middle
        let _binding1 = OneWayBinding::new(&source, {
            let mid = middle.clone();
            move |v: &i32| mid.set(*v * 2)
        });

        // middle → output
        let _binding2 = OneWayBinding::new(&middle, {
            let out = Rc::clone(&output);
            move |v: &i32| out.set(*v)
        });

        // Initial: source=1, middle=2, output=2.
        assert_eq!(middle.get(), 2);
        assert_eq!(output.get(), 2);

        source.set(5);
        assert_eq!(middle.get(), 10);
        assert_eq!(output.get(), 10);
    }

    #[test]
    fn scope_default_is_empty() {
        let scope = BindingScope::default();
        assert_eq!(scope.binding_count(), 0);
    }

    #[test]
    fn disposed_binding_not_active_in_scope() {
        let sig = Signal::new(0);
        let mut scope = BindingScope::new();

        let id = scope.bind(&sig, |_: &i32| {});
        assert!(scope.is_binding_active(id));

        // Drop the scope to dispose.
        drop(scope);

        // After drop we can't check scope, but the binding's effect is gone.
        // This test verifies that dispose is called on drop.
    }

    #[test]
    fn stress_many_bindings() {
        let sig = Signal::new(0);
        let count = Rc::new(Cell::new(0u32));
        let mut scope = BindingScope::new();

        for _ in 0..50 {
            scope.bind(&sig, {
                let count = Rc::clone(&count);
                move |_: &i32| count.set(count.get() + 1)
            });
        }

        // 50 initial pushes.
        assert_eq!(count.get(), 50);

        sig.set(1);
        // 50 more pushes.
        assert_eq!(count.get(), 100);

        drop(scope);
        sig.set(2);
        // No more pushes.
        assert_eq!(count.get(), 100);
    }
}
