mod ast;
mod error;
mod grammar;
mod parser;

use crate::from_file::error::FromFileError;
use unsynn::TokenStream;

pub fn impl_from_file(input: TokenStream) -> Result<TokenStream, FromFileError> {
    todo!()
}
