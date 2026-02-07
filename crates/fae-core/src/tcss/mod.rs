//! Terminal CSS (TCSS) parser and stylesheet types.
//!
//! TCSS is a subset of CSS tailored for terminal user interfaces.
//! It supports selectors, properties, and values specific to
//! terminal rendering capabilities.

pub mod ast;
pub mod error;
pub mod parser;
pub mod property;
pub mod selector;
pub mod value;

pub use ast::{Rule, Stylesheet};
pub use error::TcssError;
pub use property::{Declaration, PropertyName};
pub use selector::{
    Combinator, CompoundSelector, PseudoClass, Selector, SelectorList, SimpleSelector,
};
pub use value::{CssValue, Length};
