extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, Type, TypePath};

#[proc_macro_derive(DynLayout)]
pub fn dyn_layout_macro(input: TokenStream) -> TokenStream {
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
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64",
        // glam types (to avoid orphan rule issues with glam types)
        "IVec2", "IVec3", "IVec4", "UVec2", "UVec3", "UVec4", "Vec2", "Vec3", "Vec4", "Mat2",
        "Mat3", "Mat4", "Quat", "DVec2", "DVec3", "DVec4", "DMat2", "DMat3", "DMat4", "DAffine2",
        "DAffine3",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    let mut field_inits = Vec::new();

    for field in fields {
        let field_name = &field.ident;
        let field_type = &field.ty;

        let align_expr = quote! { std::mem::align_of::<#field_type>() };
        let size_expr = quote! { std::mem::size_of::<#field_type>() };

        let struct_layout = if is_basic_type(field_type, &basic_types) {
            quote! { dyn_pod_struct::base_type::get_base_type::<#field_type>() }
        } else {
            quote! {
                dyn_pod_struct::base_type::BaseType::Struct({
                    let nested_layout = <#field_type as dyn_pod_struct::dyn_layout::HasDynLayout>::dyn_layout();
                    let nested_fields = nested_layout.fields.iter().map(|(name, field)| {
                        let mut field = field.clone();
                        field.offset += offset as u32;  // Adjust for parent offset
                        (name.clone(), field)
                    }).collect();
                    Arc::new(dyn_pod_struct::dyn_layout::DynLayout::new(&nested_layout.name, nested_layout.size, nested_fields))
                })
            }
        };

        field_inits.push(quote! {
            offset = (offset + #align_expr - 1) & !(#align_expr - 1);
            fields.push((
                stringify!(#field_name).into(),
                dyn_pod_struct::dyn_struct::DynField {
                    offset: offset as u32,
                    // size: #size_expr as u32,
                    ty: #struct_layout,
                }
            ));
            offset += #size_expr;
        });
    }

    let expanded = quote! {
        impl dyn_pod_struct::dyn_layout::HasDynLayout for #struct_name {
            fn dyn_layout() -> std::sync::Arc<dyn_pod_struct::dyn_layout::DynLayout> {
                use std::sync::Arc;
                let mut fields = Vec::new();
                let mut offset = 0usize;

                #(#field_inits)*

                Arc::new(dyn_pod_struct::dyn_layout::DynLayout::new(stringify!(#struct_name).into(), offset, fields))
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_basic_type(ty: &Type, basic_types: &HashSet<String>) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(ident) = path.get_ident() {
            let type_name = ident.to_string();
            basic_types.contains(&type_name)
        } else {
            if let Some(last_segment) = path.segments.last() {
                let type_name = last_segment.ident.to_string();
                basic_types.contains(&type_name)
            } else {
                false
            }
        }
    } else {
        false
    }
}
