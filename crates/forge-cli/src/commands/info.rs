use anyhow::{Ok, Result};
use console::style;

pub fn run() -> Result<()> {
    println!(
        "{} forge v{} ({})",
        style("◆").cyan(),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME")
    );

    Ok(())
}
