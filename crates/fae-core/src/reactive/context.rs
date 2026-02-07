//! Dependency tracking context for the reactive system.
//!
//! Uses thread-local storage to track which signals are read
//! during evaluation of computed values or effects.

use std::cell::RefCell;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for a signal instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SignalId(u64);

impl fmt::Display for SignalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Signal({})", self.0)
    }
}

/// Unique identifier for a subscriber (computed value or effect).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SubscriberId(u64);

impl fmt::Display for SubscriberId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Subscriber({})", self.0)
    }
}

static SIGNAL_COUNTER: AtomicU64 = AtomicU64::new(1);
static SUBSCRIBER_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique signal ID.
pub fn next_signal_id() -> SignalId {
    SignalId(SIGNAL_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Generate a unique subscriber ID.
pub fn next_subscriber_id() -> SubscriberId {
    SubscriberId(SUBSCRIBER_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Active tracking scope that collects signal reads.
struct TrackingScope {
    /// The subscriber that is currently evaluating.
    _subscriber_id: SubscriberId,
    /// Signals that were read during this tracking scope.
    dependencies: Vec<SignalId>,
}

thread_local! {
    static TRACKING: RefCell<Option<TrackingScope>> = const { RefCell::new(None) };
}

/// Begin tracking signal reads for the given subscriber.
///
/// Any previous tracking scope is replaced (this is intentional
/// for re-evaluation of computed values).
pub fn start_tracking(id: SubscriberId) {
    TRACKING.with(|t| {
        *t.borrow_mut() = Some(TrackingScope {
            _subscriber_id: id,
            dependencies: Vec::new(),
        });
    });
}

/// End tracking and return the collected signal dependencies.
///
/// Returns an empty vec if no tracking was active.
pub fn stop_tracking() -> Vec<SignalId> {
    TRACKING.with(|t| {
        t.borrow_mut()
            .take()
            .map(|scope| scope.dependencies)
            .unwrap_or_default()
    })
}

/// Record that a signal was read in the current tracking context.
///
/// If no tracking is active, this is a no-op.
pub fn record_read(signal_id: SignalId) {
    TRACKING.with(|t| {
        if let Some(scope) = t.borrow_mut().as_mut() {
            scope.dependencies.push(signal_id);
        }
    });
}

/// Check whether dependency tracking is currently active.
pub fn is_tracking() -> bool {
    TRACKING.with(|t| t.borrow().is_some())
}

/// Create a synthetic signal ID from a subscriber ID.
///
/// This allows computed values to participate in the tracking system
/// as if they were signals, enabling computed-of-computed chains.
/// The ID is deterministic for a given subscriber.
pub fn synthetic_signal_id(sub_id: SubscriberId) -> SignalId {
    // Use a high bit to avoid collision with real signal IDs.
    SignalId(sub_id.0 | (1 << 63))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn signal_id_uniqueness() {
        let a = next_signal_id();
        let b = next_signal_id();
        assert_ne!(a, b);
    }

    #[test]
    fn subscriber_id_uniqueness() {
        let a = next_subscriber_id();
        let b = next_subscriber_id();
        assert_ne!(a, b);
    }

    #[test]
    fn signal_id_display() {
        let id = SignalId(42);
        assert_eq!(format!("{id}"), "Signal(42)");
    }

    #[test]
    fn subscriber_id_display() {
        let id = SubscriberId(99);
        assert_eq!(format!("{id}"), "Subscriber(99)");
    }

    #[test]
    fn start_stop_tracking_roundtrip() {
        let sub = next_subscriber_id();
        start_tracking(sub);
        assert!(is_tracking());
        let deps = stop_tracking();
        assert!(!is_tracking());
        assert!(deps.is_empty());
    }

    #[test]
    fn record_read_outside_tracking_is_noop() {
        // Should not panic
        record_read(SignalId(1));
        assert!(!is_tracking());
    }

    #[test]
    fn record_read_inside_tracking() {
        let sub = next_subscriber_id();
        start_tracking(sub);

        let sig_a = SignalId(100);
        let sig_b = SignalId(200);
        record_read(sig_a);
        record_read(sig_b);

        let deps = stop_tracking();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0], sig_a);
        assert_eq!(deps[1], sig_b);
    }

    #[test]
    fn nested_start_overwrites_previous() {
        let sub_a = next_subscriber_id();
        let sub_b = next_subscriber_id();

        start_tracking(sub_a);
        record_read(SignalId(1));

        // Starting new tracking replaces the old one
        start_tracking(sub_b);
        record_read(SignalId(2));

        let deps = stop_tracking();
        // Should only have the second signal (old scope was replaced)
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0], SignalId(2));
    }

    #[test]
    fn stop_tracking_when_not_active() {
        let deps = stop_tracking();
        assert!(deps.is_empty());
    }
}
