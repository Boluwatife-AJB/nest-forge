//! Property-based tests for forge-core.
//!
//! These tests define invariants that must hold for ALL valid inputs,
//! then let proptest generate hundreds of random cases to check them.

use forge_core::artifact::ArtifactName;
use proptest::prelude::*;


// Name validation Properties
/// A regex strategy that generates valid artifact names.
fn valid_name_strategy() -> impl Strategy<Value = String> {
  // Start with a letter, followed by 0-30 valid characters
  "[a-zA-Z][a-zA-Z0-9_-]{0,30}".prop_map(|s| s)
}

proptest! {
  /// For any valid name, ArtifactName::parse should succeed
  #[test]
  fn valid_name_always_parse(name in valid_name_strategy()) {
    prop_assert!(
      ArtifactName::parse(&name).is_ok(), "valid name '{}' should parse but failed", name
    )
  }

  /// For any valid name, pascal case mut start with an uppercase letter
  #[test]
  fn pascal_case_always_starts_uppercase(name in valid_name_strategy()) {
    let parsed = ArtifactName::parse(&name).expect("valid name should parse");
    let first_char = parsed.pascal.chars().next().expect("pascal case should have at least one character");
    prop_assert!(first_char.is_uppercase(), "pascal case '{}' was '{}' should start with an uppercase letter", name, parsed.pascal);
  }

  /// For any valid name, kebab case must not contain underscores
  #[test]
  fn kebab_case_never_contains_underscores(name in valid_name_strategy()) {
    let parsed = ArtifactName::parse(&name).expect("valid name should parse");
    prop_assert!(!parsed.kebab.contains("_"), "kebab case '{}' was '{}' should not contain underscores", name, parsed.kebab);
  }

  /// For any valid name, snake case must not contain hyphens
  #[test]
  fn snake_case_never_contains_hyphens(name in valid_name_strategy()) {
    let parsed = ArtifactName::parse(&name).expect("valid name should parse");
    prop_assert!(!parsed.snake.contains("-"), "snake case '{}' was '{}' should not contain hyphens", name, parsed.snake);
  }

  /// camelCase must never start with an uppercase letter
  #[test]
  fn camel_case_never_starts_uppercase(name in valid_name_strategy()) {
    let parsed = ArtifactName::parse(&name).expect("valid name should parse");
    let first_char = parsed.camel.chars().next().expect("camel case should have at least one character");
    prop_assert!(first_char.is_lowercase(), "camelCase of '{}' was '{}' should start with a lowercase letter", name, parsed.camel);
  }

  /// raw fields is always preserved exactly as given
  #[test]
  fn raw_is_preserved_exactly(name in valid_name_strategy()) {
    let parsed = ArtifactName::parse(&name).expect("valid name should parse");
    prop_assert_eq!(&parsed.raw, &name);
  }
}

// Rejection Properties
proptest! {
  /// Names with spaces must always be rejected
  #[test]
  fn names_with_spaces_always_rejected(a in "[a-zA-Z]+", b in "[a-zA-Z]+") {
    let name_with_space = format!("{} {}", a, b);
    prop_assert!(
      ArtifactName::parse(&name_with_space).is_err(), "name with spaces '{}' should be rejected", name_with_space);
    }

  /// Names starting with a digit must always be rejected
  #[test]
  fn names_starting_with_digit_always_rejected(digit in 0u8..9u8, rest in "[a-zA-Z0-9_-]{1,30}") {
    let name = format!("{digit}{rest}");
    prop_assert!(
      ArtifactName::parse(&name).is_err(), "name starting with digit '{}' should be rejected", name);
    }    
    
}

// ArtifactKind resolution properties
proptest! {
  /// template_name() must always return a non-empty string
  /// (Guards against accidental empty string return in the match)
  #[test]
  fn template_name_never_empty(kind_str in prop::sample::select(vec![
    "module", "service", "controller", "guard", "interceptor", "middleware", "pipe", "filter", "resolver", "entity", "dto", "decorator", "strategy", "interface", "class", "config"
  ])) {
    use forge_core::artifact::ArtifactKind;
    let kind = ArtifactKind::from_str(kind_str).expect("known artifact string should resolve to a kind");
    prop_assert!(!kind.template_name().is_empty(), "template name for '{}' should be non-empty", kind_str);
  }
}