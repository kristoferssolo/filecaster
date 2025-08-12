use proc_macro2::{Ident, TokenStream};

#[derive(Debug)]
pub struct StructInfo {
    pub ident: Ident,
    pub vis: TokenStream,
    pub generics: TokenStream,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug)]
pub struct FieldInfo {
    pub ident: Ident,
    pub ty: TokenStream,
    pub attrs: Vec<AttributeInfo>,
}

#[derive(Debug)]
pub struct AttributeInfo {
    pub path: Ident,
    pub tokens: TokenStream,
}
