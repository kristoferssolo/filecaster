use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Error, Expr, Fields, GenericParam, Generics, Meta, Result,
    WherePredicate, parse_quote, parse2,
};

const WITH_MERGE: bool = cfg!(feature = "merge");

pub fn impl_from_file(input: &DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let vis = &input.vis;
    let generics = add_trait_bouts(input.generics.clone());
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let file_ident = format_ident!("{name}File");

    let fields = match &input.data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    &input.ident,
                    "FromFile can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input.ident,
                "FromFile can only be derived for structs",
            ));
        }
    };

    let mut field_assignments = Vec::new();
    let mut file_fields = Vec::new();
    let mut default_bounds = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let mut default_expr = None;
        for attr in &field.attrs {
            if let Some(expr) = parse_default_attr(attr)? {
                default_expr = Some(expr);
                break;
            }
        }

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

    let where_clause = if default_bounds.is_empty() {
        where_clause.cloned()
    } else {
        let mut where_clause = where_clause.cloned();
        if let Some(wc) = &mut where_clause {
            wc.predicates.extend(
                default_bounds
                    .into_iter()
                    .map(|bound| parse2::<WherePredicate>(bound).unwrap()),
            );
        } else {
            where_clause = Some(parse_quote!(where #(#default_bounds),*));
        }
        where_clause
    };

    // Conditionally include Merge derive based on feature
    let derive_clause = if WITH_MERGE {
        quote! {
            #[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize, merge::Merge)]
        }
    } else {
        quote! {
            #[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
        }
    };

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
    }.into())
}

fn add_trait_bouts(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(Default));
        }
    }
    generics
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
