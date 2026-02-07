//! Terminal abstraction for backend-agnostic rendering.

mod crossterm_backend;
mod detect;
pub mod multiplexer;
mod profiles;
mod query;
mod test_backend;
mod traits;

pub use crossterm_backend::CrosstermBackend;
pub use detect::{
    MultiplexerKind, TerminalInfo, TerminalKind, detect, detect_multiplexer, detect_terminal,
};
pub use profiles::{merge_multiplexer_limits, profile_for};
pub use query::{LiveQuerier, MockQuerier, TerminalQuerier, detect_capabilities};
pub use test_backend::TestBackend;
pub use traits::{ColorSupport, Terminal, TerminalCapabilities};
