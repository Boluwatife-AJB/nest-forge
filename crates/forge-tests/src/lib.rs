//! Shared test utilities for forge integration tests.

use std::path::Path;

/// Assert that a generated TypeScript file is well-formed at a basic level
pub fn assert_valid_typescript(path: &Path) {
  assert!(path.exists(), "file does not exist: {}", path.display());

  let contents = std::fs::read_to_string(path).expect("failed to read file");

//   Must not be empty
assert!(!contents.is_empty(), "file is empty: {}", path.display());

// Basic bracket balance check
let open_brackets = contents.chars().filter(|&c| c == '{').count();
let close_brackets = contents.chars().filter(|&c| c == '}').count();
assert_eq!(
        open_brackets,
        close_brackets,
        "unbalanced braces in {}: {} open, {} close",
        path.display(),
        open_brackets,
        close_brackets
    );

    // Must end with a newline (POSIX standard, also what formatters expect)
    assert!(
        contents.ends_with('\n'),
        "file does not end with newline: {}",
        path.display()
    );
}

/// Assert that a file contains all the given strings
pub fn assert_file_contains(path: &Path, expected: &[&str]) {
  let contents = std::fs::read_to_string(path).expect("failed to read file");

  for s in expected {
    assert!(contents.contains(s), "file {} should contain '{}' but doesn't.\nFull contents:\n{}", path.display(), s, contents);
  }
}