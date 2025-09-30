//! Utility functions for procedural macros.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Attribute, FnArg, Ident, Lit, Pat, PatType, ReturnType, Type};

use crate::error::{MacroError, MacroResult};

/// Parameter-level schema customization attributes.
#[derive(Clone, Debug, Default)]
pub(crate) struct ParamAttributes {
    /// Custom description for the parameter
    pub description: Option<String>,
    /// Minimum value for numeric types
    pub min: Option<i64>,
    /// Maximum value for numeric types
    pub max: Option<i64>,
    /// Minimum length for string types
    pub min_length: Option<usize>,
    /// Maximum length for string types
    pub max_length: Option<usize>,
    /// Regex pattern for string validation
    pub pattern: Option<String>,
}

/// Parses #[param(...)] attributes from a parameter.
fn parse_param_attributes(attrs: &[Attribute]) -> MacroResult<ParamAttributes> {
    let mut result = ParamAttributes::default();

    for attr in attrs {
        // Only process #[param(...)] attributes
        if !attr.path().is_ident("param") {
            continue;
        }

        // Parse the meta list inside #[param(...)]
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("description") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Str(lit_str) = value {
                    result.description = Some(lit_str.value());
                }
            } else if meta.path.is_ident("min") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Int(lit_int) = value {
                    result.min = Some(lit_int.base10_parse()?);
                }
            } else if meta.path.is_ident("max") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Int(lit_int) = value {
                    result.max = Some(lit_int.base10_parse()?);
                }
            } else if meta.path.is_ident("min_length") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Int(lit_int) = value {
                    result.min_length = Some(lit_int.base10_parse()?);
                }
            } else if meta.path.is_ident("max_length") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Int(lit_int) = value {
                    result.max_length = Some(lit_int.base10_parse()?);
                }
            } else if meta.path.is_ident("pattern") {
                let value: Lit = meta.value()?.parse()?;
                if let Lit::Str(lit_str) = value {
                    result.pattern = Some(lit_str.value());
                }
            }

            Ok(())
        })?;
    }

    Ok(result)
}

/// Extracts parameter information from function arguments.
pub(crate) fn extract_parameters(
    inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>,
) -> MacroResult<Vec<ParameterInfo>> {
    let mut parameters = Vec::new();

    for input in inputs {
        match input {
            FnArg::Receiver(receiver) => {
                return Err(MacroError::invalid_signature_spanned(
                    "Tool functions cannot have self parameters",
                    receiver.span(),
                ));
            }
            FnArg::Typed(PatType { pat, ty, attrs, .. }) => {
                let param_name = extract_param_name(pat)?;
                let param_type = ty.as_ref().clone();
                let is_optional = is_option_type(&param_type);
                let attributes = parse_param_attributes(attrs)?;

                parameters.push(ParameterInfo {
                    name: param_name,
                    ty: param_type,
                    is_optional,
                    attributes,
                });
            }
        }
    }

    Ok(parameters)
}

/// Information about a function parameter.
#[derive(Clone)]
pub(crate) struct ParameterInfo {
    /// Parameter name
    pub name: Ident,
    /// Parameter type
    pub ty: Type,
    /// Whether the parameter is optional (Option<T>)
    pub is_optional: bool,
    /// Schema customization attributes
    pub attributes: ParamAttributes,
}

impl std::fmt::Debug for ParameterInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParameterInfo")
            .field("name", &self.name)
            .field("ty", &"Type") // Don't debug print the full Type
            .field("is_optional", &self.is_optional)
            .field("attributes", &self.attributes)
            .finish()
    }
}

/// Extracts the parameter name from a pattern.
fn extract_param_name(pat: &Pat) -> MacroResult<Ident> {
    match pat {
        Pat::Ident(pat_ident) => Ok(pat_ident.ident.clone()),
        _ => Err(MacroError::invalid_signature_spanned(
            "Only simple parameter names are supported",
            pat.span(),
        )),
    }
}

/// Checks if a type is Option<T>.
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Extracts the return type from a function signature.
pub(crate) fn extract_return_type(output: &ReturnType) -> Type {
    match output {
        ReturnType::Default => syn::parse_quote!(()),
        ReturnType::Type(_, ty) => ty.as_ref().clone(),
    }
}

/// Generates a parameter structure name from a function name.
pub(crate) fn generate_param_struct_name(fn_name: &Ident) -> Ident {
    format_ident!("{}Params", to_pascal_case(&fn_name.to_string()))
}

/// Converts `snake_case` to `PascalCase`.
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

/// Generates JSON schema for a parameter type.
#[allow(dead_code)]
pub(crate) fn generate_param_schema(param: &ParameterInfo) -> TokenStream {
    let name = &param.name;
    let name_str = name.to_string();

    // Generate appropriate schema based on type
    quote! {
        {
            "type": "string",
            "description": concat!("Parameter ", #name_str)
        }
    }
}

/// Generates a complete JSON Schema `input_schema` for tool parameters.
///
/// Returns `TokenStream` that creates `Arc<serde_json::Map<String, serde_json::Value>>`.
pub(crate) fn generate_json_schema_from_parameters(params: &[ParameterInfo]) -> TokenStream {
    if params.is_empty() {
        // Empty schema for functions with no parameters
        return quote! {
            ::std::sync::Arc::new(::serde_json::Map::new())
        };
    }

    // Generate property schemas for each parameter
    let property_entries: Vec<TokenStream> = params
        .iter()
        .map(|param| {
            let param_name = param.name.to_string();
            let json_type = get_json_type_for_rust_type(&param.ty);

            // Use custom description or default
            let param_description = param
                .attributes
                .description
                .clone()
                .unwrap_or_else(|| format!("Parameter: {param_name}"));

            // Build schema object with conditional fields
            let mut schema_fields = vec![
                quote! { "type": #json_type },
                quote! { "description": #param_description },
            ];

            // Add numeric constraints if present
            if let Some(min) = param.attributes.min {
                schema_fields.push(quote! { "minimum": #min });
            }
            if let Some(max) = param.attributes.max {
                schema_fields.push(quote! { "maximum": #max });
            }

            // Add string constraints if present
            if let Some(min_length) = param.attributes.min_length {
                schema_fields.push(quote! { "minLength": #min_length });
            }
            if let Some(max_length) = param.attributes.max_length {
                schema_fields.push(quote! { "maxLength": #max_length });
            }
            if let Some(pattern) = &param.attributes.pattern {
                schema_fields.push(quote! { "pattern": #pattern });
            }

            quote! {
                properties.insert(
                    #param_name.to_string(),
                    ::serde_json::json!({
                        #(#schema_fields),*
                    })
                );
            }
        })
        .collect();

    // Generate required field list (non-optional parameters)
    let required_params: Vec<TokenStream> = params
        .iter()
        .filter(|param| !param.is_optional)
        .map(|param| {
            let param_name = param.name.to_string();
            quote! { #param_name }
        })
        .collect();

    let required_array = if required_params.is_empty() {
        quote! { ::serde_json::json!([]) }
    } else {
        quote! { ::serde_json::json!([#(#required_params),*]) }
    };

    quote! {
        {
            let mut schema = ::serde_json::Map::new();
            let mut properties = ::serde_json::Map::new();

            #(#property_entries)*

            schema.insert("type".to_string(), ::serde_json::json!("object"));
            schema.insert("properties".to_string(), ::serde_json::json!(properties));
            schema.insert("required".to_string(), #required_array);

            ::std::sync::Arc::new(schema)
        }
    }
}

/// Maps Rust types to JSON Schema types.
fn get_json_type_for_rust_type(ty: &Type) -> &'static str {
    // Extract the base type name from the Type
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();

            // Handle Option<T> - unwrap to get inner type
            if type_name == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return get_json_type_for_rust_type(inner_ty);
                    }
                }
            }

            // Map common Rust types to JSON Schema types
            #[allow(clippy::match_same_arms)]
            return match type_name.as_str() {
                "String" | "str" | "&str" => "string",
                "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128"
                | "isize" | "usize" => "integer",
                "f32" | "f64" => "number",
                "bool" => "boolean",
                "Vec" => "array",
                _ => "string", // Default fallback
            };
        }
    }

    // Default fallback
    "string"
}

/// Generates validation code for parameters.
#[allow(dead_code)]
pub(crate) fn generate_param_validation(params: &[ParameterInfo]) -> TokenStream {
    if params.is_empty() {
        return quote! {};
    }

    let validations: Vec<TokenStream> = params
        .iter()
        .filter(|param| !param.is_optional)
        .map(|param| {
            let name = &param.name;
            let name_str = name.to_string();

            quote! {
                if args.#name.is_none() {
                    return Err(format!("Missing required parameter: {}", #name_str));
                }
            }
        })
        .collect();

    if validations.is_empty() {
        quote! {}
    } else {
        quote! {
            #(#validations)*
        }
    }
}

/// Generates tool registration code.
#[allow(dead_code)]
pub(crate) fn generate_tool_registration(
    fn_name: &Ident,
    description: Option<&str>,
) -> TokenStream {
    let fn_name_str = fn_name.to_string();
    let default_description = format!("Tool: {fn_name_str}");
    let description = description.unwrap_or(&default_description);

    quote! {
        ::icarus_core::Tool::builder()
            .name(::icarus_core::ToolId::new(#fn_name_str).expect("Valid tool ID"))
            .description(#description)
            .build()
            .expect("Valid tool definition")
    }
}

/// Checks if a function is async.
pub(crate) fn is_async_function(sig: &syn::Signature) -> bool {
    sig.asyncness.is_some()
}

/// Generates appropriate call expression for sync/async functions.
pub(crate) fn generate_function_call(
    fn_name: &Ident,
    params: &[ParameterInfo],
    is_async: bool,
) -> TokenStream {
    let param_names: Vec<&Ident> = params.iter().map(|p| &p.name).collect();

    if is_async {
        quote! {
            #fn_name(#(args.#param_names),*).await
        }
    } else {
        quote! {
            #fn_name(#(args.#param_names),*)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, Type};

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("simple"), "Simple");
        assert_eq!(to_pascal_case("a_b_c"), "ABC");
    }

    #[test]
    fn test_is_option_type() {
        let option_type: Type = parse_quote!(Option<String>);
        let string_type: Type = parse_quote!(String);

        assert!(is_option_type(&option_type));
        assert!(!is_option_type(&string_type));
    }

    #[test]
    fn test_generate_param_struct_name() {
        let fn_name = format_ident!("my_function");
        let struct_name = generate_param_struct_name(&fn_name);
        assert_eq!(struct_name.to_string(), "MyFunctionParams");
    }

    #[test]
    fn test_parse_param_attributes() {
        // Test parsing a parameter with attributes
        let attrs: Vec<Attribute> =
            vec![parse_quote!(#[param(description = "Age value", min = 1, max = 120)])];

        let result = parse_param_attributes(&attrs).expect("Should parse attributes");

        assert_eq!(result.description, Some("Age value".to_string()));
        assert_eq!(result.min, Some(1));
        assert_eq!(result.max, Some(120));
        assert_eq!(result.min_length, None);
        assert_eq!(result.max_length, None);
        assert_eq!(result.pattern, None);
    }

    #[test]
    fn test_parse_param_attributes_string_constraints() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[param(description = "Username", min_length = 3, max_length = 20, pattern = "^[a-z]+$")]),
        ];

        let result = parse_param_attributes(&attrs).expect("Should parse attributes");

        assert_eq!(result.description, Some("Username".to_string()));
        assert_eq!(result.min_length, Some(3));
        assert_eq!(result.max_length, Some(20));
        assert_eq!(result.pattern, Some("^[a-z]+$".to_string()));
        assert_eq!(result.min, None);
        assert_eq!(result.max, None);
    }

    #[test]
    fn test_parse_param_attributes_empty() {
        let attrs: Vec<Attribute> = vec![];

        let result = parse_param_attributes(&attrs).expect("Should parse empty attributes");

        assert_eq!(result.description, None);
        assert_eq!(result.min, None);
        assert_eq!(result.max, None);
        assert_eq!(result.min_length, None);
        assert_eq!(result.max_length, None);
        assert_eq!(result.pattern, None);
    }
}
