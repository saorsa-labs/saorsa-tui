//! Terminal abstraction for backend-agnostic rendering.

mod crossterm_backend;
mod detect;
mod profiles;
mod test_backend;
mod traits;

pub use crossterm_backend::CrosstermBackend;
pub use detect::{
    MultiplexerKind, TerminalInfo, TerminalKind, detect, detect_multiplexer, detect_terminal,
};
pub use profiles::{merge_multiplexer_limits, profile_for};
pub use test_backend::TestBackend;
pub use traits::{ColorSupport, Terminal, TerminalCapabilities};
