use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

use crate::ForgeJsonFile;
use crate::ResolvedConfig;
use crate::defaults::ConfigDefaults;
use crate::error::{ConfigError, ConfigResult};

#[derive(Debug, Default)]
pub struct CliOverrides {
    pub generate_spec: Option<bool>,
    pub flat: Option<bool>,
    pub source_root: Option<String>,
}

pub fn resolve_config(
    project_root: &Path,
    file_config: Option<&ForgeJsonFile>,
    cli_overrides: &CliOverrides,
) -> ConfigResult<ResolvedConfig> {
    let source_root_str = cli_overrides
        .source_root
        .as_deref()
        .or_else(|| file_config.and_then(|c| c.source_root.as_deref()))
        .unwrap_or(ConfigDefaults::source_root());

    let source_root = project_root.join(source_root_str);
    debug!(source_root = %source_root.display(), "Resolved source root");

    // Language
    let language = file_config
        .and_then(|c| c.language.as_deref())
        .unwrap_or(ConfigDefaults::language())
        .to_string();

    validate_language(&language)?;

    // Generate spec
    let generate_spec = cli_overrides
        .generate_spec
        .or_else(|| file_config.and_then(|c| c.generate_spec))
        .unwrap_or(ConfigDefaults::generate_spec());

    // Flat
    let flat = cli_overrides
        .flat
        .or_else(|| file_config.and_then(|c| c.flat))
        .unwrap_or(ConfigDefaults::flat());

    // paths: converts relative string path in the config to absolute PathBufs
    let paths = resolve_path_overrides(project_root, file_config.and_then(|c| c.paths.as_ref()))?;

    Ok(ResolvedConfig {
        project_root: project_root.to_path_buf(),
        source_root,
        language,
        generate_spec,
        flat,
        paths,
    })
}

/// Resolve per-artifact path overrides to absolute paths.
fn resolve_path_overrides(
    project_root: &Path,
    raw_paths: Option<&HashMap<String, String>>,
) -> ConfigResult<HashMap<String, PathBuf>> {
    let Some(raw) = raw_paths else {
        return Ok(HashMap::new());
    };

    raw.iter()
        .map(|(artifact, rel_path)| {
            // Reject paths that try to escape the project root
            if rel_path.starts_with("..") {
                return Err(ConfigError::InvalidField {
                    field: format!("paths.{artifact}"),
                    reason: format!("path '{rel_path}' must not escape project root"),
                });
            }

            Ok((artifact.clone(), project_root.join(rel_path)))
        })
        .collect()
}

fn validate_language(lang: &str) -> ConfigResult<()> {
    match lang {
        "ts" | "js" => Ok(()),
        other => Err(ConfigError::InvalidField {
            field: "language".to_string(),
            reason: format!("'{other}' is not supported: use 'ts' or 'js'"),
        }),
    }
}

#[cfg(test)]
mod tests {
  use super::*;

  const FAKE_ROOT : &str = "/project";

  fn root() -> &'static Path {
    Path::new(FAKE_ROOT)
  }

  // Defaults
  #[test]
  fn all_defaults_when_no_config_and_no_overrides() {
    let config = resolve_config(root(), None, &CliOverrides::default()).expect("should resolve with no inputs");

    assert_eq!(config.source_root, Path::new("/project/src"));
    assert_eq!(config.language, "ts");
    assert!(config.generate_spec);
    assert!(!config.flat);
    assert!(config.paths.is_empty());
  }

  // File config overrides defaults
  #[test]
  fn file_config_overrides_source_root() {
    let file = ForgeJsonFile {
      source_root: Some("app".to_string()),
      ..Default::default()
    };

    let config = resolve_config(root(), Some(&file), &CliOverrides::default()).expect("should resolve with file config");

    assert_eq!(config.source_root, Path::new("/project/app"));
  }

  #[test]
  fn file_can_disable_spec_generation() {
    let file = ForgeJsonFile {
      generate_spec: Some(false),
      ..Default::default()
    };

    let config = resolve_config(root(), Some(&file), &CliOverrides::default()).expect("should resolve with file config");

    assert!(!config.generate_spec);
  }

  #[test]
  fn file_config_path_overrides_are_resolved_to_absolute() {
    let mut paths = HashMap::new();
    paths.insert("entity".into(), "src/database/entities".into());


    let file = ForgeJsonFile {
      paths: Some(paths),
      ..Default::default()
    };

    let config = resolve_config(root(), Some(&file), &CliOverrides::default()).expect("should resolve with file config");

    assert_eq!(config.paths.get("entity"), Some(&PathBuf::from("/project/src/database/entities")));
  }

  // CLI Overrides
  #[test]
  fn cli_overrides_beats_file_config() {
  let overrides = CliOverrides {
    generate_spec: Some(false),
    ..Default::default()
  };

  let config = resolve_config(root(), None, &overrides).expect("should resolve with file config and cli overrides");
  assert!(!config.flat);
}


// Validation
#[test]
fn invalid_language_is_rejected() {
  let file = ForgeJsonFile {
    language: Some("rust".to_string()),
    ..Default::default()
};

  let result = resolve_config(root(), Some(&file), &CliOverrides::default());
  assert!(matches!(result, Err(ConfigError::InvalidField { field, .. }) if field == "language"), "should reject InvalidField for bad language");
}

#[test]
fn path_escape_attempt_returns_error() {
  let mut paths = HashMap::new();
  paths.insert("entity".into(), "../outside-project".into());

  let file = ForgeJsonFile {
    paths: Some(paths),
    ..Default::default()
};

  let result = resolve_config(root(), Some(&file), &CliOverrides::default());
  assert!(matches!(result, Err(ConfigError::InvalidField { .. })), "paths that escape the project root should be rejected");
}

#[test]
fn output_path_for_returns_override_when_set() {
  let mut paths = HashMap::new();
  paths.insert("entity".into(), PathBuf::from("project/src/db"));

  let config = ResolvedConfig {
    project_root: PathBuf::from("project"),
    source_root: PathBuf::from("project/src"),
    language: "ts".to_string(),
    generate_spec: true,
    flat: false,
    paths,
  };

  assert_eq!(config.output_path_for("entity"), &PathBuf::from("project/src/db"));
}

#[test]
fn output_path_for_falls_back_to_source_root_path_when_not_set() {
  let config = ResolvedConfig {
    project_root: PathBuf::from("project"),
    source_root: PathBuf::from("project/src"),
    language: "ts".to_string(),
    generate_spec: true,
    flat: false,
    paths: HashMap::new(),
  };

  assert_eq!(config.output_path_for("service"), &PathBuf::from("project/src"));
}
}