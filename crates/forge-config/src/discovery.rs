use std::path::{Path, PathBuf};
use tracing::{debug, instrument};

pub const CONFIG_FILE_NAME: &str = "forge.json";

#[instrument(fields(start = %start.display()))]
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();

    loop {
        let candidate = current.join(CONFIG_FILE_NAME);
        debug!(checking = %candidate.display());

        if candidate.is_file() {
            debug!(found = %candidate.display(), "Found forge.json");
            return Some(current);
        }

        // Move up one directory level
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                debug!("Reached root directory, no forge.json found");
                return None;
            }
        }
    }
}

pub fn find_project_root_or_fallback(start: &Path) -> (PathBuf, bool) {
    match find_project_root(start) {
        Some(root) => (root, true), // (root, found_config)
        None => (start.to_path_buf(), false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_project(structure: &[&str]) -> TempDir {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        for path in structure {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&full, "{}").unwrap();
        }
        dir
    }

    #[test]
    fn finds_config_in_current_directory() {
        let project = setup_project(&["forge.json"]);
        let found = find_project_root(project.path());
        assert_eq!(found, Some(project.path().to_path_buf()));
    }

    #[test]
    fn finds_config_in_parent_directory() {
        let project = setup_project(&["forge.json", "src/products/.gitkeep"]);
        let subdir = project.path().join("src/products");

        let found = find_project_root(&subdir);
        assert_eq!(found, Some(project.path().to_path_buf()));
    }

    // fn finds_config_in_grandparent_directory() {
    //   let project = setup_project(&["forge.json", "src/modules/products/dto/.gitkeep"]);
    //   let deepdir = project.path().join("src/modules/products/dto");

    //   let found = find_project_root(&deepdir);
    //   assert_eq!(found, Some(project.path().to_path_buf()));
    // }

    #[test]
    fn returns_none_when_no_config_exists() {
        let dir = tempfile::tempdir().unwrap();
        // No forge.json file anywhere
        let found = find_project_root(dir.path());
        assert!(found.is_none(), "should return None when no config found");
    }

    #[test]
    fn fallback_returns_start_path_when_no_config_found() {
        let dir = tempfile::tempdir().unwrap();
        let (root, found) = find_project_root_or_fallback(dir.path());
        assert_eq!(root, dir.path());
        assert!(!found, "should report config was not found");
    }
}
