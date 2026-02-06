//! fae-cli: Thin CLI entry point for the fae AI coding agent.

fn main() -> anyhow::Result<()> {
    println!("fae-cli v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
