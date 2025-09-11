use crate::from_file::ast::Attribute;
use std::iter::{Peekable, once};
use unsynn::*;

pub fn parse_from_file_default_attr(attrs: &[Attribute]) -> Result<Option<TokenStream>> {
    for attr in attrs {
        if attr.path == "from_file" {
            return extract_default_token(attr.tokens.clone())
                .map(Some)
                .ok_or_else(|| Error::no_error()); // TODO: different error
        }
    }
    Ok(None)
}

fn extract_default_token(token: TokenStream) -> Option<TokenStream> {
    let mut iter = token.into_token_iter().peekable();

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
        let is_comma = matches!(tt, TokenTree::Punct(p) if p.as_char() == ',');
        if is_comma {
            iter.next();
            break;
        }
        expr.extend(once(iter.next().unwrap()));
    }
    expr
}
