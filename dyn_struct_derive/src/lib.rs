extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use std::collections::HashSet;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Type, TypePath};

#[proc_macro_derive(DynLayout)]
pub fn dyn_struct_layout_macro(input: TokenStream) -> TokenStream {
    let dyn_struct_core = match crate_name("dyn_struct_core") {
        Ok(FoundCrate::Itself) => quote!(crate),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(#ident)
        }
        Err(_) => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "Could not find `dyn_struct_core` in `Cargo.toml`",
            )
            .to_compile_error()
            .into();
        }
    };

    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.clone().ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<_>>(),
            _ => {
                return syn::Error::new(
                    data_struct.struct_token.span(),
                    "DynLayout only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new(input.span(), "DynLayout can only be used with structs")
                .to_compile_error()
                .into();
        }
    };

    let basic_types: HashSet<String> = [
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let glam_types: HashSet<String> = [
        "Vec2", "Vec3", "Vec4", "Mat2", "Mat3", "Mat4", "Quat", "Affine2", "Affine3A", "DVec2",
        "DVec3", "DVec4", "DMat2", "DMat3", "DMat4", "DAffine2", "DAffine3",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let mut field_inits = Vec::new();
    let mut offset_calc = Vec::new();

    for field in fields {
        let field_name = &field.ident;
        let field_type = &field.ty;

        let align_expr = quote! { std::mem::align_of::<#field_type>() };
        let size_expr = quote! { std::mem::size_of::<#field_type>() };

        offset_calc.push(quote! {
            offset = (offset + #align_expr - 1) & !(#align_expr - 1);
        });

        let struct_layout = if is_basic_type(field_type, &basic_types, &glam_types) {
            quote! { None }
        } else {
            quote! { Some(<#field_type as #dyn_struct_core::HasDynStructLayout>::dyn_struct_layout()) }
        };

        field_inits.push(quote! {
            fields.insert(stringify!(#field_name).into(), #dyn_struct_core::DynField {
                offset: offset as u32,
                size: #size_expr as u32,
                struct_: #struct_layout,
            });
            offset += #size_expr;
        });
    }

    let expanded = quote! {
        impl #dyn_struct_core::HasDynStructLayout for #struct_name {
            fn dyn_struct_layout() -> std::sync::Arc<#dyn_struct_core::DynStructLayout> {
                use std::sync::Arc;
                let mut fields = indexmap::IndexMap::new();
                let mut offset = 0usize;

                #(#offset_calc)*
                #(#field_inits)*

                Arc::new(#dyn_struct_core::DynStructLayout {
                    fields,
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_basic_type(ty: &Type, basic_types: &HashSet<String>, glam_types: &HashSet<String>) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(ident) = path.get_ident() {
            let type_name = ident.to_string();
            basic_types.contains(&type_name) || glam_types.contains(&type_name)
        } else {
            if let Some(last_segment) = path.segments.last() {
                let type_name = last_segment.ident.to_string();
                basic_types.contains(&type_name) || glam_types.contains(&type_name)
            } else {
                false
            }
        }
    } else {
        false
    }
}
