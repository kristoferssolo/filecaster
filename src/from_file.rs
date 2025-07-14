use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Fields, FieldsNamed, GenericParam, Generics, Meta,
    Result, WhereClause, WherePredicate, parse_quote, parse2,
};

const WITH_MERGE: bool = cfg!(feature = "merge");

pub fn impl_from_file(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = add_trait_bouds(input.generics.clone());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let file_ident = format_ident!("{name}File");

    let fields = extract_named_fields(input)?;
    let (field_assignments, file_fields, default_bounds) = process_fields(fields)?;

    let where_clause = build_where_clause(where_clause.cloned(), default_bounds)?;

    let derive_clause = build_derive_clause();

    Ok(quote! {
        #derive_clause
        #vis struct #file_ident #where_clause {
            #(#file_fields),*
        }

        impl #impl_generics #name #ty_generics #where_clause {
            pub fn from_file(file: Option<#file_ident #ty_generics>) -> Self {
                let file = file.unwrap_or_default();
                Self {
                    #(#field_assignments),*
                }
            }
        }

        impl #impl_generics From<Option<#file_ident #ty_generics>> for #name #ty_generics #where_clause {
            fn from(value: Option<#file_ident #ty_generics>) -> Self {
                Self::from_file(value)
            }
        }
    })
}

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

fn process_fields(
    fields: &FieldsNamed,
) -> Result<(Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>)> {
    let mut field_assignments = Vec::new();
    let mut file_fields = Vec::new();
    let mut default_bounds = Vec::new();

    for field in &fields.named {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new_spanned(field, "Expected named fields"))?;
        let ty = &field.ty;

        let default_expr = parse_from_file_default_attr(&field.attrs)?;

        let field_attrs = if WITH_MERGE {
            quote! {
                #[merge(strategy = merge::option::overwrite_none)]
            }
        } else {
            quote! {}
        };
        file_fields.push(quote! {
            #field_attrs
            pub #ident: Option<#ty>
        });

        if let Some(expr) = default_expr {
            field_assignments.push(quote! {
                #ident: file.#ident.unwrap_or_else(|| #expr)
            });
        } else {
            default_bounds.push(quote! { #ty: Default });
            field_assignments.push(quote! {
                #ident: file.#ident.unwrap_or_default()
            });
        }
    }

    Ok((field_assignments, file_fields, default_bounds))
}

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

fn add_trait_bouds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(Default));
        }
    }
    generics
}

/// Parses attributes for `#[from_file(default = ...)]`
fn parse_from_file_default_attr(attrs: &[Attribute]) -> Result<Option<Expr>> {
    for attr in attrs {
        if !attr.path().is_ident("from_file") {
            continue; // Not a #[from_file] attribute, skip it
        }

        // Parse the content inside the parentheses of #[from_file(...)]
        return match &attr.meta {
            Meta::List(meta_list) => {
                let mut default_expr = None;
                meta_list.parse_nested_meta(|meta| {
                    if meta.path.is_ident("default") {
                        let value = meta.value()?;
                        let expr = value.parse::<Expr>()?;
                        default_expr = Some(expr);
                    }
                    Ok(())
                })?;
                Ok(default_expr)
            }
            _ => Err(Error::new_spanned(
                attr,
                "Expected #[from_file(default = \"literal\")] or similar",
            )),
        };
    }
    Ok(None)
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
    fn parse_default_attrs_picks_first_default() {
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[foo]),
            parse_quote!(#[from_file(default = "bar")]),
            parse_quote!(#[from_file(default = "baz")]),
        ];
        let expr = parse_from_file_default_attr(&attrs).unwrap().unwrap();
        // should pick the first default attribute
        assert_eq!(expr, parse_quote!("bar"));
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
        dbg!(&s);
        assert!(s.contains(
            "derive (Debug , Clone , Default , serde :: Deserialize , serde :: Serialize)"
        ));
    }

    #[test]
    fn add_trait_bouds_appends_default() {
        let gens: Generics = parse_quote!(<T, U>);
        let new = add_trait_bouds(gens);
        let s = new.to_token_stream().to_string();
        assert!(s.contains("T : Default"));
        assert!(s.contains("U : Default"));
    }
}
