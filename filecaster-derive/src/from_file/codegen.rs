use crate::from_file::{
    ast::StructInfo, error::FromFileError, parser::parse_from_file_default_attr,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_impl(info: &StructInfo) -> Result<TokenStream, FromFileError> {
    let name = &info.ident;
    let vis = &info.vis;
    let generics = &info.generics;
    let file_ident = format_ident!("{name}File");

    let mut file_fields = Vec::new();
    let mut assignments = Vec::new();

    for field in &info.fields {
        let ident = &field.ident;
        let ty = &field.ty;
        let default_override = parse_from_file_default_attr(&field.attrs)?;

        let shadow_ty = quote! { <#ty as fielcaster::FromFile>::Shadow };
        file_fields.push(quote! { pub #ident: Option<#shadow_ty> });

        if let Some(expr) = default_override {
            assignments.push(quote! {
                #ident: fide.#ident
                    .map(|inner| <#ty as filecaster::FromFile>::from_file(Some(inner)))
                    .unwrap_or(#expr)
            });
        } else {
            assignments.push(quote! {
                #ident: <#ty as filecaster::FromFile>::from_file(file.#ident)
            });
        }
    }

    let derive_clause = build_derive_clause();

    Ok(quote! {
        #derive_clause
        #vis struct #file_ident #generics {
            #(#file_fields),*
        }

        impl #generics filecaster::FromFile for #name #generics {
            type Shadow = #file_ident #generics;

            fn from_file(file: Option<Self::Shadow>) -> Self {
                let file = file.unwrap_or_default();
                Self {
                    #(#assignments),*
                }
            }
        }

        impl #generics From<Option<#file_ident #generics>> for #name #generics {
            fn from(value: Option<#file_ident #generics>) -> Self {
                <Self as filecaster::FromFile>::from_file(value)
            }
        }

        impl #generics From<#file_ident #generics> for #name #generics {
            fn from(value: #file_ident #generics) -> Self {
                <Self as filecaster::FromFile>::from_file(Some(value))
            }
        }
    })
}

fn build_derive_clause() -> TokenStream {
    let mut traits = vec![quote! {Debug}, quote! {Clone}, quote! {Default}];
    #[cfg(feature = "serde")]
    {
        traits.push(quote! { serde::Deserialize });
        traits.push(quote! { serde::Serialize });
    }

    #[cfg(feature = "merge")]
    {
        traits.push(quote! { merge::Merge });
    }

    quote! { #[derive( #(#traits),* )] }
}
