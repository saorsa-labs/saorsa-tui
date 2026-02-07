# Phase 5.1: Signals & Computed

**Milestone**: 5 — Reactive System
**Goal**: Signal-based reactivity primitives. Change a value, dependent computations update automatically.

---

## Architecture Overview

The reactive system introduces `Signal<T>`, `Computed<T>`, and `Effect` — core primitives that enable automatic propagation of state changes through the UI. This is foundational for Phase 5.2 (Data Binding) where widgets bind directly to signals.

### Design Principles

1. **Interior mutability with `Rc<RefCell<...>>`** — Signals are cheaply cloneable handles (like `Rc`). No `Arc`/`Mutex` needed since saorsa-core is single-threaded (terminal UI event loop).
2. **Push-based notification, pull-based evaluation** — When a signal changes, it marks dependents as dirty. Computed values re-evaluate lazily on next read.
3. **Automatic dependency tracking** — Reading a signal inside a `Computed` or `Effect` closure automatically registers the dependency. No manual subscription.
4. **Batched updates** — Multiple signal changes within a batch only trigger one round of effect re-runs.
5. **Zero external dependencies** — Only uses `std`. No reactive framework crate needed.

### Module Structure

New module: `crates/saorsa-core/src/reactive/`

```
reactive/
├── mod.rs          — Public API, re-exports
├── signal.rs       — Signal<T> (readable + writable)
├── computed.rs     — Computed<T> (derived/memo)
├── effect.rs       — Effect (side effects on change)
├── scope.rs        — ReactiveScope (owner/cleanup)
├── batch.rs        — Batching and scheduling
├── context.rs      — Dependency tracking context (thread-local)
└── tests.rs        — Integration tests (comprehensive)
```

---

## Tasks

### Task 1: Reactive Context & Signal Core

**Files**: `reactive/mod.rs`, `reactive/context.rs`, `reactive/signal.rs`

Create the dependency tracking infrastructure and `Signal<T>`.

**context.rs:**
- Thread-local `TRACKING_CONTEXT` — stores the currently-evaluating reactive node
- `ReactiveNode` — an entry in the dependency graph (stores a `Vec<SubscriberId>`)
- `with_tracking(node_id, closure)` — sets tracking context, runs closure, restores previous
- `track(signal_id)` — called when a signal is read; registers current node as dependent

**signal.rs:**
- `Signal<T>` — cheaply cloneable handle (`Rc<RefCell<SignalInner<T>>>`)
- `SignalInner<T>` — `{ value: T, id: SignalId, subscribers: Vec<Weak<dyn Subscriber>> }`
- `Signal::new(value) -> Signal<T>` — create a new signal
- `Signal::get(&self) -> T` where `T: Clone` — read value (tracks dependency if in reactive context)
- `Signal::get_untracked(&self) -> T` — read without tracking
- `Signal::set(&self, value: T)` — write value, notify subscribers
- `Signal::update(&self, f: impl FnOnce(&mut T))` — update in place
- `Signal::with<R>(&self, f: impl FnOnce(&T) -> R) -> R` — borrow without cloning
- `SignalId` — `#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] struct SignalId(u64)`
- Global atomic counter for unique IDs

**mod.rs:**
- `pub mod signal; pub mod context;`
- Re-exports: `Signal`, `SignalId`

**Tests** (~15):
- Create signal, get/set roundtrip
- Signal::update modifies in place
- Signal::with borrows without clone
- get_untracked does NOT register dependency
- Multiple signals independent
- Signal<T> is Clone (cheap handle clone)
- Setting same value still notifies (simple semantics)
- SignalId uniqueness
- Thread-local tracking context push/pop

---

### Task 2: Subscriber Trait & Notification

**Files**: `reactive/context.rs` (extend), `reactive/signal.rs` (extend)

Define the `Subscriber` trait and wire signal writes to subscriber notification.

**context.rs additions:**
- `Subscriber` trait: `fn notify(&self)` — called when a dependency changes
- `SubscriberId` — `#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] struct SubscriberId(u64)`
- `SubscriberEntry` — `{ id: SubscriberId, subscriber: Weak<dyn Subscriber> }`
- `DependencyTracker` — manages current tracking context
  - `start_tracking(subscriber_id: SubscriberId)` — begin collecting dependencies
  - `stop_tracking() -> Vec<SignalId>` — end tracking, return collected signal IDs
  - `record_read(signal_id: SignalId)` — record a signal read

**signal.rs extensions:**
- On `Signal::set()` / `Signal::update()`:
  1. Update value
  2. Iterate subscribers, call `notify()` on each live `Weak`
  3. Prune dead `Weak` references
- On `Signal::get()`:
  1. If tracking active, record this signal in the tracker
  2. Return value

**Tests** (~12):
- Mock subscriber receives notification on set
- Mock subscriber receives notification on update
- Dead subscriber (dropped Weak) is pruned automatically
- Multiple subscribers all notified
- Notification order is insertion order
- Tracking context records signal reads
- Nested tracking context push/pop works correctly

---

### Task 3: Computed Values

**Files**: `reactive/computed.rs`

`Computed<T>` is a derived value that automatically re-evaluates when its dependencies change.

**computed.rs:**
- `Computed<T>` — cloneable handle (`Rc<RefCell<ComputedInner<T>>>`)
- `ComputedInner<T>`:
  - `value: Option<T>` — cached result
  - `compute_fn: Box<dyn Fn() -> T>` — the derivation function
  - `dirty: bool` — needs re-evaluation
  - `dependencies: Vec<SignalId>` — tracked signal dependencies
  - `id: SubscriberId`
- `Computed::new(f: impl Fn() -> T + 'static) -> Computed<T>`
  - Evaluates `f` immediately within tracking context to discover dependencies
  - Subscribes to all discovered signals
- `Computed::get(&self) -> T` where `T: Clone`:
  - If dirty, re-evaluate within tracking context (re-discover dependencies)
  - If not dirty, return cached value
  - If in tracking context, register as dependency (computed can depend on computed)
- Implements `Subscriber` trait:
  - `notify()` marks `dirty = true`, then notifies own subscribers (propagation)
- Computed values are read-only (no `set`)

**Tests** (~15):
- Computed from single signal updates when signal changes
- Computed from multiple signals tracks all deps
- Computed is lazy — doesn't re-evaluate until read
- Computed chains (computed depending on computed)
- Computed re-discovers dependencies on re-evaluation (dynamic deps)
- Computed::get in tracking context registers as dependency
- Diamond dependency (A -> B, A -> C, B&C -> D) — D updates once
- Computed with no signal deps (constant) never goes dirty

---

### Task 4: Effects (Side Effects)

**Files**: `reactive/effect.rs`

`Effect` runs a closure whenever its reactive dependencies change.

**effect.rs:**
- `Effect` — owns its closure, auto-runs when dependencies change
- `EffectInner`:
  - `effect_fn: Box<dyn FnMut()>` — the side effect closure
  - `dependencies: Vec<SignalId>` — tracked dependencies
  - `id: SubscriberId`
  - `active: bool` — can be paused/disposed
- `Effect::new(f: impl FnMut() + 'static) -> Effect`
  - Runs `f` immediately within tracking context to discover dependencies
  - Subscribes to all discovered signals
- Implements `Subscriber` trait:
  - `notify()` re-runs the effect (within tracking context to re-discover deps)
- `Effect::dispose(&self)` — permanently deactivates the effect
- `Effect::is_active(&self) -> bool`
- Effects are NOT lazy — they run immediately on notification (unlike Computed)

**Tests** (~12):
- Effect runs immediately on creation
- Effect re-runs when dependency signal changes
- Effect tracks multiple signal dependencies
- Effect re-discovers dependencies on each run (dynamic deps)
- Disposed effect does not run
- Effect with no signal deps runs only once
- Effect can read Computed values (chain: signal -> computed -> effect)
- Multiple effects on same signal all fire

---

### Task 5: ReactiveScope (Ownership & Cleanup)

**Files**: `reactive/scope.rs`

`ReactiveScope` provides automatic cleanup of effects and computed values when the scope is dropped. This prevents memory leaks in widget trees.

**scope.rs:**
- `ReactiveScope`:
  - `effects: Vec<Effect>` — owned effects
  - `children: Vec<ReactiveScope>` — nested scopes
  - `cleanups: Vec<Box<dyn FnOnce()>>` — cleanup callbacks
- `ReactiveScope::new() -> ReactiveScope`
- `ReactiveScope::create_signal<T>(&self, value: T) -> Signal<T>` — just creates, scope doesn't own signals (they're shared handles)
- `ReactiveScope::create_computed<T>(&self, f: impl Fn() -> T + 'static) -> Computed<T>` — tracked in scope
- `ReactiveScope::create_effect(&mut self, f: impl FnMut() + 'static) -> &Effect` — owned by scope
- `ReactiveScope::on_cleanup(&mut self, f: impl FnOnce() + 'static)` — register cleanup callback
- `ReactiveScope::child(&mut self) -> &mut ReactiveScope` — create nested scope
- `Drop for ReactiveScope` — disposes all effects, runs all cleanups, drops children

**Tests** (~12):
- Dropping scope disposes all effects
- Dropping scope runs cleanup callbacks
- Nested scope dropped before parent
- Cleanup callbacks run in reverse registration order
- Effects in dropped scope no longer fire on signal change
- create_signal/create_computed/create_effect all work through scope
- Scope can be moved (no self-referential borrows)

---

### Task 6: Batched Updates

**Files**: `reactive/batch.rs`

Batching allows multiple signal changes to coalesce into a single notification pass.

**batch.rs:**
- Thread-local `BATCH_DEPTH: Cell<u32>` — tracks batch nesting depth
- Thread-local `PENDING_EFFECTS: RefCell<Vec<SubscriberId>>` — effects to run after batch
- `batch(f: impl FnOnce())`:
  1. Increment `BATCH_DEPTH`
  2. Run `f`
  3. Decrement `BATCH_DEPTH`
  4. If depth reaches 0, flush: run all pending effects
- When `BATCH_DEPTH > 0`:
  - Signal notifications queue subscribers instead of running them immediately
  - Duplicate subscriber IDs are deduplicated in the queue
- When `BATCH_DEPTH == 0`:
  - Normal behavior (immediate notification)

**signal.rs modifications:**
- `Signal::set()` and `Signal::update()` check `BATCH_DEPTH`
  - If batching active: mark subscribers dirty, enqueue for later
  - If not batching: notify immediately (existing behavior)

**Tests** (~10):
- Without batch: effect runs on every signal set
- With batch: effect runs once after batch completes
- Nested batch: effects only run after outermost batch completes
- Batch with multiple signals: single notification pass
- Empty batch (no changes): no spurious notifications
- Batch within effect: works correctly (no infinite loop)
- Computed values within batch stay dirty until batch ends

---

### Task 7: Module Integration & Public API

**Files**: `reactive/mod.rs` (finalize), `lib.rs` (add module), `error.rs` (add variant)

Wire everything together and export a clean public API.

**reactive/mod.rs:**
```rust
pub mod batch;
pub mod computed;
pub mod context;
pub mod effect;
pub mod scope;
pub mod signal;

pub use batch::batch;
pub use computed::Computed;
pub use effect::Effect;
pub use scope::ReactiveScope;
pub use signal::Signal;
```

**lib.rs additions:**
- `pub mod reactive;`
- Add to re-exports: `Signal`, `Computed`, `Effect`, `ReactiveScope`, `batch`

**error.rs additions:**
- `Reactive(String)` variant to `SaorsaCoreError`

**Ensure:**
- All public types have doc comments
- All public methods have doc comments with examples
- `#[must_use]` on `Signal::new()`, `Computed::new()`, `Effect::new()`, `ReactiveScope::new()`
- No clippy warnings
- No compilation warnings

**Tests** (~8):
- Full roundtrip: signal -> computed -> effect through public API
- Public re-exports accessible from `saorsa_core::reactive::*`
- Public re-exports accessible from `saorsa_core::Signal` etc.
- Error variant for reactive errors

---

### Task 8: Integration Tests & Stress Tests

**Files**: `reactive/tests.rs` (new), extend existing integration test pattern

Comprehensive integration tests covering realistic usage patterns and edge cases.

**Integration tests:**
- Counter app pattern: `Signal<i32>` + `Computed` for display text + `Effect` for rendering
- Todo list pattern: `Signal<Vec<Item>>` + multiple computed (count, filtered) + effects
- Theme switch pattern: `Signal<Theme>` + computed styles + effect to reapply
- Scope-based widget lifecycle: create scope, create signals/effects, drop scope, verify cleanup
- Nested computed chain (3+ levels deep)
- Circular dependency detection / no infinite loop (computed A reads B, B reads A — should NOT stack overflow)
- Stress: 1000 signals, 100 computed, 50 effects — no panic, reasonable performance
- Stress: rapid set() calls (1000 times) — no accumulation of dead subscribers
- Batch with complex dependency graph
- Dynamic dependency switching (computed reads different signals based on another signal's value)

**Tests** (~15):
- All above integration scenarios
- Verify no memory leaks (scope cleanup)
- Verify subscriber pruning (Weak references cleaned up)

---

## Summary

| Task | Description | Estimated Tests |
|------|-------------|----------------|
| 1 | Reactive Context & Signal Core | ~15 |
| 2 | Subscriber Trait & Notification | ~12 |
| 3 | Computed Values | ~15 |
| 4 | Effects (Side Effects) | ~12 |
| 5 | ReactiveScope (Ownership & Cleanup) | ~12 |
| 6 | Batched Updates | ~10 |
| 7 | Module Integration & Public API | ~8 |
| 8 | Integration Tests & Stress Tests | ~15 |
| **Total** | | **~99 tests** |

## Dependencies

- **No new crate dependencies** — pure `std` implementation using `Rc`, `RefCell`, `Cell`, `Weak`, `thread_local!`
- All types are `!Send` and `!Sync` (single-threaded by design)

## Key Design Decisions

1. **`Rc<RefCell<T>>` over `Arc<Mutex<T>>`** — Terminal UI is single-threaded. Avoiding atomic overhead.
2. **Lazy computed, eager effect** — Computed values don't re-evaluate until read. Effects run immediately on change.
3. **Weak subscriber references** — Prevents memory leaks. Dead subscribers are automatically pruned.
4. **Thread-local tracking** — Automatic dependency discovery via thread-local context stack.
5. **Explicit batching** — `batch()` function for grouping updates. No implicit batching.
6. **Scope-based cleanup** — `ReactiveScope` ties reactive primitives to widget lifetimes.
