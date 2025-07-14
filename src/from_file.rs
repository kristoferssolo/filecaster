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
        #vis struct #file_ident {
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

        let default_expr = parse_default_attrs(&field.attrs)?;

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

fn parse_default_attrs(attrs: &[Attribute]) -> Result<Option<Expr>> {
    for attr in attrs {
        if let Some(expr) = parse_default_attr(attr)? {
            return Ok(Some(expr));
        }
    }
    Ok(None)
}

fn parse_default_attr(attr: &Attribute) -> Result<Option<Expr>> {
    if !attr.path().is_ident("default") {
        return Ok(None);
    }

    let meta = attr.parse_args::<Meta>()?;
    let Meta::NameValue(name_value) = meta else {
        return Err(Error::new_spanned(attr, "Expected #[default = \"value\"]"));
    };

    match name_value.value {
        Expr::Lit(expr_lit) => Ok(Some(Expr::Lit(expr_lit))),
        _ => Err(Error::new_spanned(
            &name_value.value,
            "Default value must be a literal",
        )),
    }
}
