//! saorsa-cli: Thin CLI entry point for the saorsa-tui AI coding agent.
//!
//! This binary delegates to saorsa-app for all functionality.
//! It exists as a separate crate to keep the binary minimal.

fn main() -> anyhow::Result<()> {
    // saorsa-app's main.rs handles everything.
    // This binary is a thin wrapper for the `saorsa-tui` command.
    // For now, print version info.
    println!("saorsa-tui v{}", env!("CARGO_PKG_VERSION"));
    println!("Run `saorsa-app` for the full interactive experience.");
    Ok(())
}
