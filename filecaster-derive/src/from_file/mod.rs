mod ast;
mod codegen;
mod error;
mod grammar;
mod parser;

use crate::from_file::{codegen::generate_impl, error::FromFileError, parser::parse_scruct_info};
use proc_macro2::TokenStream;

pub fn impl_from_file(input: TokenStream) -> Result<TokenStream, FromFileError> {
    let info = parse_scruct_info(input)?;
    generate_impl(&info)
}
