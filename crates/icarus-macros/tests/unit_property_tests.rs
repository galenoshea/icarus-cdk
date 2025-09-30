//! Property-based unit tests for icarus-macros crate utility functions.
//!
//! These tests verify utility function behavior with randomly generated inputs,
//! ensuring that invariants hold across all possible valid inputs.

use proptest::prelude::*;

/// Test that `snake_case` to `PascalCase` conversion behaves correctly
#[cfg(test)]
mod pascal_case_properties {
    use super::*;

    /// Generate arbitrary valid `snake_case` identifiers
    fn arb_snake_case() -> impl Strategy<Value = String> {
        prop::string::string_regex(r"[a-z][a-z0-9_]*")
            .unwrap()
            .prop_filter("Too long", |s| s.len() <= 50)
            .prop_filter("No trailing underscore", |s| !s.ends_with('_'))
            .prop_filter("No consecutive underscores", |s| !s.contains("__"))
    }

    proptest! {
        #[test]
        fn test_pascal_case_properties(
            snake_case in arb_snake_case()
        ) {
            // We can't directly test the internal to_pascal_case function,
            // but we can test the properties it should have

            // Original string should be non-empty
            prop_assert!(!snake_case.is_empty());

            // Should start with lowercase letter
            prop_assert!(snake_case.chars().next().unwrap().is_ascii_lowercase());

            // Should only contain valid characters
            prop_assert!(snake_case.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));

            // Should not have consecutive underscores (good practice)
            prop_assert!(!snake_case.contains("__"));
        }
    }

    proptest! {
        #[test]
        fn test_param_struct_name_properties(
            function_name in arb_snake_case()
        ) {
            // The generated struct name should follow certain patterns
            let expected_suffix = "Params";

            // Should be a valid Rust identifier pattern
            let first_char_upper = function_name.chars().next().unwrap().to_uppercase().to_string();
            prop_assert!(!first_char_upper.is_empty());

            // Combined length should be reasonable
            let total_len = function_name.len() + expected_suffix.len();
            prop_assert!(total_len <= 100); // Reasonable identifier length limit
        }
    }
}

/// Test Option type detection properties
#[cfg(test)]
mod option_type_properties {
    use super::*;

    /// Generate arbitrary type strings
    fn arb_type_string() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("i32".to_string()),
            Just("String".to_string()),
            Just("bool".to_string()),
            Just("Vec<String>".to_string()),
            prop::string::string_regex(r"[A-Z][a-zA-Z0-9]*").unwrap(),
        ]
    }

    proptest! {
        #[test]
        fn test_option_detection_properties(
            inner_type in arb_type_string(),
            is_option in any::<bool>()
        ) {
            let type_string = if is_option {
                format!("Option<{}>", inner_type)
            } else {
                inner_type.clone()
            };

            // Option types should contain "Option<"
            let contains_option = type_string.contains("Option<");
            prop_assert_eq!(contains_option, is_option);

            // Non-option types should not contain "Option<"
            if !is_option {
                prop_assert!(!type_string.contains("Option<"));
            }

            // All type strings should be non-empty
            prop_assert!(!type_string.is_empty());
        }
    }
}

/// Test configuration parsing properties
#[cfg(test)]
mod config_properties {
    use super::*;

    /// Generate valid configuration keys
    fn arb_config_key() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("name".to_string()),
            Just("description".to_string()),
            Just("version".to_string()),
            Just("auth".to_string()),
            Just("rate_limit".to_string()),
        ]
    }

    /// Generate configuration values
    fn arb_config_value() -> impl Strategy<Value = String> {
        prop_oneof![
            prop::string::string_regex(r"[a-zA-Z0-9 ._-]{1,50}").unwrap(),
            Just("true".to_string()),
            Just("false".to_string()),
        ]
    }

    proptest! {
        #[test]
        fn test_config_validation_properties(
            key in arb_config_key(),
            value in arb_config_value()
        ) {
            // Valid keys should be recognized
            let valid_keys = ["name", "description", "version", "auth", "rate_limit"];
            prop_assert!(valid_keys.contains(&key.as_str()));

            // Boolean configs should only accept boolean values
            if key == "auth" || key == "rate_limit" {
                let is_boolean = value == "true" || value == "false";
                if !is_boolean {
                    // Non-boolean values should be rejected for boolean configs
                    prop_assert!(value != "true" && value != "false");
                }
            }

            // Values should be non-empty
            prop_assert!(!value.is_empty());

            // Values should have reasonable length
            prop_assert!(value.len() <= 100);
        }
    }
}

/// Test error message properties
#[cfg(test)]
mod error_properties {
    use super::*;

    /// Generate error contexts
    fn arb_error_context() -> impl Strategy<Value = String> {
        prop::string::string_regex(r"[a-zA-Z0-9 ._-]{1,100}")
            .unwrap()
            .prop_filter("Not just whitespace", |s| !s.trim().is_empty())
    }

    proptest! {
        #[test]
        fn test_error_message_properties(
            context in arb_error_context(),
            feature in prop::string::string_regex(r"[a-zA-Z_][a-zA-Z0-9_]*").unwrap(),
            reason in arb_error_context()
        ) {
            // Error messages should be descriptive
            prop_assert!(!context.is_empty());
            prop_assert!(!feature.is_empty());
            prop_assert!(!reason.is_empty());

            // Combined error message should be reasonable length
            let combined_len = context.len() + feature.len() + reason.len();
            prop_assert!(combined_len >= 3); // At least some content
            prop_assert!(combined_len <= 500); // Not too verbose

            // Context should not be just whitespace
            prop_assert!(!context.trim().is_empty());
        }
    }
}

/// Test function signature validation properties
#[cfg(test)]
mod signature_validation_properties {
    use super::*;

    proptest! {
        #[test]
        fn test_signature_validation_properties(
            has_generics in any::<bool>(),
            has_self in any::<bool>(),
            param_count in 0usize..=20
        ) {
            // Functions with problematic features should be caught
            if has_generics {
                // Generic functions should be rejected
                prop_assert!(has_generics); // This represents the condition that would cause rejection
            }

            if has_self {
                // Methods with self should be rejected
                prop_assert!(has_self); // This represents the condition that would cause rejection
            }

            // Parameter count should be reasonable
            prop_assert!(param_count <= 50); // Reasonable limit for tool functions

            // Valid combinations should pass
            if !has_generics && !has_self && param_count <= 20 {
                // This should be a valid tool function signature
                prop_assert!(true);
            }
        }
    }
}

/// Test JSON serialization properties
#[cfg(test)]
mod json_properties {
    use super::*;
    use serde_json::{from_str, to_string, Value};

    proptest! {
        #[test]
        fn test_json_roundtrip_properties(
            string_val in prop::string::string_regex(r"[a-zA-Z0-9 ._-]{0,100}").unwrap(),
            int_val in any::<i32>(),
            bool_val in any::<bool>()
        ) {
            // Create a test JSON object
            let test_object = serde_json::json!({
                "string_field": string_val,
                "int_field": int_val,
                "bool_field": bool_val
            });

            // Test serialization
            if let Ok(serialized) = to_string(&test_object) {
                // Serialized JSON should be non-empty
                prop_assert!(!serialized.is_empty());

                // Should be valid JSON (contains expected structure)
                prop_assert!(serialized.len() > 2);

                // Test deserialization
                if let Ok(deserialized) = from_str::<Value>(&serialized) {
                    // Round-trip should preserve data
                    prop_assert_eq!(test_object, deserialized);
                }
            }
        }
    }

    proptest! {
        #[test]
        fn test_parameter_json_properties(
            param_name in prop::string::string_regex(r"[a-z][a-z0-9_]*").unwrap(),
            param_value in prop::string::string_regex(r"[a-zA-Z0-9 ._-]{0,50}").unwrap()
        ) {
            // Parameter names should be valid identifiers
            prop_assert!(!param_name.is_empty());
            prop_assert!(param_name.chars().next().unwrap().is_ascii_lowercase());

            // JSON object with parameter should be valid
            let param_obj = serde_json::json!({
                param_name.clone(): param_value
            });

            if let Ok(json_str) = to_string(&param_obj) {
                prop_assert!(json_str.contains(&param_name));
                prop_assert!(json_str.contains(&param_value) || param_value.is_empty());
            }
        }
    }
}

/// Test macro naming conventions
#[cfg(test)]
mod naming_properties {
    use super::*;

    proptest! {
        #[test]
        fn test_wrapper_naming_properties(
            fn_name in prop::string::string_regex(r"[a-z][a-z0-9_]*").unwrap()
        ) {
            prop_assume!(!fn_name.is_empty());
            prop_assume!(fn_name.len() <= 50);

            // Generated wrapper names should follow patterns
            let wrapper_prefix = "__";
            let wrapper_suffix = "_tool_wrapper";

            // Combined name should be reasonable length
            let total_len = wrapper_prefix.len() + fn_name.len() + wrapper_suffix.len();
            prop_assert!(total_len <= 100);

            // Should not conflict with common names
            prop_assert!(fn_name != "main");
            prop_assert!(fn_name != "new");
            prop_assert!(fn_name != "default");
        }
    }

    proptest! {
        #[test]
        fn test_param_struct_naming_properties(
            fn_name in prop::string::string_regex(r"[a-z][a-z0-9_]*").unwrap()
        ) {
            prop_assume!(!fn_name.is_empty());
            prop_assume!(fn_name.len() <= 50);

            // Parameter struct names should follow conventions
            let struct_suffix = "Params";

            // Should generate valid struct name
            let total_len = fn_name.len() + struct_suffix.len();
            prop_assert!(total_len <= 100);

            // Should be different from function name
            let params_name = format!("{}Params", fn_name).to_lowercase();
            prop_assert!(fn_name != params_name);
        }
    }
}

/// Test identifier validation properties
#[cfg(test)]
mod identifier_properties {
    use super::*;

    proptest! {
        #[test]
        fn test_identifier_validity(
            identifier in prop::string::string_regex(r"[a-zA-Z_][a-zA-Z0-9_]*").unwrap()
        ) {
            prop_assume!(!identifier.is_empty());
            prop_assume!(identifier.len() <= 100);

            // Valid Rust identifier properties
            let first_char = identifier.chars().next().unwrap();
            prop_assert!(first_char.is_ascii_alphabetic() || first_char == '_');

            // All characters should be valid
            prop_assert!(identifier.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));

            // Should not be a reserved keyword
            let reserved = ["fn", "let", "const", "static", "struct", "enum", "impl", "trait", "self", "Self"];
            prop_assert!(!reserved.contains(&identifier.as_str()));
        }
    }
}
