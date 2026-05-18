use anyhow::{Result, anyhow};
use clap::Args;
use console::style;

use forge_config::{
    find_project_root_or_fallback, 
    parse_forge_json, 
    resolve_config, 
    CliOverrides
};
use forge_core::artifact::{ArtifactKind, GenerationRequest};
use forge_core::fs::RealFileSystem;
use forge_core::generator::Generator;

#[derive(Args, Debug)]
pub struct GenerateArgs {
    /// The artifact type to generate (e.g module, service)
    pub artifact: String,

    /// The name of the artifact (e.g product, auth)
    pub name: String,

    /// Skip creating a subdirectory
    #[arg(long)]
    pub flat: Option<bool>,

    /// Preview what will be generated without writing any files
    #[arg(long)]
    pub dry_run: bool,

    /// Output path relative to src/
    #[arg(long, short)]
    pub path: Option<String>,

    /// Generate a spec (test) file alongside artifact
    #[arg(long)]
    pub spec: Option<bool>,
}

pub fn run(args: GenerateArgs) -> Result<()> {
    // Resolve artifact kind
    let kind = ArtifactKind::from_str(&args.artifact).ok_or_else(|| {
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

    // Resolve output path
    // let base_path = match &args.path {
    //     Some(path) => PathBuf::from(path),
    //     None => std::env::current_dir()?,
    // };

    let artifact_name = forge_core::artifact::ArtifactName::parse(&args.name)
        .map_err(|e| anyhow!("{}", style(e.to_string()).red()))?;

    let cwd = std::env::current_dir()?;
    let (project_root, config_found) = find_project_root_or_fallback(&cwd);

    let file_config = if  config_found {
        let config_path = project_root.join("forge.json");
        let parsed = parse_forge_json(&config_path).map_err(|e| anyhow!("Config error: {e}"))?;

        Some(parsed)
    } else {
        tracing::debug!("No project config found, using defaults");
        None
    };

    let cli_overrides = CliOverrides {
        generate_spec: args.spec,
        flat: args.flat,
        source_root: args.path.clone(),
    };

    let config = resolve_config(&project_root, file_config.as_ref(), &cli_overrides).map_err(|e| anyhow!("Config error: {e}"))?;

    // Per-artifact path override > source_root
    let output_path = config.output_path_for(&kind.template_name()).clone();

    // Build generation request
    let request = GenerationRequest {
        kind,
        name: artifact_name,
        output_path,
        flat: config.flat,
        dry_run: args.dry_run,
        generate_spec: config.generate_spec,
    };

    if config_found {
        tracing::debug!(root = %project_root.display(), source_root = %config.source_root.display(), "using project config");
    } 

    // Run generator
    let generator = Generator::new(RealFileSystem)?;
    let output = generator.generate(&request)?;

    crate::output::print_generation_result(&output);

    // Show where config came from (only in verbose mode)
    if config_found {
        tracing::debug!("config loaded from {}", project_root.join("forge.json").display());
    } 

    Ok(())
}
