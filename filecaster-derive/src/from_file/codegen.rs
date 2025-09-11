use crate::from_file::{ast::Struct, parser::parse_from_file_default_attr};
use quote::{format_ident, quote};
use unsynn::*;

pub fn generate_impl(info: &Struct) -> Result<TokenStream> {
    let name = &info.name;
    let vis = &info.vis;
    let generics = &info.generics;
    let file_ident = format_ident!("{name}File");

    let mut file_fields = Vec::new();
    let mut assignments = Vec::new();

    for field in &info.fields {
        let name = &field.name;
        let ty = &field.ty;
        let vis = &field.vis;
        let default_override = parse_from_file_default_attr(&field.attrs)?;

        let shadow_ty = quote! { <#ty as filecaster::FromFile>::Shadow };
        file_fields.push(quote! { #vis #name: Option<#shadow_ty> });

        if let Some(expr) = default_override {
            assignments.push(quote! {
                #name: file.#name
                    .map(|inner| <#ty as filecaster::FromFile>::from_file(Some(inner)))
                    .unwrap_or(#expr.into())
            });
        } else {
            assignments.push(quote! {
                #name: <#ty as filecaster::FromFile>::from_file(file.#name)
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
    let mut traits = vec![quote! { Debug }, quote! { Clone }, quote! { Default }];
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::from_file::grammar::StructDef;

    const SAMPLE: &str = r#"
        pub struct Foo {
            #[attr("value")]
            pub bar: String,
            #[attr("number")]
            pub baz: i32
        }
"#;

    #[test]
    fn implementation() {
        let sdef = SAMPLE
            .to_token_iter()
            .parse::<StructDef>()
            .expect("failed to parse StructDef");

        let foo = generate_impl(&sdef.into()).expect("failed to generate implementation");

        dbg!(foo.tokens_to_string());
    }
}
