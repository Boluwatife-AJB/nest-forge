use std::path::PathBuf;
use thiserror::Error;

/// The canonical error type for all forge-core operations.
///
/// Each variant is specific enough that callers can pattern-match on it
/// and give the user a meaningful message. This is the difference
/// between "an error occurred" and "the template 'service' was not found".
#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("Artifact type '{input}' is not supported\n {suggestion}")]
    UnsupportedArtifact { input: String, suggestion: String },

    #[error("Invalid artifact name '{name}': {reason}")]
    InvalidName { name: String, reason: String },

    #[error("File already exists at '{path}'\n Hint: use --force to overwrite")]
    FileAlreadyExists { path: PathBuf },

    #[error("Failed to create directory '{path}': {source}")]
    DirectoryCreation {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to read '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Template '{name}' failed to render: {reason}")]
    TemplateRender { name: String, reason: String },

    #[error("Template '{name}' not found")]
    TemplateNotFound { name: String },

    #[error("Config error: {0}")]
    ConfigParse(String),
}

pub type ForgeResult<T> = Result<T, ForgeError>;
