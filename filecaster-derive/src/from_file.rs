use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Field, Fields, FieldsNamed, GenericParam, Generics,
    Ident, Lit, Meta, MetaList, Result, Type, parse_quote,
};

const WITH_MERGE: bool = cfg!(feature = "merge");
const WITH_SERDE: bool = cfg!(feature = "serde");

/// Entry point: generate the shadow struct + `FromFile` impls.
pub fn impl_from_file(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = add_trait_bounds(input.generics.clone());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let file_ident = format_ident!("{name}File");

    let fields = extract_named_fields(input)?;
    let (field_assignments, file_fields) = process_fields(fields)?;

    let derive_clause = build_derive_clause();

    Ok(quote! {
        #derive_clause
        #vis struct #file_ident #ty_generics #where_clause {
            #(#file_fields),*
        }

        impl #impl_generics filecaster::FromFile for #name #ty_generics #where_clause {
            type Shadow = #file_ident #ty_generics;

            fn from_file(file: Option<Self::Shadow>) -> Self {
                let file = file.unwrap_or_default();
                Self {
                    #(#field_assignments),*
                }
            }
        }

        impl #impl_generics From<Option<#file_ident #ty_generics>> for #name #ty_generics #where_clause {
            fn from(value: Option<#file_ident #ty_generics>) -> Self {
                <Self as filecaster::FromFile>::from_file(value)
            }
        }

        impl #impl_generics From<#file_ident #ty_generics> for #name #ty_generics #where_clause {
            fn from(value: #file_ident #ty_generics) -> Self {
                <Self as filecaster::FromFile>::from_file(Some(value))
            }
        }
    })
}

/// Ensure we only work on named-field structs
fn extract_named_fields(input: &DeriveInput) -> Result<&FieldsNamed> {
    match &input.data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Named(fields) => Ok(fields),
            _ => Err(Error::new_spanned(
                &input.ident,
                r#"FromFile only works on structs with *named* fields.
Tuple structs and unit structs are not supported."#,
            )),
        },
        _ => Err(Error::new_spanned(
            &input.ident,
            r#"FromFile only works on structs.
Enums are not supported."#,
        )),
    }
}

/// Build the shadow field + assignment for one original field
fn build_file_field(field: &Field) -> Result<(TokenStream, TokenStream)> {
    let ident = field
        .ident
        .as_ref()
        .ok_or_else(|| Error::new_spanned(field, "Expected named fields"))?;
    let ty = &field.ty;

    let default_override = parse_from_file_default_attr(&field.attrs)?;

    let field_attrs = if WITH_MERGE {
        quote! { #[merge(strategy = merge::option::overwrite_none)] }
    } else {
        quote! {}
    };

    // Nested struct -> delegate to its own `FromFile` impl
    let shadow_ty = quote! { <#ty as filecaster::FromFile>::Shadow };
    let field_decl = quote! {
        #field_attrs
        pub #ident: Option<#shadow_ty>
    };

    let assign = build_file_assing(ident, ty, default_override);

    Ok((field_decl, assign))
}

fn build_file_assing(ident: &Ident, ty: &Type, default_override: Option<Expr>) -> TokenStream {
    if let Some(expr) = default_override {
        return quote! {
            #ident: file.#ident.map(|inner| <#ty as filecaster::FromFile>::from_file(Some(inner))).unwrap_or(#expr)
        };
    }
    quote! {
        #ident: <#ty as filecaster::FromFile>::from_file(file.#ident)
    }
}

/// Process all fields
fn process_fields(fields: &FieldsNamed) -> Result<(Vec<TokenStream>, Vec<TokenStream>)> {
    fields.named.iter().try_fold(
        (Vec::new(), Vec::new()),
        |(mut assignments, mut file_fields), field| {
            let (file_field, assignment) = build_file_field(field)?;
            file_fields.push(file_field);
            assignments.push(assignment);
            Ok((assignments, file_fields))
        },
    )
}

/// Derive clause for the shadow struct
fn build_derive_clause() -> TokenStream {
    let mut traits = vec![quote! {Debug}, quote! {Clone}, quote! {Default}];
    if WITH_SERDE {
        traits.extend([quote! { serde::Deserialize }, quote! { serde::Serialize }]);
    }

    if WITH_MERGE {
        traits.push(quote! { merge::Merge });
    }

    quote! { #[derive( #(#traits),* )] }
}

/// Add Default bound to every generic parameter
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ty) = param {
            ty.bounds.push(parse_quote!(Default));
        }
    }
    generics
}

/// Attribute parsing: `#[from_file(default = ...)]`
fn parse_from_file_default_attr(attrs: &[Attribute]) -> Result<Option<Expr>> {
    for attr in attrs {
        if !attr.path().is_ident("from_file") {
            continue; // Not a #[from_file] attribute, skip it
        }

        // Parse the content inside the parentheses of #[from_file(...)]
        return match &attr.meta {
            Meta::List(meta_list) => parse_default(meta_list),
            _ => Err(Error::new_spanned(
                attr,
                "Expected #[from_file(default = \"literal\")] or similar",
            )),
        };
    }
    Ok(None)
}

fn parse_default(list: &MetaList) -> Result<Option<Expr>> {
    let mut default_expr = None;
    list.parse_nested_meta(|meta| {
        if meta.path.is_ident("default") {
            let value = meta.value()?;
            let expr = value.parse::<Expr>()?;

            if let Expr::Lit(expr_lit) = &expr
                && let Lit::Str(lit_str) = &expr_lit.lit
            {
                default_expr = Some(parse_quote! {
                    #lit_str.to_string()
                });
                return Ok(());
            }
            default_expr = Some(expr);
        }
        Ok(())
    })?;
    Ok(default_expr)
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_none};
    use quote::ToTokens;

    use super::*;

    #[test]
    fn extract_named_fields_success() {
        let input: DeriveInput = parse_quote! {
            struct S { x: i32, y: String }
        };
        let fields = extract_named_fields(&input).unwrap();
        let names = fields
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["x", "y"]);
    }

    #[test]
    fn extract_named_fields_err_on_enum() {
        let input: DeriveInput = parse_quote! {
            enum E { A, B }
        };
        assert_err!(extract_named_fields(&input));
    }

    #[test]
    fn extract_named_fields_err_on_tuple_struct() {
        let input: DeriveInput = parse_quote! {
            struct T(i32, String);
        };
        assert_err!(extract_named_fields(&input));
    }

    #[test]
    fn parse_default_attrs_none() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[foo])];
        assert_none!(parse_from_file_default_attr(&attrs).unwrap());
    }

    #[test]
    fn process_fields_mixed() {
        let fields: FieldsNamed = parse_quote! {
            {
                #[from_file(default = 1)]
                a: u32,
                b: String,
            }
        };
        let (assign, file_fields) = process_fields(&fields).unwrap();
        // two fields
        assert_eq!(assign.len(), 2);
        assert_eq!(file_fields.len(), 2);
    }

    #[test]
    fn add_trait_bouds_appends_default() {
        let gens: Generics = parse_quote!(<T, U>);
        let new = add_trait_bounds(gens);
        let s = new.to_token_stream().to_string();
        assert!(s.contains("T : Default"));
        assert!(s.contains("U : Default"));
    }
}
