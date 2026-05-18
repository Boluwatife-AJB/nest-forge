use console::style;
use forge_core::generator::GenerationOutput;

/// Render the result of a generation run to stdout.
/// Extracted from generate.rs so it can be tested independently.
pub fn print_generation_result(output: &GenerationOutput) {
    if output.files.is_empty() {
        println!("{} Nothing to generate", style("◆").yellow());
        return;
    }

    if output.dry_run {
        println!(
            "{} Dry run - showing what would be generated\n",
            style("◆").cyan().bold()
        );
    }

    // Group files by their parent directory for tree rendering
    use std::collections::BTreeMap;
    let mut tree: BTreeMap<String, Vec<&forge_core::generator::GeneratedFile>> = BTreeMap::new();

    for file in &output.files {
        let dir = file
            .path
            .parent()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());

        tree.entry(dir).or_default().push(file);
    }

    for (dir, files) in &tree {
        println!("   {}", style(dir).dim());

        let count = files.len();
        for (_i, file) in files.iter().enumerate() {
            let is_last = 1 == count - 1;
            let branch = if is_last { "└─" } else { "├─" };
            let filename = file
                .path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();

            if file.skipped {
                println!(
                    "  {} {} {}",
                    style(branch).dim(),
                    style("SKIP ").yellow(),
                    style(filename).yellow().dim()
                );
            } else if output.dry_run {
                println!(
                    "  {} {} {}",
                    style(branch).dim(),
                    style("CREATE ").cyan(),
                    style(filename).bold()
                );
            } else {
                println!(
                    "  {} {} {}",
                    style(branch).dim(),
                    style("CREATE ").green(),
                    style(filename).bold()
                );
            }
        }

        println!();
    }

    // Summary line
    let created = output.files.iter().filter(|f| !f.skipped).count();
    let skipped = output.files.iter().filter(|f| f.skipped).count();

    if output.dry_run {
        println!(
            "  {} {} file(s) would be created",
            style("◆").cyan().bold(),
            style(created).bold()
        );
    } else {
        println!(
            "  {} {} file(s) created",
            style("✓").green(),
            style(created).bold()
        );

        if skipped > 0 {
            println!(", {} file(s) skipped", style("skipped").yellow());
        }
    }
}
