use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument, warn};

use crate::artifact::GenerationRequest;
use crate::error::{ForgeError, ForgeResult};
use crate::fs::FileSystem;
use forge_template::engine::{NameContext, TemplateContext, TemplateEngine};

/// The result of a generation operation.
#[derive(Debug)]
pub struct GenerationOutput {
    pub files: Vec<GeneratedFile>,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub skipped: bool,
}

/// The main artifact generator.
pub struct Generator<F: FileSystem> {
    fs: F,
    engine: TemplateEngine,
}

impl<F: FileSystem> Generator<F> {
    pub fn new(fs: F) -> ForgeResult<Self> {
        let engine = TemplateEngine::new().map_err(|e| ForgeError::TemplateRender {
            name: "engine_init".into(),
            reason: e.to_string(),
        })?;

        Ok(Self { fs, engine })
    }

    /// Execute a generation request end-to-end
    #[instrument(skip(self), fields(
    artifact = %request.kind.template_name(),
    name = %request.name.kebab,
    dry_run = %request.dry_run,
  ))]
    pub fn generate(&self, request: &GenerationRequest) -> ForgeResult<GenerationOutput> {
        debug!("Building template context");

        let ctx = TemplateContext {
            name: NameContext {
                raw: request.name.raw.clone(),
                pascal: request.name.pascal.clone(),
                camel: request.name.camel.clone(),
                snake: request.name.snake.clone(),
                kebab: request.name.kebab.clone(),
            },
        };

        let rendered_files = self
            .engine
            .render(
                request.kind.template_name(),
                &request.name.kebab,
                &ctx,
                request.generate_spec,
            )
            .map_err(|e| ForgeError::TemplateRender {
                name: request.kind.template_name().into(),
                reason: e.to_string(),
            })?;

        let mut output_files = Vec::new();

        for rendered_file in rendered_files {
            let file_path = self.resolve_output_path(request, &rendered_file.relative_path);

            debug!(path = %file_path.display(), "Resolved output path");

            // Check for existing files
            if self.fs.file_exists(&file_path) {
                warn!(path = %file_path.display(), "File already exists, skipping");
                output_files.push(GeneratedFile {
                    path: file_path,
                    skipped: true,
                });
                continue;
            }

            if !request.dry_run {
                // Ensure parent directories exist
                if let Some(parent) = file_path.parent() {
                    self.fs.create_dir_all(parent)?;
                }

                self.fs.write_file(&file_path, &rendered_file.contents)?;
                info!(path = %file_path.display(), "Generated file");
            }

            output_files.push(GeneratedFile {
                path: file_path,
                skipped: false,
            });
        }

        Ok(GenerationOutput {
            files: output_files,
            dry_run: request.dry_run,
        })
    }

    /// Resolve the output path for a rendered file.
    fn resolve_output_path(&self, request: &GenerationRequest, relative_file: &Path) -> PathBuf {
        if request.flat {
            request.output_path.join(relative_file)
        } else {
            request
                .output_path
                .join(&request.name.kebab)
                .join(relative_file)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{ArtifactKind, ArtifactName};
    use crate::fs::test_support::InMemoryFileSystem;
    use std::path::PathBuf;

    fn make_request(kind: ArtifactKind, name: &str) -> GenerationRequest {
        GenerationRequest {
            kind,
            name: ArtifactName::new_unchecked(name),
            output_path: PathBuf::from("/output"),
            flat: false,
            dry_run: false,
            generate_spec: false,
        }
    }

    fn make_generator() -> Generator<InMemoryFileSystem> {
        Generator::new(InMemoryFileSystem::new()).expect("generator should initialize")
    }

    #[test]
    fn generates_service_files_at_correct_paths() {
        let generator = make_generator();
        let req = make_request(ArtifactKind::Service, "products");
        let output = generator.generate(&req).expect("should generate");

        assert!(!output.files.is_empty());
        let paths: Vec<String> = output
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().to_string())
            .collect();

        // Should be nested in a directory named after the artifact
        assert!(
            paths
                .iter()
                .any(|p| p.contains("products") && p.contains("service")),
            "expected a products service file, got: {:?}",
            paths
        );
    }

    

    #[test]
    fn flat_flag_skips_subdirectory() {
        let fs = InMemoryFileSystem::new();
        let generator = Generator::new(fs.clone()).expect("should init");
        let mut req = make_request(ArtifactKind::Service, "products");
        req.flat = true;

        generator.generate(&req).expect("should generate flat");

        let files = fs.written_files();
        for path in files.keys() {
            // With --flat, path should NOT contain /products/products/
            // it should be /output/products.service.ts directly
            let components: Vec<_> = path.components().collect();
            let has_double_products = components.windows(2).any(|w| {
                w[0].as_os_str() == "products"
                    && w[1].as_os_str().to_string_lossy().starts_with("products")
            });
            assert!(
                !has_double_products,
                "--flat should not create subdirectory, path was: {}",
                path.display()
            )
        }
    }

    #[test]
    fn dry_run_writes_no_files() {
        let fs = InMemoryFileSystem::new();
        let generator = Generator::new(fs.clone()).expect("should init");
        let mut req = make_request(ArtifactKind::Service, "products");
        req.dry_run = true;

        let output = generator.generate(&req).expect("should run");

        assert!(output.dry_run);
        assert!(
            fs.written_files().is_empty(),
            "dry run should not write any files"
        );
    }

    #[test]
    fn skips_existing_files() {
        let fs = InMemoryFileSystem::new();
        let generator = Generator::new(fs.clone()).expect("should init");
        let req = make_request(ArtifactKind::Service, "products");

        // First run
        generator.generate(&req).expect("first run");
        let written_count = fs.written_files().len();

        // Second run
        let output = generator.generate(&req).expect("second run");

        assert!(
            output.files.iter().all(|f| f.skipped),
            "all files should be skipped on second run"
        );
        assert_eq!(
            fs.written_files().len(),
            written_count,
            "should not write any new files on second run"
        );
    }

    #[test]
    fn controller_product_inventory_defines_product_controller() {
        let fs = InMemoryFileSystem::new();
        let generator = Generator::new(fs.clone()).expect("should init");

        let mut req = make_request(ArtifactKind::Controller, "product-inventory");
        req.generate_spec = true;

        generator.generate(&req).expect("controller generation should succeed");

        let has_class = fs.written_files().values().any(|src| src.contains("class ProductInventoryController"));
        assert!(has_class, "expected ProductInventoryController in written files, paths: {:?}", fs.written_files().keys().collect::<Vec<_>>());

       
    }
}
