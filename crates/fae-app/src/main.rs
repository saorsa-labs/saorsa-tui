//! fae-app: The AI coding agent application.

fn main() -> anyhow::Result<()> {
    println!("fae v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
