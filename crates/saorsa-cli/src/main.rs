//! saorsa-cli: Thin CLI entry point for the saorsa AI coding agent.
//!
//! This binary delegates to the `saorsa` crate for all functionality.
//! It exists as a separate crate to keep the binary minimal.

fn main() -> anyhow::Result<()> {
    // saorsa crate's main.rs handles everything.
    // This binary is a thin wrapper for the `saorsa` command.
    // For now, print version info.
    println!("saorsa v{}", env!("CARGO_PKG_VERSION"));
    println!("Run `saorsa` for the full interactive experience.");
    Ok(())
}
