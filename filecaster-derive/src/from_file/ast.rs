use unsynn::{Ident, ToTokens, TokenStream};

use crate::from_file::grammar;

#[derive(Debug)]
pub struct Struct {
    pub vis: TokenStream,
    pub name: Ident,
    pub generics: TokenStream,
    pub fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Field {
    pub attrs: Vec<Attribute>,
    pub vis: TokenStream,
    pub name: Ident,
    pub ty: Ident,
}

#[derive(Debug)]
pub struct Attribute {
    pub path: Ident,
    pub tokens: TokenStream,
}

impl From<grammar::StructDef> for Struct {
    fn from(value: grammar::StructDef) -> Self {
        Self {
            vis: value.vis.to_token_stream(),
            name: value.name,
            generics: value.generics.to_token_stream(),
            fields: value
                .body
                .content
                .0
                .0
                .into_iter()
                .map(|x| x.value.into())
                .collect(),
        }
    }
}

impl From<grammar::Field> for Field {
    fn from(value: grammar::Field) -> Self {
        Self {
            attrs: value
                .attrs
                .unwrap_or_default()
                .into_iter()
                .map(Attribute::from)
                .collect(),
            vis: value.vis.to_token_stream(),
            name: value.name,
            ty: value.ty,
        }
    }
}

impl From<grammar::AttributeGroup> for Attribute {
    fn from(value: grammar::AttributeGroup) -> Self {
        let attr = value.bracket_group.content;
        Self {
            path: attr.path,
            tokens: attr.tokens.content,
        }
    }
}
