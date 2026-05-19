use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get a Command pointer at the forge binary.
fn forge() -> Command {
    Command::cargo_bin("forge").expect("forge binary must be compiled")
}

/// Create a temporary directory with an optional forge.json
fn temp_project(forge_json: Option<&str>) -> TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    if let Some(config) = forge_json {
        fs::write(dir.path().join("forge.json"), config).expect("failed to write forge.json");
    }
    dir
}

// Basic invocation
#[test]
fn shows_help_with_no_args() {
    forge()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn shows_version() {
    forge()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("forge"));
}

#[test]
fn info_command_succeeds() {
    forge()
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("forge"));
}

// Generate - argument validation

#[test]
fn generate_requires_artifact_and_name() {
    forge().args(["generate"]).assert().failure();
}

#[test]
fn generate_rejects_unknown_artifact() {
    let dir = temp_project(None);
    forge()
        .current_dir(dir.path())
        .args(["generate", "foobar", "products"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown artifact"));
}

#[test]
fn generate_suggests_correction_for_typo() {
    let dir = temp_project(None);
    forge()
        .current_dir(dir.path())
        .args(["generate", "servce", "products"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("service"));
}

#[test]
fn generate_rejects_name_with_space() {
    let dir = temp_project(None);
    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "my service"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("spaces"));
}

#[test]
fn generate_rejects_name_starting_with_digit() {
    let dir = temp_project(None);
    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "3products"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("letter"));
}

// Generate - dry run

#[test]
fn dry_run_creates_no_files() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "products", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"));

    // Confirm nothing was actually written
    let entries: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
    assert!(
        entries.is_empty(),
        "dry run must not write any files, but found: {:?}",
        entries
            .iter()
            .map(|e| e.as_ref().unwrap().path())
            .collect::<Vec<_>>()
    )
}

#[test]
fn dry_run_shows_files_that_would_be_created() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "products", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("products.service.ts"))
        .stdout(predicate::str::contains("CREATE"));
}

// Generate - real file creation
#[test]
fn generates_service_files() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "products"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CREATE"));

    let service_file = dir
        .path()
        .join("src")
        .join("products")
        .join("products.service.ts");
    assert!(service_file.exists());

    let contents = fs::read_to_string(&service_file).unwrap();
    assert!(contents.contains("ProductsService"));
    assert!(contents.contains("@Injectable("));
}

#[test]
fn generate_module_files() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "module", "auth"])
        .assert()
        .success();

    let module_file = dir.path().join("src").join("auth").join("auth.module.ts");
    assert!(module_file.exists());

    let contents = fs::read_to_string(&module_file).unwrap();
    assert!(contents.contains("AuthModule"), "should contain class name");
    assert!(contents.contains("@Module("), "should contain decorator");
}

#[test]
fn generates_controller_with_correct_route() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "controller", "products"])
        .assert()
        .success();

    let controller_file = dir
        .path()
        .join("src")
        .join("products")
        .join("products.controller.ts");

    let contents = fs::read_to_string(&controller_file).unwrap();

    assert!(contents.contains("ProductsController"));
    assert!(contents.contains("@Controller('products')"));
}

#[test]
fn generate_spec_file_by_default() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "products"])
        .assert()
        .success();

    let spec_file = dir
        .path()
        .join("src")
        .join("products")
        .join("products.service.spec.ts");

    assert!(
        spec_file.exists(),
        "spec file should be generated by default"
    );
}

#[test]
fn no_spec_flag_suppresses_spec_file() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "products", "--spec=false"])
        .assert()
        .success();

    let spec_file = dir.path().join("products").join("products.service.spec.ts");

    assert!(
        !spec_file.exists(),
        "spec file should not be generated when --spec=false"
    );
}

// Generate - flat flag
#[test]
fn flat_flag_skips_subdirectory() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "products", "--flat=true"])
        .assert()
        .success();

    // With --flat, file should be directly in cwd, not in users
    let flat_file = dir.path().join("src").join("products.service.ts");
    let nested_file = dir
        .path()
        .join("src")
        .join("products")
        .join("products.service.ts");

    assert!(
        flat_file.exists(),
        "flat file should exist at {:?}",
        flat_file
    );
    assert!(!nested_file.exists(), "nested file should NOT with --flat");
}

// Generate - aliases
#[test]
fn g_is_alias_for_generate() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["g", "service", "products", "--dry-run"])
        .assert()
        .success();
}

#[test]
fn artifact_aliases_work() {
    let dir = temp_project(None);

    // "s" is alias for "service"
    forge()
        .current_dir(dir.path())
        .args(["g", "s", "products", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("service"));

    // "m" is alias for "module"
    forge()
        .current_dir(dir.path())
        .args(["g", "mo", "auth", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("module"));
}

// Multiword names
#[test]
fn multiword_kebab_name_generates_correct_files() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["g", "service", "product-category"])
        .assert()
        .success();

    let service_file = dir
        .path()
        .join("src")
        .join("product-category")
        .join("product-category.service.ts");

    assert!(service_file.exists(), "file should use kebab name");

    let contents = fs::read_to_string(&service_file).unwrap();
    assert!(
        contents.contains("ProductCategoryService"),
        "should use PascalCase class name"
    );
    assert!(
        !contents.contains("product-categoryService"),
        "no kebab casing in class name"
    );
}

// Config awareness

#[test]
fn respects_source_root_from_forge_json() {
    let config = r#"{ "sourceRoot": "app", "generateSpec": false }"#;
    let dir = temp_project(Some(config));

    forge()
        .current_dir(dir.path())
        .args(["g", "service", "products"])
        .assert()
        .success()
        .stdout(predicate::str::contains("app").and(predicate::str::contains("products")));

    // Should go into app/ not src/
    let file = dir
        .path()
        .join("app")
        .join("products")
        .join("products.service.ts");
    assert!(file.exists(), "file should be in app/, not src/");
}

#[test]
fn respects_path_override_from_forge_json() {
    let config = r#"{ "generateSpec": false, "paths": { "entity": "src/database/entities" } }"#;

    let dir = temp_project(Some(config));

    forge()
        .current_dir(dir.path())
        .args(["g", "entity", "product"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("src/database/entities")
                .and(predicate::str::contains("product.entity.ts")),
        );

    let file = dir
        .path()
        .join("src/database/entities")
        .join("product")
        .join("product.entity.ts");

    assert!(
        file.exists(),
        "entity should go to configured path, no default src/ directory"
    );
}

#[test]
fn cli_spec_flag_overrides_forge_json() {
    // forge.json says no spec, but --spec=true on CLI should win
    let config = r#"{ "generateSpec": false }"#;
    let dir = temp_project(Some(config));

    forge()
        .current_dir(dir.path())
        .args(["g", "service", "products", "--spec=true"])
        .assert()
        .success()
        .stdout(predicate::str::contains("products.service.spec.ts"));

    let spec_file = dir
        .path()
        .join("src")
        .join("products")
        .join("products.service.spec.ts");

    println!("{:?}", spec_file);

    assert!(
        spec_file.exists(),
        "CLI --spec=true should override forge.json generateSpec=false"
    );
}

// Idempotency
#[test]
fn second_generate_skips_existing_files() {
    let dir = temp_project(None);

    // Forge run
    forge()
        .current_dir(dir.path())
        .args(["g", "service", "products"])
        .assert()
        .success();

    // Second run should skip existing files not fail
    forge()
        .current_dir(dir.path())
        .args(["g", "service", "products"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SKIP"));
}

// Init command
#[test]
fn init_creates_forge_json() {
    let dir = tempfile::tempdir().unwrap();

    forge()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("forge.json"));

    assert!(
        dir.path().join("forge.json").exists(),
        "forge.json should be created in current directory"
    );

    // Verify it's valid JSON
    let contents = fs::read_to_string(dir.path().join("forge.json")).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&contents).expect("forge init should write valid JSON");

    assert_eq!(
        parsed["sourceRoot"], "src",
        "default sourceRoot should be 'src'"
    );
    assert_eq!(
        parsed["generateSpec"], true,
        "default generateSpec should be true"
    );
}

#[test]
fn init_fails_if_forge_json_already_exists() {
    let dir = temp_project(Some(r#"{ "sourceRoot": "src", "generateSpec": false }"#));

    forge()
        .current_dir(dir.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("forge.json already exists"));
}

// Error output quality
#[test]
fn error_output_is_on_stderr_not_stdout() {
    let dir = temp_project(None);

    let output = forge()
        .current_dir(dir.path())
        .args(["generate", "foobar", "products"])
        .output()
        .unwrap();

    assert!(
        output.stdout.is_empty() || String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "error must go to stderr, not stdout"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown artifact"));
}

#[test]
fn error_includes_hint() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "foobar", "products"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("hint:").or(predicate::str::contains("forge generate --help")));
}

// Shell completions
#[test]
fn completions_bash_outputs_shell_script() {
    forge()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("forge"));
}

#[test]
fn completions_zsh_outputs_shell_scripts() {
    forge()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish_outputs_shell_script() {
    forge()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// Info command

#[test]
fn info_shows_version() {
    forge()
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn info_shows_source_root() {
    let dir = temp_project(Some(r#"{ "sourceRoot": "app" } "#));

    forge()
        .current_dir(dir.path())
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("app"));
}

#[test]
fn info_reports_missing_config() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("not found"));
}

#[test]
fn info_shows_path_overrides_when_configured() {
    let config = r#"{ "paths": { "entity": "src/database/entities" } }"#;
    let dir = temp_project(Some(config));

    forge()
        .current_dir(dir.path())
        .arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("entity"))
        .stdout(predicate::str::contains("src/database/entities"));
}

// NO color support
#[test]
fn no_color_flag_produces_plain_output() {
    let dir = temp_project(None);

    let output = forge()
        .current_dir(dir.path())
        .args(["--no-color", "generate", "service", "products", "dry-run"])
        .output()
        .unwrap();

    assert!(
        output.stdout.is_empty() || String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "output should be plain text, not colored"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\x1b"), "output should contain no ANSI escape codes with --no-color");
}

#[test]
fn no_color_env_var_produces_plain_output() {
    let dir = temp_project(None);

    let output = forge()
        .current_dir(dir.path())
        .env("NO_COLOR", "1")
        .args(["generate", "service", "products", "dry-run"])
        .output()
        .unwrap();


    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\x1b"), "NO_COLOR env var should disable colours");
}

// Exit codes
#[test]
fn success_exits_zero() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "service", "products", "--dry-run"])
        .assert()
        .success()
        .code(0);
}

#[test]
fn error_exits_nonzero() {
    let dir = temp_project(None);

    forge()
        .current_dir(dir.path())
        .args(["generate", "foobar", "products"])
        .assert()
        .failure()
        .code(1);

}

#[test]
fn displayed_paths_use_forward_slashes() {
    let dir = temp_project(None);

    let output = forge()
        .current_dir(dir.path())
        .args(["generate", "service", "products", "--dry-run"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("products.service.ts"));
    assert!(!stdout.contains("\\"), "displayed paths should use forward slashes, got: \n{stdout}");
}