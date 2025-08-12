use std::iter::{Peekable, once};

use crate::from_file::ast::AttributeInfo;
use crate::from_file::grammar::{Field, StructDef};
use crate::from_file::{
    ast::{FieldInfo, StructInfo},
    error::FromFileError,
};
use proc_macro2::{Ident, Span, TokenStream};
use unsynn::TokenTree;
use unsynn::{IParse, ToTokens};

pub fn parse_scruct_info(input: TokenStream) -> Result<StructInfo, FromFileError> {
    let mut iter = input.to_token_iter();
    let def = iter
        .parse::<StructDef>()
        .map_err(|_| FromFileError::NotNamedStruct {
            span: Span::call_site(),
        })?;

    Ok(def.into())
}

pub fn parse_from_file_default_attr(
    attrs: &[AttributeInfo],
) -> Result<Option<TokenStream>, FromFileError> {
    for attr in attrs {
        if attr.path == "from_file" {
            return extract_default_token(attr.tokens.clone())
                .map(Some)
                .ok_or_else(|| FromFileError::InvalidAttribute {
                    span: attr.path.span(),
                });
        }
    }
    Ok(None)
}

fn extract_default_token(tokens: TokenStream) -> Option<TokenStream> {
    let mut iter = tokens.into_iter().peekable();

    while let Some(TokenTree::Ident(id)) = iter.next() {
        if id != "default" {
            continue;
        }

        match iter.next() {
            Some(TokenTree::Punct(eq)) if eq.as_char() == '=' => {
                return Some(collect_until_commas(&mut iter));
            }
            _ => return None,
        }
    }
    None
}

fn collect_until_commas<I>(iter: &mut Peekable<I>) -> TokenStream
where
    I: Iterator<Item = TokenTree>,
{
    let mut expr = TokenStream::new();
    while let Some(tt) = iter.peek() {
        let is_comma = matches!(tt, TokenTree::Punct(p) if p.as_char() ==',');
        if is_comma {
            iter.next();
            break;
        }
        expr.extend(once(iter.next().unwrap()));
    }
    expr
}

impl From<StructDef> for StructInfo {
    fn from(value: StructDef) -> Self {
        Self {
            ident: value.name,
            vis: value.vis,
            generics: value.generics,
            fields: value
                .body
                .content
                .0
                .into_iter()
                .map(|d| d.value.into())
                .collect(),
        }
    }
}

impl From<Field> for FieldInfo {
    fn from(value: Field) -> Self {
        Self {
            ident: value.name,
            ty: value.ty,
            attrs: value
                .attrs
                .into_iter()
                .map(|ts| {
                    let path = extract_attr_path(ts.clone());
                    AttributeInfo { path, tokens: ts }
                })
                .collect(),
        }
    }
}

fn extract_attr_path(attr_tokens: TokenStream) -> Ident {
    attr_tokens
        .into_iter()
        .find_map(|tt| {
            if let TokenTree::Ident(id) = tt {
                Some(id)
            } else {
                None
            }
        })
        .unwrap_or_else(|| Ident::new("unknown", Span::call_site()))
}
