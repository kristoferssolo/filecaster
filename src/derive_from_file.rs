use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Field, Fields, FieldsNamed, GenericParam, Generics,
    Lit, Meta, MetaList, Result, Type, TypePath, WhereClause, WherePredicate, parse_quote, parse2,
};

const WITH_MERGE: bool = cfg!(feature = "merge");

/// Entry point: generate the shadow struct + [`FromFile`] impls.
pub fn impl_from_file(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = add_trait_bounds(input.generics.clone());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let file_ident = format_ident!("{name}File");

    let fields = extract_named_fields(input)?;
    let (field_assignments, file_fields, default_bounds) = process_fields(fields)?;

    let where_clause = build_where_clause(where_clause.cloned(), default_bounds)?;
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
                "FromFile can only be derived for structs with named fields",
            )),
        },
        _ => Err(Error::new_spanned(
            &input.ident,
            "FromFile can only be derived for structs",
        )),
    }
}

/// Nested-struct detection
fn is_from_file_struct(ty: &Type) -> bool {
    if let Type::Path(TypePath { qself: None, path }) = ty {
        return path.segments.len() == 1;
    }
    false
}

/// Build the shadow field + assignment for one original field
fn build_file_field(field: &Field) -> Result<(TokenStream, TokenStream, Option<TokenStream>)> {
    let ident = field
        .ident
        .as_ref()
        .ok_or_else(|| Error::new_spanned(field, "Expected named fields"))?;
    let ty = &field.ty;

    let field_attrs = if WITH_MERGE {
        quote! {
            #[merge(strategy = merge::option::overwrite_none)]
        }
    } else {
        quote! {}
    };

    if is_from_file_struct(ty) {
        // Nested FromFile struct
        let field_decl = quote! {
            #field_attrs
            pub #ident: Option<#ty>
        };
        let assign = quote! {
            #ident: <#ty>::from_file(file.#ident)
        };
        return Ok((field_decl, assign, None));
    }

    // Primitive / leaf field
    let default_expr = parse_from_file_default_attr(&field.attrs)?;
    let field_decl = quote! {
        #field_attrs
        pub #ident: Option<#ty>
    };
    let assign = default_expr.map_or_else(
        || quote! { #ident: file.#ident.unwrap_or_default() },
        |expr| quote! { #ident: file.#ident.unwrap_or_else(|| #expr) },
    );
    let default = quote! { #ty: Default };

    Ok((field_decl, assign, Some(default)))
}

/// Process all fields
fn process_fields(
    fields: &FieldsNamed,
) -> Result<(Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>)> {
    fields.named.iter().try_fold(
        (Vec::new(), Vec::new(), Vec::new()),
        |(mut assignments, mut file_fields, mut defaults), field| {
            let (file_field, assignment, default_value) = build_file_field(field)?;
            file_fields.push(file_field);
            assignments.push(assignment);
            if let Some(value) = default_value {
                defaults.push(value);
            }
            Ok((assignments, file_fields, defaults))
        },
    )
}

/// Where-clause helpers
fn build_where_clause(
    where_clause: Option<WhereClause>,
    default_bounds: Vec<TokenStream>,
) -> Result<Option<WhereClause>> {
    if default_bounds.is_empty() {
        return Ok(where_clause);
    }

    let mut where_clause = where_clause;
    if let Some(wc) = &mut where_clause {
        for bound in default_bounds {
            let predicate = parse2::<WherePredicate>(bound.clone())
                .map_err(|_| Error::new_spanned(&bound, "Failed to parse where predicate"))?;
            wc.predicates.push(predicate);
        }
    } else {
        where_clause = Some(parse_quote!(where #(#default_bounds),*));
    }
    Ok(where_clause)
}

/// Derive clause for the shadow struct
fn build_derive_clause() -> TokenStream {
    if WITH_MERGE {
        return quote! {
            #[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, merge::Merge)]
        };
    }

    quote! {
        #[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
    }
}

/// Add Default bound to every generic parameter
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(Default));
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

            if let Expr::Lit(expr_lit) = &expr {
                if let Lit::Str(lit_str) = &expr_lit.lit {
                    default_expr = Some(parse_quote! {
                        #lit_str.to_string()
                    });
                    return Ok(());
                }
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
        let (assign, file_fields, bounds) = process_fields(&fields).unwrap();
        // two fields
        assert_eq!(assign.len(), 2);
        assert_eq!(file_fields.len(), 2);
        // a uses unwrap_or_else
        assert!(
            assign[0]
                .to_string()
                .contains("a : file . a . unwrap_or_else")
        );
        // b uses unwrap_or_default
        assert!(
            assign[1]
                .to_string()
                .contains("b : file . b . unwrap_or_default")
        );
        // default-bound should only mention String
        assert_eq!(bounds.len(), 1);
        assert!(bounds[0].to_string().contains("String : Default"));
    }

    #[test]
    fn build_where_clause_to_new() {
        let bounds = vec![quote! { A: Default }, quote! { B: Default }];
        let wc = build_where_clause(None, bounds).unwrap().unwrap();
        let s = wc.to_token_stream().to_string();
        assert!(s.contains("where A : Default , B : Default"));
    }

    #[test]
    fn build_where_clause_append_existing() {
        let orig: WhereClause = parse_quote!(where X: Clone);
        let bounds = vec![quote! { Y: Default }];
        let wc = build_where_clause(Some(orig.clone()), bounds)
            .unwrap()
            .unwrap();
        let preds: Vec<_> = wc
            .predicates
            .iter()
            .map(|p| p.to_token_stream().to_string())
            .collect();
        assert!(preds.contains(&"X : Clone".to_string()));
        assert!(preds.contains(&"Y : Default".to_string()));
    }

    #[test]
    fn build_where_clause_no_bounds_keeps_original() {
        let orig: WhereClause = parse_quote!(where Z: Eq);
        let wc = build_where_clause(Some(orig.clone()), vec![])
            .unwrap()
            .unwrap();
        let preds: Vec<_> = wc
            .predicates
            .iter()
            .map(|p| p.to_token_stream().to_string())
            .collect();
        assert_eq!(preds, vec!["Z : Eq".to_string()]);
    }

    #[test]
    fn build_derive_clause_defaults() {
        let derive_ts = build_derive_clause();
        let s = derive_ts.to_string();
        if WITH_MERGE {
            assert!(s.contains(
                "derive (Debug , Clone , Default , serde :: Deserialize , serde :: Serialize , merge :: Merge)"
            ));
        } else {
            assert!(s.contains(
                "derive (Debug , Clone , Default , serde :: Deserialize , serde :: Serialize)"
            ));
        }
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
