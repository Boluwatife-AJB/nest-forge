mod defaults;
mod discovery;
mod error;
mod merge;
mod parser;

pub use defaults::ConfigDefaults;
pub use discovery::{find_project_root, find_project_root_or_fallback};
pub use error::{ConfigError, ConfigResult};
pub use merge::{resolve_config, CliOverrides};
pub use parser::parse_forge_json;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Raw shape of the forge.json file
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ForgeJsonFile {
    pub source_root: Option<String>,
    pub language: Option<String>,
    pub generate_spec: Option<bool>,
    pub flat: Option<bool>,
    pub paths: Option<HashMap<String, String>>,
}

// Fully resolved configuration
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub project_root: PathBuf,
    pub source_root: PathBuf,
    pub language: String,
    pub generate_spec: bool,
    pub flat: bool,
    pub paths: HashMap<String, PathBuf>,
}

impl ResolvedConfig {
  pub fn output_path_for(&self, artifact_kind: &str) -> &PathBuf {
    self.paths.get(artifact_kind).unwrap_or(&self.source_root)
  }
}