use crate::error::ForgeResult;
use std::path::Path;

/// An abstraction over filesystem operations.
///
/// The real implementation uses std::fs.
/// The test implementation uses an in-memory HashMap.
pub trait FileSystem: Send + Sync {
    fn create_dir_all(&self, path: &Path) -> ForgeResult<()>;
    fn write_file(&self, path: &Path, contents: &str) -> ForgeResult<()>;
    fn file_exists(&self, path: &Path) -> bool;
    fn read_file(&self, path: &Path) -> ForgeResult<String>;
}

/// The real filesystem wraps std::fs
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn create_dir_all(&self, path: &Path) -> ForgeResult<()> {
        std::fs::create_dir_all(path).map_err(|source| {
            crate::error::ForgeError::DirectoryCreation {
                path: path.to_path_buf(),
                source,
            }
        })
    }

    fn write_file(&self, path: &Path, contents: &str) -> ForgeResult<()> {
        std::fs::write(path, contents).map_err(|source| crate::error::ForgeError::FileWrite {
            path: path.to_path_buf(),
            source,
        })
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_file(&self, path: &Path) -> ForgeResult<String> {
        std::fs::read_to_string(path).map_err(|source| crate::error::ForgeError::FileWrite {
            path: path.to_path_buf(),
            source,
        })
    }
}

/// In-memory filesystem for tests
#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    pub struct InMemoryFileSystem {
        files: Arc<Mutex<HashMap<PathBuf, String>>>,
    }

    impl InMemoryFileSystem {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn written_files(&self) -> HashMap<PathBuf, String> {
            self.files.lock().unwrap().clone()
        }
    }

    impl FileSystem for InMemoryFileSystem {
        fn create_dir_all(&self, _path: &Path) -> ForgeResult<()> {
            Ok(())
        }

        fn write_file(&self, path: &Path, contents: &str) -> ForgeResult<()> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), contents.to_string());
            Ok(())
        }

        fn file_exists(&self, path: &Path) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }

        fn read_file(&self, path: &Path) -> ForgeResult<String> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| crate::error::ForgeError::TemplateNotFound {
                    name: path.display().to_string(),
                })
        }
    }
}
