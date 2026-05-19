use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::Level;
use tracing_subscriber::EnvFilter;

mod commands;
mod output;

#[derive(Parser)]
#[command(
    name="forge",
    version,
    about = "A NestJS artifact generator, forged in Rust",
    long_about = None,
    propagate_version = true,
)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a NestJS artifact
    #[command(alias = "g")]
    Generate(commands::generate::GenerateArgs),

    /// Initalise forge.json in the current directory
    Init,

    /// Show forge environment info
    Info,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initalize tracing. RUST_LOG env var overrides --verbose.
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::from_default_env().add_directive(Level::WARN.into())
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .init();

    match cli.command {
        Commands::Generate(args) => commands::generate::run(args),
        Commands::Info => commands::info::run(),
        Commands::Init => commands::init::run(),
    }
}
