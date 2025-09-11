mod ast;
mod codegen;
mod grammar;
mod parser;

use crate::from_file::{codegen::generate_impl, grammar::StructDef};
use unsynn::*;

pub fn impl_from_file(input: TokenStream) -> Result<TokenStream> {
    let parsed = input.to_token_iter().parse::<StructDef>()?;
    generate_impl(&parsed.into())
}
