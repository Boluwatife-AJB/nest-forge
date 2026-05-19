use anyhow::Result;
use console::style;
use std::collections::HashMap;

use forge_config::{find_project_root_or_fallback, resolve_config, CliOverrides, parse_forge_json};

use crate::errors::{print_info, print_section, print_warning};
use crate::theme;

pub fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let (project_root, config_found) = find_project_root_or_fallback(&cwd);

    let file_config = if config_found {
        let config_path = project_root.join("forge.json");
        match parse_forge_json(&config_path) {
            Ok(c) => Some(c),
            Err(e) => {
                print_warning(&format!("forge.json found but could not be parsed: {e}"));
                None
            }
        }
    } else {
        None
    };

    let config = resolve_config(&project_root, file_config.as_ref(), &CliOverrides::default()).unwrap_or_else(|_| {
        forge_config::ResolvedConfig { project_root:project_root.clone(), source_root: project_root.join("src"), language: "ts".into(), generate_spec: true, flat: false, paths: HashMap::new() }
    });

    // Header
    println!();
    println!(
        "   {} {}",
        style("forge").bold().cyan(),
        style(env!("CARGO_PKG_VERSION")).bold()
    );
    println!(
        "   {}",
        theme::muted("A NestJS artifact generator, forged in Rust")
    );
    
    // Environment
    print_section("Environment");

    print_info("working dir", &style(cwd.display().to_string()).dim().to_string());

    print_info("project root", &style(project_root.display().to_string()).bold().to_string());

    // Configuration
    print_section("Configuration");

    if config_found {
        let config_path = project_root.join("forge.json");
        print_info("config file", &format!("{} {}", style(config_path.display().to_string()).bold(), style("(found)").green()));
    } else {
        print_info("config file", &format!("{} {}", style("forge.json").dim(), style("(not found) - using defaults").yellow()));
        println!();
        println!("   {} Run {} to create a config file", theme::icon_info(), style("forge init").bold().cyan());
    }

    // Resolved settings
    print_section("Resolved settings");

    print_info("source root", &style(config.source_root.display().to_string()).bold().to_string());

    print_info("language", &style(&config.language).bold().to_string());

    print_info("generate spec", &bool_display(config.generate_spec));

    print_info("flat", &bool_display(config.flat));

    // Path overrides
    if !config.paths.is_empty() {
        print_section("Path overrides");

        let mut sorted_paths: Vec<_> = config.paths.iter().collect();
        sorted_paths.sort_by_key(|(k, _)| k.as_str());
        
        for (artifact, path) in &sorted_paths {
            print_info(artifact, &style(path.display().to_string()).bold().to_string());
        }
    }

    // Supported artifacts
    print_section("Supported Artifacts");
    
    let artifacts = [
        ("module", "mo"),
        ("service", "s"),
        ("controller", "co"),
        ("class", "cl"),
        ("dto", "dto"),
        ("guard", "gu"),
        ("interceptor", "itc"),
        ("middleware", "mi"),
        ("pipe", "pi"),
        ("decorator", "d"),
        ("strategy", "-"),
        ("interface", "itf"),
        ("filter", "f"), 
        ("config", "-"), 
        ("resolver", "r"),
        ("entity", "e"),
    ];
    
    for (name, alias) in &artifacts {
        println!(
            "   {:>14} {} {}", style(name).bold(), style("alias:").dim(), style(alias).cyan());
    }

    // Shell completion hint
    println!();
    println!("   {} Shell completion: {}", theme::icon_info(), style("forge completions <bash|zsh|fish|powershell>").cyan());
    
    println!();

    Ok(())
}


fn bool_display(value: bool) -> String {
    if value {
        style("true").green().bold().to_string()
    } else {
        style("false").dim().to_string()
    }
}