use std::path::Path;
use tracing::{debug, instrument};

use crate::error::{ConfigError, ConfigResult};
use crate::ForgeJsonFile;

#[instrument(fields(path = %path.display()))]
pub fn parse_forge_json(path: &Path) -> ConfigResult<ForgeJsonFile> {
  debug!("Reading config file");

  let contents = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadFailed { path: path.to_path_buf(), source: e })?;

  parse_forge_json_str(&contents)
}


pub fn parse_forge_json_str(contents: &str) -> ConfigResult<ForgeJsonFile> {
  serde_json::from_str(contents).map_err(|e| ConfigError::InvalidJson { reason: e.to_string() })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_full_config() {
    let json = r#"
    {
      "sourceRoot": "app",
      "language": "ts",
      "generateSpec": false,
      "flat": true,
      "paths": {
        "entity": "src/db/entities"
      }
    }"#;

    let config = parse_forge_json_str(json).expect("should parse");
    assert_eq!(config.source_root.as_deref(), Some("app"));
    assert_eq!(config.generate_spec, Some(false));
    assert_eq!(config.flat, Some(true));
    assert!(config.paths.is_some());
  }

  #[test]
  fn parses_empty_config_as_all_none() {
    let config = parse_forge_json_str("{}").expect("should parse empty object");

    assert!(config.source_root.is_none());
        assert!(config.language.is_none());
        assert!(config.generate_spec.is_none());
  }

  #[test]
  fn rejects_invalid_json() {
    let result = parse_forge_json_str("{sourceRoot: oops}");

    assert!(matches!(result, Err(ConfigError::InvalidJson { .. })), "should return InvalidJson for bad JSON")
  } 

  #[test]
  fn reject_wrong_types() {
    // generateSpec must be bool, not string
    let result = parse_forge_json_str(r#"{"generateSpec": "yes"}"#);

    assert!(result.is_err(), "should reject wrong type for generateSpec")

  }
}