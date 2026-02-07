//! Terminal CSS (TCSS) parser and stylesheet types.
//!
//! TCSS is a subset of CSS tailored for terminal user interfaces.
//! It supports selectors, properties, and values specific to
//! terminal rendering capabilities.

pub mod error;
pub mod value;

pub use error::TcssError;
pub use value::{CssValue, Length};
