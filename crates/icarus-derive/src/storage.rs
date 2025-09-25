//! Storage-related procedural macros

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// Expand the IcarusStorable derive macro
pub fn expand_icarus_storable(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;

    // Extract generics if any
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse attributes
    let mut unbounded = false;
    let mut max_size_bytes = 1024 * 1024; // 1MB default

    for attr in &input.attrs {
        if attr.path().is_ident("icarus_storable") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("unbounded") {
                    unbounded = true;
                    Ok(())
                } else if meta.path.is_ident("max_size") {
                    let value = meta.value()?;
                    let lit_str: syn::LitStr = value.parse()?;
                    let size_str = lit_str.value();
                    max_size_bytes = crate::parse_size_string(&size_str);
                    Ok(())
                } else {
                    Err(meta.error("unsupported icarus_storable attribute"))
                }
            })
            .unwrap_or_else(|e| panic!("Failed to parse icarus_storable attribute: {}", e));
        }
    }

    let bound = if unbounded {
        quote! { ic_stable_structures::storable::Bound::Unbounded }
    } else {
        quote! {
            ic_stable_structures::storable::Bound::Bounded {
                max_size: #max_size_bytes,
                is_fixed_size: false,
            }
        }
    };

    // Generate implementation
    let expanded = quote! {
        impl #impl_generics ic_stable_structures::Storable for #struct_name #ty_generics #where_clause {
            fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
                std::borrow::Cow::Owned(
                    candid::encode_one(self).expect("Failed to encode to Candid")
                )
            }

            fn into_bytes(self) -> std::vec::Vec<u8> {
                candid::encode_one(&self).expect("Failed to encode to Candid")
            }

            fn from_bytes(bytes: std::borrow::Cow<'_, [u8]>) -> Self {
                candid::decode_one(&bytes).expect("Failed to decode from Candid")
            }

            const BOUND: ic_stable_structures::storable::Bound = #bound;
        }
    };

    Ok(expanded)
}

/// Expand the IcarusStorage derive macro
pub fn expand_icarus_storage(input: &DeriveInput) -> syn::Result<TokenStream> {
    if let syn::Data::Struct(data_struct) = &input.data {
        if let syn::Fields::Named(fields_named) = &data_struct.fields {
            let struct_name = &input.ident;
            let mut storage_declarations = vec![];
            let mut accessor_methods = vec![];
            let mut memory_id = 0u8;

            for field in &fields_named.named {
                if let Some(field_name) = &field.ident {
                    let field_type = &field.ty;
                    let field_name_upper =
                        syn::Ident::new(&field_name.to_string().to_uppercase(), field_name.span());

                    // Generate storage declaration based on field type
                    let storage_decl = if is_stable_map_type(field_type) {
                        quote! {
                            #field_name_upper: #field_type =
                                ::ic_stable_structures::StableBTreeMap::init(
                                    MEMORY_MANAGER.with(|m| m.borrow().get(
                                        ::ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                                    ))
                                );
                        }
                    } else if is_stable_cell_type(field_type) {
                        quote! {
                            #field_name_upper: ::ic_stable_structures::StableCell<#field_type, ::ic_stable_structures::memory_manager::VirtualMemory<::ic_stable_structures::DefaultMemoryImpl>> =
                                ::ic_stable_structures::StableCell::init(
                                    MEMORY_MANAGER.with(|m| m.borrow().get(
                                        ::ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                                    )),
                                    Default::default()
                                ).expect("Failed to initialize StableCell");
                        }
                    } else {
                        // For simple types, wrap in StableCell
                        quote! {
                            #field_name_upper: ::ic_stable_structures::StableCell<#field_type, ::ic_stable_structures::memory_manager::VirtualMemory<::ic_stable_structures::DefaultMemoryImpl>> =
                                ::ic_stable_structures::StableCell::init(
                                    MEMORY_MANAGER.with(|m| m.borrow().get(
                                        ::ic_stable_structures::memory_manager::MemoryId::new(#memory_id)
                                    )),
                                    Default::default()
                                ).expect("Failed to initialize StableCell");
                        }
                    };

                    storage_declarations.push(storage_decl);

                    // Generate accessor method
                    let accessor = if is_stable_map_type(field_type) {
                        quote! {
                            pub fn #field_name() -> impl std::ops::Deref<Target = #field_type> {
                                #field_name_upper.with(|storage| storage.borrow())
                            }
                        }
                    } else {
                        let setter_name =
                            syn::Ident::new(&format!("{}_set", field_name), field_name.span());

                        quote! {
                            pub fn #field_name() -> #field_type
                            where
                                #field_type: Clone + Default
                            {
                                #field_name_upper.with(|cell| cell.borrow().get().clone())
                            }

                            pub fn #setter_name(value: #field_type)
                            where
                                #field_type: Clone
                            {
                                #field_name_upper.with(|cell| {
                                    cell.borrow_mut().set(value)
                                        .expect("Failed to set value in StableCell");
                                });
                            }
                        }
                    };

                    accessor_methods.push(accessor);
                    memory_id += 1;
                }
            }

            let expanded = quote! {
                thread_local! {
                    static MEMORY_MANAGER: ::std::cell::RefCell<
                        ::ic_stable_structures::memory_manager::MemoryManager<
                            ::ic_stable_structures::DefaultMemoryImpl
                        >
                    > = ::std::cell::RefCell::new(
                        ::ic_stable_structures::memory_manager::MemoryManager::init(
                            ::ic_stable_structures::DefaultMemoryImpl::default()
                        )
                    );

                    #(static #storage_declarations)*
                }

                impl #struct_name {
                    #(#accessor_methods)*
                }
            };

            Ok(expanded)
        } else {
            Err(syn::Error::new_spanned(
                input,
                "IcarusStorage can only be used on structs with named fields",
            ))
        }
    } else {
        Err(syn::Error::new_spanned(
            input,
            "IcarusStorage can only be used on structs",
        ))
    }
}

/// Expand the IcarusType derive macro
pub fn expand_icarus_type(input: &DeriveInput) -> syn::Result<TokenStream> {
    let struct_name = &input.ident;

    // Extract generics if any
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse attributes for storage configuration
    let mut unbounded = true; // Default to unbounded for convenience
    let mut max_size_bytes = 1024 * 1024; // 1MB default

    for attr in &input.attrs {
        if attr.path().is_ident("icarus_storable") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("unbounded") {
                    unbounded = true;
                    Ok(())
                } else if meta.path.is_ident("bounded") {
                    unbounded = false;
                    Ok(())
                } else if meta.path.is_ident("max_size") {
                    let value = meta.value()?;
                    let lit_str: syn::LitStr = value.parse()?;
                    let size_str = lit_str.value();
                    max_size_bytes = crate::parse_size_string(&size_str);
                    unbounded = false;
                    Ok(())
                } else {
                    Ok(()) // Ignore other attributes
                }
            })
            .unwrap_or(()); // Ignore parse errors
        }
    }

    let bound = if unbounded {
        quote! { ic_stable_structures::storable::Bound::Unbounded }
    } else {
        quote! {
            ic_stable_structures::storable::Bound::Bounded {
                max_size: #max_size_bytes,
                is_fixed_size: false,
            }
        }
    };

    // Generate all the common trait implementations
    let expanded = quote! {
        // Note: We expect the user to add #[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
        // This macro just adds the IcarusStorable functionality

        // Implement Storable for ICP
        impl #impl_generics ic_stable_structures::Storable for #struct_name #ty_generics #where_clause {
            fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
                std::borrow::Cow::Owned(
                    candid::encode_one(self).expect("Failed to encode to Candid")
                )
            }

            fn from_bytes(bytes: std::borrow::Cow<'_, [u8]>) -> Self {
                candid::decode_one(&bytes).expect("Failed to decode from Candid")
            }

            const BOUND: ic_stable_structures::storable::Bound = #bound;
        }
    };

    Ok(expanded)
}

// Helper function to check if a type is StableBTreeMap
fn is_stable_map_type(ty: &syn::Type) -> bool {
    let type_string = quote!(#ty).to_string();
    type_string.contains("StableBTreeMap")
}

// Helper function to check if a type is StableCell
fn is_stable_cell_type(ty: &syn::Type) -> bool {
    let type_string = quote!(#ty).to_string();
    type_string.contains("StableCell")
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_is_stable_map_type() {
        let ty: syn::Type = parse_quote!(StableBTreeMap<String, u64>);
        assert!(is_stable_map_type(&ty));

        let ty: syn::Type = parse_quote!(ic_stable_structures::StableBTreeMap<String, u64>);
        assert!(is_stable_map_type(&ty));

        let ty: syn::Type = parse_quote!(String);
        assert!(!is_stable_map_type(&ty));

        let ty: syn::Type = parse_quote!(Vec<String>);
        assert!(!is_stable_map_type(&ty));
    }

    #[test]
    fn test_is_stable_cell_type() {
        let ty: syn::Type = parse_quote!(StableCell<u64>);
        assert!(is_stable_cell_type(&ty));

        let ty: syn::Type = parse_quote!(ic_stable_structures::StableCell<String>);
        assert!(is_stable_cell_type(&ty));

        let ty: syn::Type = parse_quote!(String);
        assert!(!is_stable_cell_type(&ty));

        let ty: syn::Type = parse_quote!(Vec<u64>);
        assert!(!is_stable_cell_type(&ty));
    }
}
