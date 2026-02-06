//! Terminal abstraction for backend-agnostic rendering.

mod crossterm_backend;
mod test_backend;
mod traits;

pub use crossterm_backend::CrosstermBackend;
pub use test_backend::TestBackend;
pub use traits::{ColorSupport, Terminal, TerminalCapabilities};
