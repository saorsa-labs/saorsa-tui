//! Application commands for the fae AI agent.

pub mod bookmark;
pub mod clear;
pub mod compact;
pub mod export;
pub mod fork;
pub mod help;
pub mod hotkeys;
pub mod login;
pub mod logout;
pub mod model;
pub mod settings;
pub mod share;
pub mod thinking;
pub mod tree;

pub use bookmark::BookmarkCommand;
pub use export::ExportCommand;
pub use fork::ForkCommand;
pub use tree::TreeCommand;
