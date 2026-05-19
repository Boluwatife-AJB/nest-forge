use anyhow::{Ok, Result, anyhow};
use console::style;
// use std::path::PathBuf;

// use forge_config::ForgeJsonFile;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join("forge.json");

    if config_path.exists() {
        return Err(anyhow!(
            "{} already exists in this directory.\n Delete it first if you want to reinitialize.",
            style("forge.json").yellow()
        ));
    }

    let default_config = serde_json::json!({
      "sourceRoot": "src",
      "language": "ts",
      "generateSpec": true,
      "flat": false,
      "path": {}
    });

    let contents = serde_json::to_string_pretty(&default_config)
        .map_err(|e| anyhow!("Failed to seralize config: {e}"))?;

    std::fs::write(&config_path, contents + "\n")?;

    println!(
        "  {} {} {}",
        style("✓").green(),
        style("CREATE").green(),
        style(config_path.display()).bold()
    );
    println!(
        "\n  {} Edit {} to customise source root, paths, and generation defaults.",
        style("◆").cyan(),
        style("forge.json").cyan()
    );

    Ok(())
}
