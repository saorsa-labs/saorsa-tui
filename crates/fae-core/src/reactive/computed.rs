//! Computed (derived) reactive values.
//!
//! A [`Computed<T>`] value is derived from one or more signals. It
//! caches its result and only re-evaluates when a dependency changes.
//! Reading a computed value inside a tracking context registers it
//! as a dependency, enabling computed-of-computed chains.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use super::context::{self, SubscriberId};
use super::signal::Subscriber;

/// A computed (derived) reactive value.
///
/// Computed values lazily re-evaluate when their signal dependencies
/// change. They are read-only — there is no `set` method.
///
/// # Examples
///
/// ```ignore
/// let count = Signal::new(3);
/// let doubled = Computed::new({
///     let count = count.clone();
///     move || count.get() * 2
/// });
/// assert_eq!(doubled.get(), 6);
/// count.set(5);
/// assert_eq!(doubled.get(), 10);
/// ```
pub struct Computed<T>(Rc<ComputedInner<T>>);

struct ComputedInner<T> {
    /// Cached computed value.
    value: RefCell<Option<T>>,
    /// The derivation function.
    compute_fn: RefCell<Box<dyn Fn() -> T>>,
    /// Whether the cached value is stale.
    dirty: Cell<bool>,
    /// Subscriber ID for this computed value.
    sub_id: SubscriberId,
    /// Subscribers to notify when this computed value changes.
    subscribers: RefCell<Vec<Weak<dyn Subscriber>>>,
}

impl<T: Clone + 'static> Computed<T> {
    /// Create a new computed value from a derivation function.
    ///
    /// The function is called immediately to compute the initial value
    /// and discover dependencies.
    #[must_use]
    pub fn new(f: impl Fn() -> T + 'static) -> Self {
        let sub_id = context::next_subscriber_id();

        let inner = Rc::new(ComputedInner {
            value: RefCell::new(None),
            compute_fn: RefCell::new(Box::new(f)),
            dirty: Cell::new(true),
            sub_id,
            subscribers: RefCell::new(Vec::new()),
        });

        // Compute initial value.
        inner.evaluate();

        Computed(inner)
    }

    /// Read the computed value, re-evaluating if dirty.
    ///
    /// Records this computed as a dependency if inside a tracking context.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        if self.0.dirty.get() {
            self.0.evaluate();
        }

        // Record as dependency for outer tracking context.
        context::record_read(self.signal_id());

        match self.0.value.borrow().as_ref() {
            Some(v) => v.clone(),
            None => {
                // Should not happen after evaluate(), but handle gracefully.
                self.0.evaluate();
                match self.0.value.borrow().as_ref() {
                    Some(v) => v.clone(),
                    None => {
                        let f = self.0.compute_fn.borrow();
                        f()
                    }
                }
            }
        }
    }

    /// Borrow the computed value and apply a function.
    ///
    /// Records this computed as a dependency if inside a tracking context.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        if self.0.dirty.get() {
            self.0.evaluate();
        }

        context::record_read(self.signal_id());

        match self.0.value.borrow().as_ref() {
            Some(v) => f(v),
            None => {
                self.0.evaluate();
                match self.0.value.borrow().as_ref() {
                    Some(v) => f(v),
                    None => {
                        let compute = self.0.compute_fn.borrow();
                        let val = compute();
                        f(&val)
                    }
                }
            }
        }
    }

    /// Get this computed value's subscriber ID.
    pub fn subscriber_id(&self) -> SubscriberId {
        self.0.sub_id
    }

    /// Subscribe to be notified when this computed value changes.
    pub fn subscribe(&self, subscriber: Weak<dyn Subscriber>) {
        self.0.subscribers.borrow_mut().push(subscriber);
    }

    /// Check if the computed value needs re-evaluation.
    pub fn is_dirty(&self) -> bool {
        self.0.dirty.get()
    }

    /// Get a weak reference suitable for subscribing to signals.
    ///
    /// Use this to register the computed value as a subscriber
    /// on the signals it depends on.
    pub fn as_subscriber(&self) -> Weak<dyn Subscriber> {
        Rc::downgrade(&self.0) as Weak<dyn Subscriber>
    }

    /// Synthetic signal ID for dependency tracking of computed values.
    fn signal_id(&self) -> context::SignalId {
        context::synthetic_signal_id(self.0.sub_id)
    }
}

impl<T> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Computed(Rc::clone(&self.0))
    }
}

impl<T: Clone + 'static> ComputedInner<T> {
    /// Evaluate the compute function, cache the result, and update dependencies.
    fn evaluate(&self) {
        context::start_tracking(self.sub_id);

        let new_value = {
            let f = self.compute_fn.borrow();
            f()
        };

        let _deps = context::stop_tracking();

        *self.value.borrow_mut() = Some(new_value);
        self.dirty.set(false);
    }
}

impl<T: Clone + 'static> Subscriber for ComputedInner<T> {
    fn notify(&self) {
        self.dirty.set(true);

        // Propagate notification to our subscribers.
        let to_notify: Vec<Rc<dyn Subscriber>> = {
            let subs = self.subscribers.borrow();
            subs.iter().filter_map(|w| w.upgrade()).collect()
        };

        for sub in &to_notify {
            sub.notify();
        }

        // Prune dead weak references.
        self.subscribers
            .borrow_mut()
            .retain(|w| w.strong_count() > 0);
    }

    fn id(&self) -> SubscriberId {
        self.sub_id
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::reactive::Signal;
    use std::cell::Cell;

    /// Mock subscriber that counts notifications.
    struct MockSub {
        sub_id: SubscriberId,
        count: Rc<Cell<u32>>,
    }

    impl MockSub {
        fn new() -> (Rc<Self>, Rc<Cell<u32>>) {
            let count = Rc::new(Cell::new(0));
            let sub = Rc::new(Self {
                sub_id: context::next_subscriber_id(),
                count: Rc::clone(&count),
            });
            (sub, count)
        }
    }

    impl Subscriber for MockSub {
        fn notify(&self) {
            self.count.set(self.count.get() + 1);
        }
        fn id(&self) -> SubscriberId {
            self.sub_id
        }
    }

    #[test]
    fn computed_from_single_signal() {
        let sig = Signal::new(3);

        let doubled = Computed::new({
            let sig = sig.clone();
            move || sig.get() * 2
        });

        assert_eq!(doubled.get(), 6);
    }

    #[test]
    fn computed_updates_when_signal_changes() {
        let sig = Signal::new(3);
        let doubled = Computed::new({
            let sig = sig.clone();
            move || sig.get() * 2
        });

        // Subscribe computed to the signal.
        sig.subscribe(doubled.as_subscriber());

        assert_eq!(doubled.get(), 6);
        sig.set(5);
        assert_eq!(doubled.get(), 10);
    }

    #[test]
    fn computed_from_multiple_signals() {
        let a = Signal::new(2);
        let b = Signal::new(3);

        let sum = Computed::new({
            let a = a.clone();
            let b = b.clone();
            move || a.get() + b.get()
        });

        a.subscribe(sum.as_subscriber());
        b.subscribe(sum.as_subscriber());

        assert_eq!(sum.get(), 5);
        a.set(10);
        assert_eq!(sum.get(), 13);
        b.set(7);
        assert_eq!(sum.get(), 17);
    }

    #[test]
    fn computed_is_lazy() {
        let call_count = Rc::new(Cell::new(0u32));

        let sig = Signal::new(1);
        let computed = Computed::new({
            let sig = sig.clone();
            let count = Rc::clone(&call_count);
            move || {
                count.set(count.get() + 1);
                sig.get() * 2
            }
        });

        // Initial evaluation happened during new().
        assert_eq!(call_count.get(), 1);

        // Reading without change shouldn't re-evaluate.
        assert_eq!(computed.get(), 2);
        assert_eq!(call_count.get(), 1);

        // Mark dirty and re-read.
        sig.subscribe(computed.as_subscriber());
        sig.set(5);
        assert!(computed.is_dirty());

        // Now read triggers re-evaluation.
        assert_eq!(computed.get(), 10);
        assert_eq!(call_count.get(), 2);
    }

    #[test]
    fn computed_chain() {
        let base = Signal::new(2);

        let doubled = Computed::new({
            let base = base.clone();
            move || base.get() * 2
        });

        let quadrupled = Computed::new({
            let doubled = doubled.clone();
            move || doubled.get() * 2
        });

        base.subscribe(doubled.as_subscriber());
        doubled.subscribe(Rc::downgrade(&quadrupled.0) as Weak<dyn Subscriber>);

        assert_eq!(quadrupled.get(), 8);

        base.set(3);
        assert_eq!(quadrupled.get(), 12);
    }

    #[test]
    fn computed_with_borrow() {
        let sig = Signal::new(String::from("hello world"));
        let computed = Computed::new({
            let sig = sig.clone();
            move || sig.get().to_uppercase()
        });

        let len = computed.with(|s: &String| s.len());
        assert_eq!(len, 11);
    }

    #[test]
    fn computed_subscriber_notification() {
        let sig = Signal::new(1);
        let computed = Computed::new({
            let sig = sig.clone();
            move || sig.get() * 10
        });

        sig.subscribe(computed.as_subscriber());

        let (mock, count) = MockSub::new();
        computed.subscribe(Rc::downgrade(&mock) as Weak<dyn Subscriber>);

        sig.set(2);
        assert_eq!(count.get(), 1);

        sig.set(3);
        assert_eq!(count.get(), 2);

        drop(mock);
    }

    #[test]
    fn computed_no_deps_never_dirty() {
        let computed = Computed::new(|| 42);
        assert!(!computed.is_dirty());
        assert_eq!(computed.get(), 42);
        assert!(!computed.is_dirty());
    }

    #[test]
    fn computed_clone_shares_state() {
        let sig = Signal::new(1);
        let computed = Computed::new({
            let sig = sig.clone();
            move || sig.get() + 100
        });

        let clone = computed.clone();
        assert_eq!(clone.get(), 101);
        assert_eq!(computed.subscriber_id(), clone.subscriber_id());
    }

    #[test]
    fn computed_get_records_dependency() {
        let sig = Signal::new(1);
        let computed = Computed::new({
            let sig = sig.clone();
            move || sig.get() * 2
        });

        let outer_sub = context::next_subscriber_id();
        context::start_tracking(outer_sub);
        let _ = computed.get();
        let deps = context::stop_tracking();

        // Should record the computed as a dependency.
        assert!(!deps.is_empty());
    }
}
