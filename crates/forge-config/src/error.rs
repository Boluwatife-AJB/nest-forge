use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
  #[error("Could not read config file at '{path}': {source}")]
  ReadFailed {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("forge.json is not valid JSON: {reason}")]
  InvalidJson {reason:String},

  #[error("forge.json field '{field}' has invalid value: {reason}")]
  InvalidField {field: String, reason: String}
}

pub type ConfigResult<T> = Result<T, ConfigError>;