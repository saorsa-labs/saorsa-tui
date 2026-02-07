//! Reactive effects — side effects that re-run when dependencies change.
//!
//! An [`Effect`] runs a closure immediately and re-runs it whenever
//! any of its signal dependencies change. Unlike [`super::computed::Computed`], effects
//! are eager — they run immediately on notification rather than lazily
//! on read.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::context::{self, SubscriberId};
use super::signal::Subscriber;

/// A reactive effect that re-runs when its dependencies change.
///
/// Effects are eager: when a dependency signal changes, the effect
/// closure is re-run immediately. The effect also runs once on
/// creation to discover its initial dependencies.
///
/// # Examples
///
/// ```ignore
/// let count = Signal::new(0);
/// let log = Rc::new(RefCell::new(Vec::new()));
///
/// let effect = Effect::new({
///     let count = count.clone();
///     let log = log.clone();
///     move || {
///         log.borrow_mut().push(count.get());
///     }
/// });
///
/// count.set(1); // effect re-runs, logs [0, 1]
/// ```
pub struct Effect(Rc<EffectInner>);

struct EffectInner {
    /// The effect closure. Uses `RefCell` to allow re-borrowing during notify.
    effect_fn: RefCell<Box<dyn FnMut()>>,
    /// Subscriber ID for dependency tracking.
    sub_id: SubscriberId,
    /// Whether this effect is still active.
    active: Cell<bool>,
}

impl Effect {
    /// Create a new effect that runs the given closure.
    ///
    /// The closure is called immediately to discover dependencies.
    /// It will be re-called whenever any signal read during its
    /// execution changes.
    #[must_use]
    pub fn new(f: impl FnMut() + 'static) -> Self {
        let sub_id = context::next_subscriber_id();

        let inner = Rc::new(EffectInner {
            effect_fn: RefCell::new(Box::new(f)),
            sub_id,
            active: Cell::new(true),
        });

        // Run immediately to discover dependencies.
        inner.run();

        Effect(inner)
    }

    /// Check if this effect is still active.
    pub fn is_active(&self) -> bool {
        self.0.active.get()
    }

    /// Permanently deactivate this effect.
    ///
    /// Once disposed, the effect will not run again even if its
    /// dependencies change.
    pub fn dispose(&self) {
        self.0.active.set(false);
    }

    /// Get this effect's subscriber ID.
    pub fn subscriber_id(&self) -> SubscriberId {
        self.0.sub_id
    }

    /// Get a weak reference suitable for subscribing to signals.
    pub fn as_subscriber(&self) -> std::rc::Weak<dyn Subscriber> {
        Rc::downgrade(&self.0) as std::rc::Weak<dyn Subscriber>
    }
}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Effect(Rc::clone(&self.0))
    }
}

impl EffectInner {
    /// Run the effect closure within a tracking context.
    fn run(&self) {
        if !self.active.get() {
            return;
        }

        context::start_tracking(self.sub_id);

        {
            let mut f = self.effect_fn.borrow_mut();
            f();
        }

        let _deps = context::stop_tracking();
    }
}

impl Subscriber for EffectInner {
    fn notify(&self) {
        self.run();
    }

    fn id(&self) -> SubscriberId {
        self.sub_id
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::reactive::signal::Signal;
    use std::cell::Cell;

    #[test]
    fn effect_runs_immediately() {
        let ran = Rc::new(Cell::new(false));
        let _effect = Effect::new({
            let ran = Rc::clone(&ran);
            move || {
                ran.set(true);
            }
        });
        assert!(ran.get());
    }

    #[test]
    fn effect_reruns_on_signal_change() {
        let sig = Signal::new(0);
        let log: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));

        let effect = Effect::new({
            let sig = sig.clone();
            let log = Rc::clone(&log);
            move || {
                log.borrow_mut().push(sig.get());
            }
        });

        // Subscribe effect to signal.
        sig.subscribe(effect.as_subscriber());

        sig.set(1);
        sig.set(2);

        assert_eq!(*log.borrow(), vec![0, 1, 2]);
    }

    #[test]
    fn effect_tracks_multiple_signals() {
        let a = Signal::new(1);
        let b = Signal::new(10);
        let sum_log: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));

        let effect = Effect::new({
            let a = a.clone();
            let b = b.clone();
            let log = Rc::clone(&sum_log);
            move || {
                log.borrow_mut().push(a.get() + b.get());
            }
        });

        a.subscribe(effect.as_subscriber());
        b.subscribe(effect.as_subscriber());

        a.set(2);
        b.set(20);

        // Initial: 11, after a=2: 12, after b=20: 22
        assert_eq!(*sum_log.borrow(), vec![11, 12, 22]);
    }

    #[test]
    fn disposed_effect_does_not_run() {
        let sig = Signal::new(0);
        let count = Rc::new(Cell::new(0u32));

        let effect = Effect::new({
            let sig = sig.clone();
            let count = Rc::clone(&count);
            move || {
                let _ = sig.get();
                count.set(count.get() + 1);
            }
        });

        sig.subscribe(effect.as_subscriber());

        assert_eq!(count.get(), 1); // Initial run.

        effect.dispose();
        assert!(!effect.is_active());

        sig.set(1);
        assert_eq!(count.get(), 1); // Should NOT have run again.
    }

    #[test]
    fn effect_with_no_deps_runs_once() {
        let count = Rc::new(Cell::new(0u32));
        let _effect = Effect::new({
            let count = Rc::clone(&count);
            move || {
                count.set(count.get() + 1);
            }
        });
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn effect_reads_computed() {
        use crate::reactive::Computed;

        let sig = Signal::new(5);
        let doubled = Computed::new({
            let sig = sig.clone();
            move || sig.get() * 2
        });

        sig.subscribe(doubled.as_subscriber());

        let log: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
        let effect = Effect::new({
            let doubled = doubled.clone();
            let log = Rc::clone(&log);
            move || {
                log.borrow_mut().push(doubled.get());
            }
        });

        // Subscribe effect to computed's notifications.
        doubled.subscribe(effect.as_subscriber());

        assert_eq!(*log.borrow(), vec![10]);

        sig.set(7);
        assert_eq!(*log.borrow(), vec![10, 14]);
    }

    #[test]
    fn multiple_effects_on_same_signal() {
        let sig = Signal::new(0);
        let count_a = Rc::new(Cell::new(0u32));
        let count_b = Rc::new(Cell::new(0u32));

        let effect_a = Effect::new({
            let sig = sig.clone();
            let count = Rc::clone(&count_a);
            move || {
                let _ = sig.get();
                count.set(count.get() + 1);
            }
        });

        let effect_b = Effect::new({
            let sig = sig.clone();
            let count = Rc::clone(&count_b);
            move || {
                let _ = sig.get();
                count.set(count.get() + 1);
            }
        });

        sig.subscribe(effect_a.as_subscriber());
        sig.subscribe(effect_b.as_subscriber());

        sig.set(1);
        assert_eq!(count_a.get(), 2); // Initial + 1 change.
        assert_eq!(count_b.get(), 2);
    }

    #[test]
    fn effect_is_active_initially() {
        let effect = Effect::new(|| {});
        assert!(effect.is_active());
    }

    #[test]
    fn effect_clone_shares_state() {
        let effect = Effect::new(|| {});
        let clone = effect.clone();
        assert_eq!(effect.subscriber_id(), clone.subscriber_id());
        effect.dispose();
        assert!(!clone.is_active());
    }
}
