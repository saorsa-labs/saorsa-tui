//! fae-cli: Thin CLI entry point for the fae AI coding agent.
//!
//! This binary delegates to fae-app for all functionality.
//! It exists as a separate crate to keep the binary minimal.

fn main() -> anyhow::Result<()> {
    // fae-app's main.rs handles everything.
    // This binary is a thin wrapper for the `fae` command.
    // For now, print version info.
    println!("fae v{}", env!("CARGO_PKG_VERSION"));
    println!("Run `fae-app` for the full interactive experience.");
    Ok(())
}
