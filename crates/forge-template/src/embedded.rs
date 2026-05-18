use rust_embed::Embed;

/// Embeds the entire `templates/` directory into the binary at compile time.
/// The path is relative to the workspace root (where Cargo.toml lives).

#[derive(Embed)]
#[folder = "../../templates"]
pub struct EmbeddedTemplates;
