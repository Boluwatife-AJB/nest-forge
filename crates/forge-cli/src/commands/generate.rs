use anyhow::{Result, anyhow};
use clap::Args;
use console::style;
use std::path::PathBuf;

use forge_config::{CliOverrides, find_project_root_or_fallback, parse_forge_json, resolve_config};
use forge_core::artifact::{ArtifactKind, GenerationRequest};
use forge_core::fs::RealFileSystem;
use forge_core::generator::Generator;

use crate::errors::print_warning;

#[derive(Args, Debug)]
#[command(
    about = "Generate a NestJs artifact",
    long_about = "Generate a NestJS artifact with correct naming, imports, and decorators.\n\n\
                  Artifact types: module (mo), service (s), controller (co), class (cl),\n\
                  dto, guard (gu), interceptor (itc), middleware (mi), pipe (pi),\n\
                  decorator (d), strategy, interface (itf), filter (f),\n\
                  config, resolver (r), entity (e)\n\n\
                  Examples:\n  \
                  forge generate service users\n  \
                  forge g controller auth --flat\n  \
                  forge g dto create-user --dry-run\n  \
                  forge g module user-profile --spec=false",
)]
pub struct GenerateArgs {
    /// The artifact type to generate (e.g module, service)
    pub artifact: String,

    /// The name of the artifact (e.g product, auth)
    pub name: String,

    /// Skip creating a subdirectory
    #[arg(long, help = "Skip creating a subdirectory, place files in the output path directly")]
    pub flat: Option<bool>,

    /// Preview what will be generated without writing any files
    #[arg(long, help = "Preview generation output without writing files")]
    pub dry_run: bool,

    /// Output path relative to src/
    #[arg(long, short, value_name = "PATH", help = "Output path relative to project root (overrides forge.json paths")]
    pub path: Option<String>,

    /// Generate a spec (test) file alongside artifact
    #[arg(long, value_name = "BOOL", help = "Generate a spec file (default: true, or as configured in forge.json)")]
    pub spec: Option<bool>,
}

pub fn run(args: GenerateArgs) -> Result<()> {
    // Resolve artifact kind
    let kind = args.artifact.parse::<ArtifactKind>().map_err(|_| {
        let suggestion =
            forge_core::suggest::closest_match(&args.artifact, ArtifactKind::all_names())
                .map(|s| format!("Did you mean '{}'?", style(s).green()))
                .unwrap_or_else(|| {
                    format!(
                        "Run {} to see all supported artifact types.",
                        style("forge generate --help").cyan()
                    )
                });

        anyhow!(
            "Unknown artifact type '{}'\n. {}",
            style(&args.artifact).yellow(),
            suggestion
        )
    })?;

    // Validation name
    let artifact_name = forge_core::artifact::ArtifactName::parse(&args.name)
        .map_err(|e| anyhow!("{}", style(e.to_string()).red()))?;

    // Discover project root and load config
    let cwd = std::env::current_dir()?;
    let (project_root, config_found) = find_project_root_or_fallback(&cwd);

    let file_config = if config_found {
        tracing::debug!(
            root = %project_root.display(),
            "Found forge.json"
        );
        let config_path = project_root.join("forge.json");
        match parse_forge_json(&config_path) {
            Ok(c) => Some(c),
            Err(e) => {print_warning(&format!("forge.json could not be parsed ({e}), using defaults"));
        None}
        }
        // let parsed = parse_forge_json(&config_path).map_err(|e| anyhow!("Config error: {e}"))?;

        // Some(parsed)
    } else {
        tracing::debug!("No project config found, using defaults");
        None
    };

    // Build CLI overriders from flags
    let cli_overrides = CliOverrides {
        generate_spec: args.spec,
        flat: args.flat,
        source_root: args.path.clone(),
    };

    // Merge config layers
    let config = resolve_config(&project_root, file_config.as_ref(), &cli_overrides)
        .map_err(|e| anyhow!("Config error: {e}"))?;

    // Resolve final output path
    let output_path = if args.path.is_some() {
        PathBuf::from(args.path.as_deref().unwrap())
    } else {
        config.output_path_for(kind.template_name()).clone()
    };

    tracing::debug!(
        artifact = %kind.template_name(),
        name = %artifact_name.kebab,
        output_dir = %output_path.display(),
        dry_run = %args.dry_run,
        spec = %config.generate_spec,
        flat = %config.flat,
        "Generation request resolved"
    );

    // Build generation request
    let request = GenerationRequest {
        kind,
        name: artifact_name,
        output_path,
        flat: config.flat,
        dry_run: args.dry_run,
        generate_spec: config.generate_spec,
    };

    

    // Run generator
    let generator = Generator::new(RealFileSystem)?;
    let output = generator.generate(&request)?;

    // Display results
    crate::output::print_generation_result(&output);

    Ok(())
}
