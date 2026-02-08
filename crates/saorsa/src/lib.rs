//! saorsa: The AI coding agent application.
//!
//! This crate provides the application logic for the saorsa AI coding agent,
//! combining saorsa-core (TUI framework), saorsa-ai (LLM providers), and
//! saorsa-agent (agent runtime) into a complete interactive application.

pub mod app;
pub mod autocomplete;
pub mod cli;
pub mod commands;
pub mod input;
pub mod keybindings;
pub mod operating_mode;
pub mod render_throttle;
pub mod ui;
pub mod widgets;
