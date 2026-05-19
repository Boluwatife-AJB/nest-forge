#[cfg(test)]
mod tests {
    // use super::*;
    use crate::artifact::{ArtifactKind, ArtifactName};

    // ArtifactName Validation
    #[test]
    fn valid_simple_name() {
        assert!(ArtifactName::parse("products").is_ok())
    }

    #[test]
    fn valid_kebab_name() {
        assert!(ArtifactName::parse("product-inventory").is_ok())
    }

    #[test]
    fn valid_underscore_name() {
        assert!(ArtifactName::parse("product_inventory").is_ok())
    }

    #[test]
    fn rejects_empty_name() {
        let err = ArtifactName::parse("").unwrap_err();
        assert!(
            err.to_string().contains("empty"),
            "error should mention 'empty', got: {err}"
        )
    }

    #[test]
    fn rejects_name_with_space() {
        let err = ArtifactName::parse("product inventory").unwrap_err();
        assert!(
            err.to_string().contains("spaces"),
            "error should mention spaces, got: {err}"
        )
    }

    #[test]
    fn rejects_name_starting_with_digit() {
        assert!(ArtifactName::parse("1product").is_err())
    }

    #[test]
    fn rejects_name_with_special_chars() {
        assert!(ArtifactName::parse("product!inventory").is_err());
        assert!(ArtifactName::parse("product@variant").is_err());
    }

    #[test]
    fn pascal_case_is_correct() {
        let name = ArtifactName::parse("product-inventory").unwrap();
        assert_eq!(name.pascal, "ProductInventory");
    }

    #[test]
    fn camel_case_is_correct() {
        let name = ArtifactName::parse("product-inventory").unwrap();
        assert_eq!(name.camel, "productInventory");
    }

    #[test]
    fn kebab_case_is_correct() {
        let name = ArtifactName::parse("ProductInventory").unwrap();
        assert_eq!(name.kebab, "product-inventory");
    }

    // ArtifactKind resolution
    #[test]
    fn resolves_full_names() {
        assert_eq!("service".parse::<ArtifactKind>(), Ok(ArtifactKind::Service));
        assert_eq!(
            "controller".parse::<ArtifactKind>(),
            Ok(ArtifactKind::Controller)
        );
    }

    #[test]
    fn resolves_aliases() {
        assert_eq!("s".parse::<ArtifactKind>(), Ok(ArtifactKind::Service));
        assert_eq!("co".parse::<ArtifactKind>(), Ok(ArtifactKind::Controller));
        assert_eq!("mo".parse::<ArtifactKind>(), Ok(ArtifactKind::Module));
    }

    #[test]
    fn resolution_is_case_insensitive() {
        assert_eq!("SERVICE".parse::<ArtifactKind>(), Ok(ArtifactKind::Service));
        assert_eq!("Module".parse::<ArtifactKind>(), Ok(ArtifactKind::Module));
    }

    #[test]
    fn unknown_artifact_returns_none() {
        assert!("foobar".parse::<ArtifactKind>().is_err());
    }

    // Suggestion engine

    #[test]
    fn suggests_closest_match() {
        use crate::suggest::closest_match;
        let candidates = ArtifactKind::all_names();

        assert_eq!(closest_match("servce", candidates), Some("service"));
        assert_eq!(closest_match("moduel", candidates), Some("module"));
        assert_eq!(closest_match("contorller", candidates), Some("controller"));
    }

    #[test]
    fn no_suggestion_for_garbage_input() {
        use crate::suggest::closest_match;
        let candidates = ArtifactKind::all_names();
        assert_eq!(closest_match("xyzqrs", candidates), None);
    }
}
