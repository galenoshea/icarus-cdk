// Copyright (c) 2025 Icarus Team. All Rights Reserved.
// Licensed under BSL-1.1. See LICENSE and NOTICE files.

//! Compile-time validation for tool signatures
//!
//! This module provides validation logic to ensure tool functions
//! have compatible signatures for both ICP canisters and the MCP bridge.

use syn::{FnArg, ItemFn, Pat, ReturnType, Type};

/// Validation result with detailed error messages
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Validate a tool function's signature
pub fn validate_tool_function(
    func: &ItemFn,
    has_query: bool,
    has_update: bool,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    // Check that it has either #[query] or #[update], but not both
    if has_query && has_update {
        result.add_error(
            "Tool functions must have either #[query] or #[update], not both".to_string(),
        );
    } else if !has_query && !has_update {
        result.add_error(
            "Tool functions must have either #[query] or #[update] attribute".to_string(),
        );
    }

    // Validate return type
    validate_return_type(&func.sig.output, &mut result);

    // Validate parameters
    validate_parameters(&func.sig.inputs, &mut result);

    // Validate async/sync alignment
    validate_async_sync(&func.sig, has_query, &mut result);

    result
}

/// Validate that the return type is Result<T, String> or Result<T, E> where E: Display
fn validate_return_type(output: &ReturnType, result: &mut ValidationResult) {
    match output {
        ReturnType::Default => {
            result.add_warning(
                "Tool functions should return Result<T, String> for proper error handling"
                    .to_string(),
            );
        }
        ReturnType::Type(_, ty) => {
            if !is_result_type(ty) {
                result.add_error("Tool functions must return Result<T, String>".to_string());
            }
        }
    }
}

/// Check if a type is Result<T, E>
fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Result";
        }
    }
    false
}

/// Validate function parameters
fn validate_parameters(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    result: &mut ValidationResult,
) {
    for arg in inputs {
        match arg {
            FnArg::Receiver(_) => {
                result.add_error("Tool functions cannot have self parameters".to_string());
            }
            FnArg::Typed(pat_type) => {
                // Check for references and lifetimes
                if has_references(&pat_type.ty) {
                    result.add_error(
                        "Tool parameters cannot contain references. Use owned types instead."
                            .to_string(),
                    );
                }

                // Ensure parameter has a simple identifier pattern
                if !is_simple_pattern(&pat_type.pat) {
                    result.add_warning(
                        "Tool parameters should use simple identifiers (e.g., 'name: String')"
                            .to_string(),
                    );
                }
            }
        }
    }
}

/// Check if a type contains references
fn has_references(ty: &Type) -> bool {
    match ty {
        Type::Reference(_) => true,
        Type::Path(_) => {
            // For now, we only check for direct references
            // More complex reference checking would require full type resolution
            false
        }
        _ => false,
    }
}

/// Check if a pattern is a simple identifier
fn is_simple_pattern(pat: &Pat) -> bool {
    matches!(pat, Pat::Ident(_))
}

/// Validate async/sync alignment with query/update
fn validate_async_sync(sig: &syn::Signature, has_query: bool, result: &mut ValidationResult) {
    let is_async = sig.asyncness.is_some();

    if has_query && is_async {
        result.add_error(
            "Query functions cannot be async in ICP canisters. Remove 'async' or change to #[update]".to_string()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_valid_tool_function() {
        let func: ItemFn = parse_quote! {
            async fn my_tool(data: String) -> Result<String, String> {
                Ok(data)
            }
        };

        let result = validate_tool_function(&func, false, true);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_query_cannot_be_async() {
        let func: ItemFn = parse_quote! {
            async fn my_query() -> Result<String, String> {
                Ok("data".to_string())
            }
        };

        let result = validate_tool_function(&func, true, false);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("cannot be async")));
    }

    #[test]
    fn test_must_return_result() {
        let func: ItemFn = parse_quote! {
            fn my_tool() -> String {
                "data".to_string()
            }
        };

        let result = validate_tool_function(&func, false, true);
        assert!(!result.is_valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("must return Result")));
    }

    #[test]
    fn test_no_self_parameters() {
        let func: ItemFn = parse_quote! {
            fn my_tool(&self) -> Result<String, String> {
                Ok("data".to_string())
            }
        };

        let result = validate_tool_function(&func, false, true);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("cannot have self")));
    }
}
