use serde::Serialize;
use std::path::PathBuf;
use tera::Tera;
use tracing::{debug, instrument};

use crate::embedded::EmbeddedTemplates;

/// The data passed into every template render.
#[derive(Debug, Serialize)]
pub struct TemplateContext {
    /// All name variants of the artifact (pascal, camel, snake, kebab)
    pub name: NameContext,
}

#[derive(Debug, Serialize)]
pub struct NameContext {
    pub raw: String,
    pub snake: String,
    pub kebab: String,
    pub pascal: String,
    pub camel: String,
}

/// Describes a single file to be generated.
#[derive(Debug)]
pub struct RenderedFile {
    pub relative_path: PathBuf,
    pub contents: String,
}

/// The main engine for rendering templates.
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    /// Build the engine by loading all embedded templates.
    /// This is called once at startup.
    #[instrument(name = "TemplateEngine::new")]
    pub fn new() -> Result<Self, TemplateEngineError> {
        let mut tera = Tera::default();

        // Iterate over all embedded templates and add them to the Tera instance.
        // EmbeddedTemplates::iter() yields file paths as Cow<'static, str>.
        let templates: Vec<(String, String)> = EmbeddedTemplates::iter()
            .filter(|path| path.ends_with(".tera"))
            .map(|path| {
                let contents = EmbeddedTemplates::get(&path)
                    .expect("embedded file listed but not retrievable, this is a bug");

                let source = std::str::from_utf8(contents.data.as_ref())
                    .map_err(|_| TemplateEngineError::InvalidUtf8(path.to_string()))?;

                debug!(template = %path, "Registering embedded template");
                Ok((path.to_string(), source.to_string()))
            })
            .collect::<Result<_, TemplateEngineError>>()?;

        tera.add_raw_templates(templates)
            .map_err(|e| TemplateEngineError::Init(e.to_string()))?;

        Ok(Self { tera })
    }

    /// Render all templates for a given artifact type.
    ///
    /// Returns a vector of `RenderedFile` instances, one for each template.
    #[instrument(skip(self, ctx), fields(artifact = %artifact_kind))]
    pub fn render(
        &self,
        artifact_kind: &str,
        name_kebab: &str,
        ctx: &TemplateContext,
        include_spec: bool,
    ) -> Result<Vec<RenderedFile>, TemplateEngineError> {
        // Build tera ctx from the serializable struct.
        let tera_ctx =
            tera::Context::from_serialize(ctx).map_err(|e| TemplateEngineError::Render {
                template: artifact_kind.to_string(),
                reason: e.to_string(),
            })?;

        // Find all templates for this artifact kind.
        let prefix = format!("{}/", artifact_kind);
        let mut rendered = Vec::new();

        for template_path in self.tera.get_template_names() {
            if !template_path.starts_with(&prefix) {
                continue;
            }

            // Skip spec (test) files if not requested.
            if !include_spec && template_path.contains(".spec.") {
                continue;
            }

            let contents = self.tera.render(template_path, &tera_ctx).map_err(|e| {
                TemplateEngineError::Render {
                    template: template_path.to_string(),
                    reason: e.to_string(),
                }
            })?;

            // Compute the output filename from the template path.
            // "service/service.ts.tera" -> "user.service.ts"
            // "service/service.spec.ts.tera" -> "user.service.spec.ts"
            // let filename = template_path
            //     .strip_prefix(&prefix)
            //     .unwrap()
            //     .replace(".tera", "")
            //     .replace(artifa ct_kind, name_kebab);

            let s = template_path
                .strip_prefix(&prefix)
                .unwrap()
                .replace(".tera", "");

            let filename = if let Some(rest) = s.strip_prefix(&format!("{}.", artifact_kind)) {
                format!(
                    "{name}.{kind}.{rest}",
                    name = name_kebab,
                    kind = artifact_kind,
                    rest = rest
                )
            } else {
                s.replace(artifact_kind, name_kebab)
            };

            debug!(template = %template_path, output = %filename, "Rendered template");

            rendered.push(RenderedFile {
                relative_path: PathBuf::from(filename),
                contents,
            });
        }

        if rendered.is_empty() {
            return Err(TemplateEngineError::NotFound(artifact_kind.to_string()));
        }

        Ok(rendered)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateEngineError {
    #[error("Failed to initialize template engine: {0}")]
    Init(String),

    #[error("Template '{template}' failed to render: {reason}")]
    Render { template: String, reason: String },

    #[error("No templates found for this artifact type '{0}'")]
    NotFound(String),

    #[error("Template file '{0}' contains invalid UTF-8")]
    InvalidUtf8(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> TemplateEngine {
        TemplateEngine::new().expect("engine should initialize from embedded templates")
    }

    fn ctx(name: &str) -> TemplateContext {
        use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};
        TemplateContext {
            name: NameContext {
                raw: name.to_string(),
                snake: name.to_snake_case(),
                kebab: name.to_kebab_case(),
                pascal: name.to_pascal_case(),
                camel: name.to_lower_camel_case(),
            },
        }
    }

    #[test]
    fn renders_service_template() {
        let engine = engine();
        let files = engine
            .render("service", "users", &ctx("users"), false)
            .expect("should render service");

        assert_eq!(files.len(), 1, "without spec, should produce 1 file");

        let file = &files[0];
        assert!(
            file.contents.contains("UsersService"),
            "should contain Injectable decorator"
        )
    }

    #[test]
    fn renders_service_with_spec() {
        let engine = engine();
        let files = engine
            .render("service", "users", &ctx("users"), true)
            .expect("should render service with spec");

        assert_eq!(files.len(), 2, "with spec, should produce 2 files");

        let has_spec = files
            .iter()
            .any(|f| f.relative_path.to_string_lossy().contains(".spec."));
        assert!(has_spec, "one of the files should be a spec file")
    }

    #[test]
    fn multiword_name_casing_is_correct() {
        let engine = engine();
        let files = engine
            .render("service", "user-profile", &ctx("user-profile"), false)
            .expect("should render a multiword name");

        let contents = &files[0].contents;
        assert!(contents.contains("UserProfileService"), "PascalCase class");
        assert!(
            !contents.contains("user-profileService"),
            "should not have kebab in class name"
        )
    }

    #[test]
    fn unknown_artifact_returns_not_found_error() {
        let engine = engine();
        let result = engine.render("nonexistent", "foo", &ctx("foo"), false);
        assert!(
            matches!(result, Err(TemplateEngineError::NotFound(_))),
            "should return NotFound error for unknown artifact"
        );
    }

    #[test]
    fn all_artifact_types_have_at_least_one_template() {
        let engine = engine();
        let artifact_types = [
            "module",
            "service",
            "controller",
            "class",
            "dto",
            "guard",
            "interceptor",
            "middleware",
            "pipe",
            "decorator",
            "strategy",
            "interface",
            "filter",
            "config",
            "resolver",
            "entity",
        ];

        for artifact_type in artifact_types {
            let result = engine.render(artifact_type, "bar", &ctx("bar"), false);
            assert!(
                result.is_ok(),
                "artifact '{}' should have at least one template - got {:?}",
                artifact_type,
                result.err()
            );
        }
    }
}
