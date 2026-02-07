# Phase 5.2: Data Binding

## Overview
Connect reactive signals to widget properties via binding primitives.
Batch updates are already complete (Phase 5.1).

## Tasks

### Task 1: Binding Traits & IDs
- `BindingId` type alias (u64 from AtomicU64)
- `BindingDirection` enum: OneWay, TwoWay
- `Binding` trait: id(), direction(), is_active(), dispose()
- `PropertySink<T>` trait: set_value(&self, &T) — target that receives values

### Task 2: OneWayBinding
- Wraps an Effect that reads a Signal<T> and pushes to a PropertySink<T>
- Constructor: `OneWayBinding::new(source: &Signal<T>, sink: impl PropertySink<T>)`
- Subscribes to source signal, pushes on every change
- dispose() deactivates the internal effect

### Task 3: TwoWayBinding
- Bidirectional: source Signal <-> sink PropertySink, with EventSource<T> callback
- `TwoWayBinding::new(source: &Signal<T>, sink: impl PropertySink<T>)`
- Returns `TwoWayBinding<T>` with `write_back(value: T)` method
- Loop guard: `Rc<Cell<bool>>` prevents infinite ping-pong

### Task 4: BindingExpression (transform binding)
- Maps source signal through a transform function before pushing to sink
- `BindingExpression::new(source: &Signal<S>, transform: Fn(&S) -> T, sink: impl PropertySink<T>)`
- Uses Computed internally for caching the transform result

### Task 5: BindingScope
- Owns `Vec<Box<dyn Binding>>` and a `ReactiveScope`
- Drop disposes all bindings and scope
- Methods: bind(), bind_two_way(), bind_expression(), binding_count()

### Task 6: Convenience API on ReactiveScope
- `ReactiveScope::bind()` — creates OneWayBinding
- `ReactiveScope::bind_two_way()` — creates TwoWayBinding
- `ReactiveScope::bind_expression()` — creates BindingExpression

### Task 7: Integration Tests
- OneWay: signal → sink updates
- TwoWay: signal ↔ sink with loop guard
- Expression: signal → transform → sink
- Scope lifecycle: bindings disposed on scope drop
- Batch + binding interaction

### Task 8: Module Integration & Re-exports
- Add `pub mod binding;` to reactive/mod.rs
- Re-export public types from reactive module
- Re-export from lib.rs
- cargo fmt, clippy, test verification
