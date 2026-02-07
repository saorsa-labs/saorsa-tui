//! Reactive primitives for automatic UI updates.
//!
//! Provides [`Signal<T>`] for mutable state, [`Computed<T>`] for
//! derived values, [`Effect`] for side effects, [`ReactiveScope`]
//! for lifetime management, and [`batch()`] for coalescing updates.
//!
//! All primitives use automatic dependency tracking via thread-local
//! context — reading a signal inside a computed or effect closure
//! automatically registers the dependency.

pub mod batch;
pub mod binding;
pub mod computed;
pub mod context;
pub mod effect;
pub mod scope;
pub mod signal;

pub use batch::batch;
pub use binding::{
    Binding, BindingDirection, BindingExpression, BindingId, BindingScope, OneWayBinding,
    PropertySink, TwoWayBinding,
};
pub use computed::Computed;
pub use effect::Effect;
pub use scope::ReactiveScope;
pub use signal::{Signal, Subscriber};

#[cfg(test)]
mod tests;
