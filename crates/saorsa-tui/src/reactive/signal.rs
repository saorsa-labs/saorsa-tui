//! Reactive signal — a mutable value with automatic change notification.
//!
//! `Signal<T>` is the fundamental reactive primitive. When its value
//! changes, all registered subscribers are notified. When read inside
//! a tracking context, the dependency is automatically recorded.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use super::context::{self, SignalId, SubscriberId};

/// A subscriber that receives notifications when a signal changes.
pub trait Subscriber {
    /// Called when a dependency signal has been modified.
    fn notify(&self);

    /// Return this subscriber's unique identifier.
    fn id(&self) -> SubscriberId;
}

/// A reactive signal holding a value of type `T`.
///
/// Signals are cheaply cloneable — cloning produces another handle
/// to the same underlying value. Changes to one handle are visible
/// through all clones.
///
/// # Examples
///
/// ```ignore
/// let count = Signal::new(0);
/// assert_eq!(count.get(), 0);
/// count.set(5);
/// assert_eq!(count.get(), 5);
/// ```
pub struct Signal<T>(Rc<RefCell<SignalInner<T>>>);

struct SignalInner<T> {
    value: T,
    id: SignalId,
    subscribers: Vec<Weak<dyn Subscriber>>,
}

impl<T> Signal<T> {
    /// Create a new signal with the given initial value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Signal(Rc::new(RefCell::new(SignalInner {
            value,
            id: context::next_signal_id(),
            subscribers: Vec::new(),
        })))
    }

    /// Get the signal's unique identifier.
    pub fn id(&self) -> SignalId {
        self.0.borrow().id
    }

    /// Read the value, recording a dependency in the active tracking context.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        let inner = self.0.borrow();
        context::record_read(inner.id);
        inner.value.clone()
    }

    /// Read the value without recording a dependency.
    pub fn get_untracked(&self) -> T
    where
        T: Clone,
    {
        self.0.borrow().value.clone()
    }

    /// Borrow the value and apply a function, recording a dependency.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let inner = self.0.borrow();
        context::record_read(inner.id);
        f(&inner.value)
    }

    /// Borrow the value and apply a function without recording a dependency.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let inner = self.0.borrow();
        f(&inner.value)
    }

    /// Set a new value and notify all subscribers.
    pub fn set(&self, value: T) {
        {
            self.0.borrow_mut().value = value;
        }
        self.notify_subscribers();
    }

    /// Update the value in place and notify all subscribers.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            f(&mut self.0.borrow_mut().value);
        }
        self.notify_subscribers();
    }

    /// Register a subscriber to be notified when this signal changes.
    pub fn subscribe(&self, subscriber: Weak<dyn Subscriber>) {
        self.0.borrow_mut().subscribers.push(subscriber);
    }

    /// Notify all live subscribers and prune dead weak references.
    ///
    /// If batching is active, subscribers are queued for later notification
    /// rather than being notified immediately.
    fn notify_subscribers(&self) {
        // Collect live subscribers to notify (avoid holding borrow during notify).
        let to_notify: Vec<Rc<dyn Subscriber>> = {
            let inner = self.0.borrow();
            inner
                .subscribers
                .iter()
                .filter_map(|w| w.upgrade())
                .collect()
        };

        // Notify each subscriber (or queue if batching).
        for sub in &to_notify {
            if !super::batch::queue_subscriber(sub) {
                sub.notify();
            }
        }

        // Prune dead weak references.
        self.0
            .borrow_mut()
            .subscribers
            .retain(|w| w.strong_count() > 0);
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal(Rc::clone(&self.0))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// Mock subscriber that counts notifications.
    struct MockSubscriber {
        sub_id: SubscriberId,
        count: Rc<Cell<u32>>,
    }

    impl MockSubscriber {
        fn new() -> (Rc<Self>, Rc<Cell<u32>>) {
            let count = Rc::new(Cell::new(0));
            let sub = Rc::new(Self {
                sub_id: context::next_subscriber_id(),
                count: Rc::clone(&count),
            });
            (sub, count)
        }
    }

    impl Subscriber for MockSubscriber {
        fn notify(&self) {
            self.count.set(self.count.get() + 1);
        }

        fn id(&self) -> SubscriberId {
            self.sub_id
        }
    }

    #[test]
    fn new_and_get_roundtrip() {
        let sig = Signal::new(42);
        assert_eq!(sig.get(), 42);
    }

    #[test]
    fn set_changes_value() {
        let sig = Signal::new(10);
        sig.set(20);
        assert_eq!(sig.get(), 20);
    }

    #[test]
    fn update_modifies_in_place() {
        let sig = Signal::new(vec![1, 2, 3]);
        sig.update(|v| v.push(4));
        assert_eq!(sig.get(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn with_borrows_without_clone() {
        let sig = Signal::new(String::from("hello"));
        let len = sig.with(|s| s.len());
        assert_eq!(len, 5);
    }

    #[test]
    fn get_untracked_returns_value() {
        let sig = Signal::new(99);
        assert_eq!(sig.get_untracked(), 99);
    }

    #[test]
    fn with_untracked_borrows() {
        let sig = Signal::new(String::from("test"));
        let upper = sig.with_untracked(|s| s.to_uppercase());
        assert_eq!(upper, "TEST");
    }

    #[test]
    fn clone_shares_state() {
        let sig = Signal::new(1);
        let sig2 = sig.clone();
        sig.set(2);
        assert_eq!(sig2.get(), 2);
    }

    #[test]
    fn id_is_unique_per_signal() {
        let a = Signal::new(0);
        let b = Signal::new(0);
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn get_inside_tracking_records_dependency() {
        let sig = Signal::new(5);
        let sub_id = context::next_subscriber_id();

        context::start_tracking(sub_id);
        let _ = sig.get();
        let deps = context::stop_tracking();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], sig.id());
    }

    #[test]
    fn get_untracked_inside_tracking_does_not_record() {
        let sig = Signal::new(5);
        let sub_id = context::next_subscriber_id();

        context::start_tracking(sub_id);
        let _ = sig.get_untracked();
        let deps = context::stop_tracking();

        assert!(deps.is_empty());
    }

    #[test]
    fn subscriber_receives_notification_on_set() {
        let sig = Signal::new(0);
        let (sub, count) = MockSubscriber::new();

        sig.subscribe(Rc::downgrade(&sub) as Weak<dyn Subscriber>);

        assert_eq!(count.get(), 0);
        sig.set(1);
        assert_eq!(count.get(), 1);
        sig.set(2);
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn dead_subscriber_is_pruned() {
        let sig = Signal::new(0);
        let (sub, count) = MockSubscriber::new();

        sig.subscribe(Rc::downgrade(&sub) as Weak<dyn Subscriber>);
        sig.set(1);
        assert_eq!(count.get(), 1);

        // Drop the strong reference — subscriber is now dead.
        drop(sub);

        // This should prune the dead subscriber and not panic.
        sig.set(2);
        // Count stays at 1 because the subscriber was dropped.
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn multiple_subscribers_all_notified() {
        let sig = Signal::new(0);
        let (sub_a, count_a) = MockSubscriber::new();
        let (sub_b, count_b) = MockSubscriber::new();

        sig.subscribe(Rc::downgrade(&sub_a) as Weak<dyn Subscriber>);
        sig.subscribe(Rc::downgrade(&sub_b) as Weak<dyn Subscriber>);

        sig.set(1);
        assert_eq!(count_a.get(), 1);
        assert_eq!(count_b.get(), 1);

        // Keep subscribers alive for the test.
        drop(sub_a);
        drop(sub_b);
    }

    #[test]
    fn update_notifies_subscribers() {
        let sig = Signal::new(0);
        let (sub, count) = MockSubscriber::new();

        sig.subscribe(Rc::downgrade(&sub) as Weak<dyn Subscriber>);

        sig.update(|v| *v += 10);
        assert_eq!(count.get(), 1);
        assert_eq!(sig.get(), 10);

        drop(sub);
    }
}
