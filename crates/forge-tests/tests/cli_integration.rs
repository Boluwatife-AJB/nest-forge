//! Integration tests for the forge CLI binary.
//!
//! These tests use `assert_cmd` to run the real compiled binary
//! and check its stdout, stderr, and exit code.

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
  forge().assert().failure().stderr(predicate::str::contains("Usage"));
}

#[test]
fn shows_version() {
  forge().arg("--version").assert().success().stdout(predicate::str::contains("forge"));
}

#[test]
fn info_command_succeeds() {
  forge().arg("info").assert().success().stdout(predicate::str::contains("forge"));
}


// Generate - argument validation

#[test]
fn generate_requires_artifact_and_name() {
  forge().args(["generate"]).assert().failure();
}

#[test]
fn generate_rejects_unknown_artifact() {
  let dir = temp_project(None);
  forge().current_dir(dir.path()).args(["generate", "foobar", "products"]).assert().failure().stderr(predicate::str::contains("Unknown artifact"));
}

#[test]
fn generate_suggests_correction_for_typo() {
  let dir = temp_project(None);
  forge().current_dir(dir.path()).args(["generate", "servce", "products"]).assert().failure().stderr(predicate::str::contains("service"));
}

#[test]
fn generate_rejects_name_with_space() {
  let dir = temp_project(None);
  forge().current_dir(dir.path()).args(["generate", "service", "my service"]).assert().failure().stderr(predicate::str::contains("spaces"));
}

#[test]
fn generate_rejects_name_starting_with_digit() {
  let dir = temp_project(None);
  forge().current_dir(dir.path()).args(["generate", "service", "3products"]).assert().failure().stderr(predicate::str::contains("letter"));
}