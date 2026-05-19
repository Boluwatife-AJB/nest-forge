
use clap::{Parser, Subcommand, CommandFactory};
use clap_complete::Shell;
use tracing::Level;
use tracing_subscriber::EnvFilter;

mod commands;
mod errors;
mod output;
mod theme;

#[derive(Parser)]
#[command(
    name="forge",
    version,
    about = "A NestJS artifact generator, forged in Rust",
    long_about = "forge generates NestJS artifacts (modules, services, controllers, and more)\n\
                  with full TypeScript support, spec file generation, and project-aware\n\
                  path resolution via forge.json.\n\n\
                  Run `forge init` to create a forge.json in your project root.",
    propagate_version = true,
    after_help = "Examples:\n  \
                  forge init\n  \
                  forge generate service users\n  \
                  forge g controller auth --flat\n  \
                  forge g dto create-user --dry-run",
)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Disable coloured output (auto-detected for pipes and CI)
    #[arg(long, global = true, env = "NO_COLOR")]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a NestJS artifact
    #[command(alias = "g")]
    Generate(commands::generate::GenerateArgs),

    /// Initalize forge.json in the current directory
    Init,

    /// Show forge environment info
    Info,

    /// Generate shell completions for the specified shell
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    }
}

fn main() {
    let cli = Cli::parse();

    // Color detection
    if cli.no_color || !supports_color() {
        console::set_colors_enabled(false);
        console::set_colors_enabled_stderr(false);
    }

    // Tracing setup
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::from_default_env().add_directive(Level::WARN.into())
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .with_writer(std::io::stderr)
        .init();

    // Command dispatch
    let result = match cli.command {
        Commands::Generate(args) => commands::generate::run(args),
        Commands::Info => commands::info::run(),
        Commands::Init => commands::init::run(),
        Commands::Completions { shell } => run_completions(shell),
    };

    // Error rendering
    if let Err(err) = result {
        errors::render_error(&err);
        std::process::exit(1);
    }
}

fn run_completions(shell: Shell) -> anyhow::Result<()> {
    use clap_complete::generate;
    use std::io;

    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}


fn supports_color() -> bool {
    supports_color::on(supports_color::Stream::Stdout).is_some()
}