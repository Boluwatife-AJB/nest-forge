//! Snapshot tests for all artifact templates.
//!
//! These tests render every artifact template with a fixed input
//! and compare the output against a saved snapshot file.

use forge_template::engine::{NameContext, TemplateContext, TemplateEngine};
use heck::{ToKebabCase, ToLowerCamelCase, ToPascalCase, ToSnakeCase};

fn test_ctx(name: &str) -> TemplateContext {
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

fn engine() -> TemplateEngine {
    TemplateEngine::new().expect("engine must initialize from embedded templates")
}

/// Render all files for an artifact and snapshot each one.
/// The snapshot name includes the artifact type and filename
/// so you can clearly see which template regressed.
macro_rules! snapshot_artifact {
    ($test_name:ident, $artifact:expr, $name:expr, $spec:expr) => {
        #[test]
        fn $test_name() {
            let engine = engine();
            let ctx = test_ctx($name);
            let files = engine
                .render($artifact, $name, &ctx, $spec)
                .expect(concat!("should render ", $artifact));

            for file in &files {
                let snapshot_name = format!(
                    "{}__{}",
                    $artifact,
                    file.relative_path
                        .to_string_lossy()
                        .replace(['/', '\\', '.'], "_")
                );

                insta::assert_snapshot!(snapshot_name, file.contents);
            }
        }
    };
}

// Single word names
snapshot_artifact!(snapshot_module, "module", "products", true);
snapshot_artifact!(snapshot_service, "service", "products", true);
snapshot_artifact!(snapshot_controller, "controller", "products", true);
snapshot_artifact!(snapshot_guard, "guard", "products", true);
snapshot_artifact!(snapshot_interceptor, "interceptor", "products", true);
snapshot_artifact!(snapshot_middleware, "middleware", "products", true);
snapshot_artifact!(snapshot_pipe, "pipe", "products", true);
snapshot_artifact!(snapshot_filter, "filter", "products", true);
snapshot_artifact!(snapshot_resolver, "resolver", "products", true);
snapshot_artifact!(snapshot_entity, "entity", "products", true);
snapshot_artifact!(snapshot_dto, "dto", "products", true);
snapshot_artifact!(snapshot_decorator, "decorator", "products", true);
snapshot_artifact!(snapshot_strategy, "strategy", "products", true);
snapshot_artifact!(snapshot_interface, "interface", "products", true);
snapshot_artifact!(snapshot_class, "class", "products", true);
snapshot_artifact!(snapshot_config, "config", "products", true);

// Multi-word names
snapshot_artifact!(
    snapshot_service_multiword,
    "service",
    "product_categories",
    false
);
snapshot_artifact!(
    snapshot_controller_multiword,
    "controller",
    "product_categories",
    false
);
snapshot_artifact!(
    snapshot_entity_multiword,
    "entity",
    "product_categories",
    false
);
snapshot_artifact!(
    snapshot_dto_multiword,
    "dto",
    "create_product_category_dto",
    false
);
