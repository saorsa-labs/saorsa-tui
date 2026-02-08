//! Batched updates for coalescing multiple signal changes.
//!
//! The [`batch`] function groups signal changes so that subscribers
//! are only notified once, after all changes complete. This avoids
//! redundant re-evaluations when multiple signals change together.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use super::context::SubscriberId;
use super::signal::Subscriber;

thread_local! {
    /// Nesting depth of batch operations.
    static BATCH_DEPTH: Cell<u32> = const { Cell::new(0) };
    /// Pending subscribers to notify after outermost batch completes.
    static PENDING: RefCell<Vec<Rc<dyn Subscriber>>> = const { RefCell::new(Vec::new()) };
    /// Set of subscriber IDs already queued (for deduplication).
    static PENDING_IDS: RefCell<HashSet<SubscriberId>> = RefCell::new(HashSet::new());
}

/// Check if we are currently inside a batch.
pub fn is_batching() -> bool {
    BATCH_DEPTH.with(|d| d.get() > 0)
}

/// Queue a subscriber for notification after the current batch completes.
///
/// If we are not in a batch, this returns `false` and the caller
/// should notify immediately. If we are in a batch, the subscriber
/// is queued and this returns `true`.
pub fn queue_subscriber(subscriber: &Rc<dyn Subscriber>) -> bool {
    if !is_batching() {
        return false;
    }

    let id = subscriber.id();
    PENDING_IDS.with(|ids| {
        let mut ids = ids.borrow_mut();
        if ids.insert(id) {
            PENDING.with(|pending| {
                pending.borrow_mut().push(Rc::clone(subscriber));
            });
        }
    });

    true
}

/// Run a closure with batched signal notifications.
///
/// All signal changes within the closure are batched — subscribers
/// are only notified once after the closure returns. Batches can
/// be nested; notifications only fire after the outermost batch.
///
/// # Examples
///
/// ```ignore
/// let a = Signal::new(0);
/// let b = Signal::new(0);
///
/// batch(|| {
///     a.set(1);
///     b.set(2);
///     // Effects are NOT triggered yet.
/// });
/// // Effects fire once here, seeing a=1 and b=2.
/// ```
pub fn batch(f: impl FnOnce()) {
    BATCH_DEPTH.with(|d| d.set(d.get() + 1));

    f();

    BATCH_DEPTH.with(|d| {
        let depth = d.get().saturating_sub(1);
        d.set(depth);

        if depth == 0 {
            flush_pending();
        }
    });
}

/// Flush all pending subscribers.
fn flush_pending() {
    let subscribers: Vec<Rc<dyn Subscriber>> =
        PENDING.with(|pending| pending.borrow_mut().drain(..).collect());

    PENDING_IDS.with(|ids| ids.borrow_mut().clear());

    for sub in &subscribers {
        sub.notify();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::reactive::effect::Effect;
    use crate::reactive::signal::Signal;
    use std::cell::Cell;

    #[test]
    fn without_batch_effect_runs_on_every_set() {
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

        sig.set(1);
        sig.set(2);
        sig.set(3);
        // Initial + 3 changes = 4
        assert_eq!(count.get(), 4);
    }

    #[test]
    fn with_batch_effect_runs_once() {
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

        batch(|| {
            sig.set(1);
            sig.set(2);
            sig.set(3);
        });

        // Should run only once after batch (seeing value 3).
        assert_eq!(count.get(), 2);
        assert_eq!(sig.get(), 3);
    }

    #[test]
    fn nested_batch_effects_run_after_outermost() {
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

        assert_eq!(count.get(), 1);

        batch(|| {
            sig.set(1);
            batch(|| {
                sig.set(2);
            });
            // Inner batch returned, but outer batch still active.
            assert_eq!(count.get(), 1); // Still just initial run.
        });

        // Only now effects fire.
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn batch_with_multiple_signals() {
        let a = Signal::new(0);
        let b = Signal::new(0);
        let count = Rc::new(Cell::new(0u32));

        let effect = Effect::new({
            let a = a.clone();
            let b = b.clone();
            let count = Rc::clone(&count);
            move || {
                let _ = a.get() + b.get();
                count.set(count.get() + 1);
            }
        });
        a.subscribe(effect.as_subscriber());
        b.subscribe(effect.as_subscriber());

        assert_eq!(count.get(), 1);

        batch(|| {
            a.set(1);
            b.set(2);
        });

        // Effect runs once (deduplicated).
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn empty_batch_no_spurious_notifications() {
        let count = Rc::new(Cell::new(0u32));
        let sig = Signal::new(0);

        let effect = Effect::new({
            let sig = sig.clone();
            let count = Rc::clone(&count);
            move || {
                let _ = sig.get();
                count.set(count.get() + 1);
            }
        });
        sig.subscribe(effect.as_subscriber());

        assert_eq!(count.get(), 1);

        batch(|| {
            // No changes.
        });

        assert_eq!(count.get(), 1);
    }

    #[test]
    fn is_batching_flag() {
        assert!(!is_batching());
        batch(|| {
            assert!(is_batching());
        });
        assert!(!is_batching());
    }

    #[test]
    fn batch_deduplicates_subscribers() {
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

        assert_eq!(count.get(), 1);

        batch(|| {
            // Multiple sets to same signal — subscriber queued once.
            sig.set(1);
            sig.set(2);
            sig.set(3);
        });

        // Effect runs exactly once after batch.
        assert_eq!(count.get(), 2);
    }
}
