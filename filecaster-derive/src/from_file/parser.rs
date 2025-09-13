use crate::from_file::ast::Attribute;
use std::iter::{Peekable, once};
use unsynn::*;

pub fn parse_from_file_default_attr(attrs: &[Attribute]) -> Result<Option<TokenStream>> {
    for attr in attrs {
        if attr.path.tokens_to_string().trim() == "from_file" {
            let tokens = attr.tokens.clone();
            let iter = tokens.clone().into_token_iter();

            match extract_default_token(tokens) {
                Some(ts) => return Ok(Some(ts)),
                None => {
                    return Error::other(&iter, "missing default value in #[from_file]".into());
                }
            };
        }
    }
    Ok(None)
}

fn extract_default_token(token: TokenStream) -> Option<TokenStream> {
    let mut iter = token.into_token_iter().peekable();
    while let Some(tt) = iter.next() {
        match &tt {
            TokenTree::Ident(id) if id == "default" => {
                // accept optional whitespace/punct and then '='
                // next non-whitespace token should be '='
                if let Some(next) = iter.peek()
                    && let TokenTree::Punct(p) = next
                    && p.as_char() == '='
                {
                    iter.next();
                    return Some(collect_until_commas(&mut iter));
                }
                // if we see "default" without '=', treat as parse failure
                return None;
            }
            _ => continue,
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
        // peek returned Some, so unwrap is safe
        expr.extend(once(iter.next().expect("this should be impossible to see")));
    }
    expr
}
