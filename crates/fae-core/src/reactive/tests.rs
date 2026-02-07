//! Integration tests for the reactive system.
//!
//! Tests realistic usage patterns combining signals, computed values,
//! effects, scopes, and batching.

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod integration {
    use crate::reactive::batch::batch;
    use crate::reactive::computed::Computed;
    use crate::reactive::effect::Effect;
    use crate::reactive::scope::ReactiveScope;
    use crate::reactive::signal::Signal;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    /// Counter app pattern: signal + computed display + effect render.
    #[test]
    fn counter_app_pattern() {
        let count = Signal::new(0i32);
        let display = Computed::new({
            let count = count.clone();
            move || format!("Count: {}", count.get())
        });
        count.subscribe(display.as_subscriber());

        let rendered: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let effect = Effect::new({
            let display = display.clone();
            let rendered = Rc::clone(&rendered);
            move || {
                rendered.borrow_mut().push(display.get());
            }
        });
        display.subscribe(effect.as_subscriber());

        count.set(1);
        count.set(2);

        assert_eq!(*rendered.borrow(), vec!["Count: 0", "Count: 1", "Count: 2"]);
    }

    /// Todo list pattern: signal with vec + multiple computed views.
    #[test]
    fn todo_list_pattern() {
        let todos: Signal<Vec<(String, bool)>> = Signal::new(vec![
            ("Buy milk".into(), false),
            ("Write code".into(), true),
        ]);

        let total = Computed::new({
            let todos = todos.clone();
            move || todos.with(|t| t.len())
        });
        todos.subscribe(total.as_subscriber());

        let done_count = Computed::new({
            let todos = todos.clone();
            move || todos.with(|t| t.iter().filter(|(_, done)| *done).count())
        });
        todos.subscribe(done_count.as_subscriber());

        assert_eq!(total.get(), 2);
        assert_eq!(done_count.get(), 1);

        todos.update(|t| t.push(("Read book".into(), true)));
        assert_eq!(total.get(), 3);
        assert_eq!(done_count.get(), 2);
    }

    /// Theme switch pattern: signal theme + computed styles.
    #[test]
    fn theme_switch_pattern() {
        let dark_mode = Signal::new(false);

        let bg_color = Computed::new({
            let dark = dark_mode.clone();
            move || {
                if dark.get() { "black" } else { "white" }
            }
        });
        dark_mode.subscribe(bg_color.as_subscriber());

        let fg_color = Computed::new({
            let dark = dark_mode.clone();
            move || {
                if dark.get() { "white" } else { "black" }
            }
        });
        dark_mode.subscribe(fg_color.as_subscriber());

        assert_eq!(bg_color.get(), "white");
        assert_eq!(fg_color.get(), "black");

        dark_mode.set(true);
        assert_eq!(bg_color.get(), "black");
        assert_eq!(fg_color.get(), "white");
    }

    /// Scope-based widget lifecycle.
    #[test]
    fn scope_based_widget_lifecycle() {
        let global_sig = Signal::new(0);
        let effect_ran = Rc::new(Cell::new(0u32));

        {
            let mut scope = ReactiveScope::new();
            let effect = scope.create_effect({
                let sig = global_sig.clone();
                let count = Rc::clone(&effect_ran);
                move || {
                    let _ = sig.get();
                    count.set(count.get() + 1);
                }
            });
            global_sig.subscribe(effect.as_subscriber());

            global_sig.set(1);
            assert_eq!(effect_ran.get(), 2); // Initial + 1 change.
        }
        // Scope dropped.
        global_sig.set(2);
        assert_eq!(effect_ran.get(), 2); // No further runs.
    }

    /// Nested computed chain (3 levels deep).
    #[test]
    fn nested_computed_chain_three_levels() {
        let base = Signal::new(2);

        let level1 = Computed::new({
            let base = base.clone();
            move || base.get() * 2
        });
        base.subscribe(level1.as_subscriber());

        let level2 = Computed::new({
            let level1 = level1.clone();
            move || level1.get() + 10
        });
        level1.subscribe(level2.as_subscriber());

        let level3 = Computed::new({
            let level2 = level2.clone();
            move || format!("Result: {}", level2.get())
        });
        level2.subscribe(level3.as_subscriber());

        assert_eq!(level3.get(), "Result: 14");

        base.set(5);
        assert_eq!(level3.get(), "Result: 20");

        base.set(0);
        assert_eq!(level3.get(), "Result: 10");
    }

    /// Stress test: many signals and effects.
    #[test]
    fn stress_many_signals_and_effects() {
        let signals: Vec<Signal<i32>> = (0..100).map(Signal::new).collect();
        let total_count = Rc::new(Cell::new(0u32));

        let mut effects = Vec::new();
        for sig in &signals {
            let effect = Effect::new({
                let sig = sig.clone();
                let count = Rc::clone(&total_count);
                move || {
                    let _ = sig.get();
                    count.set(count.get() + 1);
                }
            });
            sig.subscribe(effect.as_subscriber());
            effects.push(effect);
        }

        // 100 initial runs.
        assert_eq!(total_count.get(), 100);

        // Update all signals.
        for (i, sig) in signals.iter().enumerate() {
            sig.set(i as i32 + 1000);
        }

        // 100 initial + 100 updates = 200.
        assert_eq!(total_count.get(), 200);
    }

    /// Stress test: rapid set calls with subscriber pruning.
    #[test]
    fn stress_rapid_sets_with_pruning() {
        let sig = Signal::new(0);

        // Create and drop subscribers rapidly.
        for _ in 0..100 {
            let sub = Effect::new({
                let sig = sig.clone();
                move || {
                    let _ = sig.get();
                }
            });
            sig.subscribe(sub.as_subscriber());
            drop(sub);
        }

        // Rapid updates should not accumulate dead subscribers.
        for i in 0..100 {
            sig.set(i);
        }
        // Should not panic or leak.
    }

    /// Batch with complex dependency graph.
    #[test]
    fn batch_complex_dependency_graph() {
        let a = Signal::new(1);
        let b = Signal::new(2);

        let sum = Computed::new({
            let a = a.clone();
            let b = b.clone();
            move || a.get() + b.get()
        });
        a.subscribe(sum.as_subscriber());
        b.subscribe(sum.as_subscriber());

        let effect_count = Rc::new(Cell::new(0u32));
        let effect = Effect::new({
            let sum = sum.clone();
            let count = Rc::clone(&effect_count);
            move || {
                let _ = sum.get();
                count.set(count.get() + 1);
            }
        });
        sum.subscribe(effect.as_subscriber());

        assert_eq!(effect_count.get(), 1); // Initial.

        batch(|| {
            a.set(10);
            b.set(20);
        });

        // Should run effect once after batch (deduplicated).
        assert_eq!(effect_count.get(), 2);
        assert_eq!(sum.get(), 30);
    }

    /// Dynamic dependency switching.
    #[test]
    fn dynamic_dependency_switching() {
        let use_a = Signal::new(true);
        let a = Signal::new(10);
        let b = Signal::new(20);

        let value = Computed::new({
            let use_a = use_a.clone();
            let a = a.clone();
            let b = b.clone();
            move || {
                if use_a.get() { a.get() } else { b.get() }
            }
        });

        // Subscribe to all signals since deps are dynamic.
        use_a.subscribe(value.as_subscriber());
        a.subscribe(value.as_subscriber());
        b.subscribe(value.as_subscriber());

        assert_eq!(value.get(), 10);

        use_a.set(false);
        assert_eq!(value.get(), 20);

        b.set(30);
        assert_eq!(value.get(), 30);

        use_a.set(true);
        assert_eq!(value.get(), 10);
    }

    /// Verify scope cleanup prevents memory accumulation.
    #[test]
    fn scope_cleanup_prevents_accumulation() {
        let sig = Signal::new(0);
        let total_effects = Rc::new(Cell::new(0u32));

        for _ in 0..50 {
            let mut scope = ReactiveScope::new();
            let effect = scope.create_effect({
                let sig = sig.clone();
                let count = Rc::clone(&total_effects);
                move || {
                    let _ = sig.get();
                    count.set(count.get() + 1);
                }
            });
            sig.subscribe(effect.as_subscriber());
            // Scope drops, effect disposed.
        }

        // 50 initial runs.
        assert_eq!(total_effects.get(), 50);

        sig.set(1);
        // Dead effects should not run.
        assert_eq!(total_effects.get(), 50);
    }

    /// Effect reading another effect's computed dependency.
    #[test]
    fn effect_chain_through_computed() {
        let input = Signal::new(5);

        let processed = Computed::new({
            let input = input.clone();
            move || input.get() * 3
        });
        input.subscribe(processed.as_subscriber());

        let output: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(Vec::new()));
        let effect = Effect::new({
            let processed = processed.clone();
            let output = Rc::clone(&output);
            move || {
                output.borrow_mut().push(processed.get());
            }
        });
        processed.subscribe(effect.as_subscriber());

        input.set(10);
        input.set(0);

        assert_eq!(*output.borrow(), vec![15, 30, 0]);
    }

    /// Batch within scope.
    #[test]
    fn batch_within_scope() {
        let sig = Signal::new(0);
        let count = Rc::new(Cell::new(0u32));

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

        assert_eq!(count.get(), 1);

        batch(|| {
            sig.set(1);
            sig.set(2);
            sig.set(3);
        });

        // Only one run after batch.
        assert_eq!(count.get(), 2);
        assert_eq!(sig.get(), 3);
    }

    /// Signal with complex type (HashMap).
    #[test]
    fn signal_with_complex_type() {
        use std::collections::HashMap;

        let map: Signal<HashMap<String, i32>> = Signal::new(HashMap::new());

        let keys = Computed::new({
            let map = map.clone();
            move || {
                let mut k: Vec<String> = map.with(|m| m.keys().cloned().collect());
                k.sort();
                k
            }
        });
        map.subscribe(keys.as_subscriber());

        assert!(keys.get().is_empty());

        map.update(|m| {
            m.insert("a".into(), 1);
            m.insert("b".into(), 2);
        });
        assert_eq!(keys.get(), vec!["a", "b"]);
    }
}
