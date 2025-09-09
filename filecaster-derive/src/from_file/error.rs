use thiserror::Error;
use unsynn::TokenStream;

#[derive(Debug, Error)]
pub enum FromFileError {}

impl FromFileError {
    pub fn to_compile_error(&self) -> TokenStream {
        todo!()
    }
}
