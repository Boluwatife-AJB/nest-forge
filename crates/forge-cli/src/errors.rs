use console::style;

#[derive(Debug)]
enum ErrorKind {
  UserInput,
  Filesystem, 
  Config,
  Internal,
}

// Render an anyhow error chain to stderr with full formatting
pub fn render_error(err: &anyhow::Error) {
  let kind = classify_error(err);
  let (prefix, header_style) = match kind {
    ErrorKind::UserInput => ("error", style("error").red().bold()),
    ErrorKind::Filesystem => ("io error", style("io error").red().bold()),
    ErrorKind::Config => ("config", style("config error").yellow().bold()),
    ErrorKind::Internal => ("bug", style("internal error").red().bold()),
  };

  // primary error message
  eprintln!();
  eprintln!("   {} {}", header_style, style(err).bold());

  // Error chain
  let chain: Vec<_> = err.chain().skip(1).collect();
  if !chain.is_empty() {
    eprintln!();
    for cause in &chain {
      eprintln!("   {} {}", style("caused by:").dim(), style(cause).dim());
    }
  }

  // Contextual hints
  if let Some(hint) = extract_hint(err) {
    eprintln!();
    eprintln!("   {} {}", style("hint:").cyan().bold(), style(hint).cyan());
  }

  eprintln!();

  let _ = prefix;
}


pub fn _print_success(message: &str) {
  println!("  {} {}", style("✓").green().bold(), message);
}

pub fn print_info(label: &str, value: &str) {
  println!(
    "   {:>12}  {}",
    style(label).cyan().bold(),
    value
  )
}

pub fn print_warning(message: &str) {
  eprintln!("   {} {}", style("warning:").yellow().bold(), message)
}

pub fn print_section(title: &str) {
    println!();
    println!("  {}", style(title).bold().underlined());
    println!();
}

// Internal helpers
fn classify_error(err: &anyhow::Error) -> ErrorKind {
  let msg = err.to_string().to_lowercase();

  if msg.contains("invalid name") 
      || msg.contains("unknown artifact")
      || msg.contains("did you mean")
      || msg.contains("spaces")
      || msg.contains("letter")
  {
    return ErrorKind::UserInput;
  }

  if msg.contains("io error")
      || msg.contains("failed to create")
      || msg.contains("failed to write")
      || msg.contains("no such file")
      || msg.contains("permission") 
  {
      return ErrorKind::Filesystem;
  }

  if msg.contains("config")
      || msg.contains("forge.json")
      || msg.contains("invalid json") 
  {
    return ErrorKind::Config;
  }

  ErrorKind::Internal

}

fn extract_hint(err: &anyhow::Error) -> Option<&'static str> {
  let msg = err.to_string().to_lowercase();

  if msg.contains("unknown artifact") {
        return Some("Run `forge generate --help` to see all supported artifact types.");
    }

    if msg.contains("spaces") {
        return Some("Use hyphens to separate words: e.g. `user-profile`, not `user profile`.");
    }

    if msg.contains("already exists") {
        return Some("Use `--force` to overwrite existing files.");
    }

    if msg.contains("permission denied") {
        return Some("Check file permissions in the output directory.");
    }

    if msg.contains("forge.json") && msg.contains("invalid") {
        return Some("Run `forge init` to generate a valid forge.json template.");
    }

    if msg.contains("template") && msg.contains("not found") {
        return Some("This may be a bug — please report it at https://github.com/you/forge/issues");
    }

    None

}