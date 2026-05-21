use std::collections::BTreeMap;

use console::style;
use forge_core::generator::{GeneratedFile, GenerationOutput};

use crate::theme;

/// Render the result of a generation run to stdout.
pub fn print_generation_result(output: &GenerationOutput) {
    if output.files.is_empty() {
        println!();
        println!("  {} Nothing to generate", theme::icon_warning());
        println!();
        return;
    }

    if output.dry_run {
        println!();
        println!(
            "   {} Dry run - no files will be written",
            theme::icon_dry_run()
        );
    }

    println!();
    render_file_tree(&output.files, output.dry_run);
    render_summary(&output.files, output.dry_run);
    println!();
}

/// Display a path with Unix-style slashes regardless of the operating system.
pub fn display_path(path: &std::path::Path) -> String {
    path.display().to_string().replace("\\", "/")
}

/// Render files grouped by their parent directory as a tree
fn render_file_tree(files: &[GeneratedFile], dry_run: bool) {
    let mut tree: BTreeMap<String, Vec<&GeneratedFile>> = BTreeMap::new();

    for file in files {
        let dir = file
            .path
            .parent()
            .map(display_path)
            .unwrap_or_else(|| ".".to_string());
        tree.entry(dir).or_default().push(file);
    }

    for (dir, dir_files) in &tree {
        // Directory line dimmed, no icon
        println!("  {}", theme::muted(dir));

        let count = dir_files.len();
        for (i, file) in dir_files.iter().enumerate() {
            let is_last = i == count - 1;
            let branch = if is_last {
                theme::tree_last()
            } else {
                theme::tree_branch()
            };

            let filename = file
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Color code by file type
            let formatted_name = format_filename(&filename);

            if file.skipped {
                println!(
                    "  {} {} {}",
                    style(branch).dim(),
                    theme::label_skip(),
                    style(filename).yellow().dim()
                );
            } else if dry_run {
                println!(
                    "  {} {} {}",
                    style(branch).dim(),
                    theme::label_create_dry(),
                    formatted_name
                );
            } else {
                println!(
                    "  {} {} {}",
                    style(branch).dim(),
                    theme::label_create(),
                    formatted_name
                );
            }
        }

        println!();
    }
}

/// Color filename by their extension, main files are bold, spec files are dimmed;
fn format_filename(name: &str) -> String {
    if name.contains(".spec") {
        style(name).dim().to_string()
    } else {
        style(name).bold().to_string()
    }
}

/// One-line summary at the bottom: "4 files created, 1 skipped"
fn render_summary(files: &[GeneratedFile], dry_run: bool) {
    let created = files.iter().filter(|f| !f.skipped).count();
    let skipped = files.iter().filter(|f| f.skipped).count();

    if dry_run {
        println!(
            "   {} {} file(s) would be created",
            theme::icon_dry_run(),
            style(created).bold()
        )
    } else {
        let mut summary = format!(
            "   {} {} file(s) created",
            theme::icon_success(),
            style(created).green().bold(),
        );
        if skipped > 0 {
            summary.push_str(&format!("   {} skipped", style(skipped).yellow()));
        }
        println!("{summary}")
    }
}
