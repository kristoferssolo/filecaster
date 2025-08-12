use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FromFileError {
    #[error("FromFile only works on structs with named fields")]
    NotNamedStruct { span: Span },
    #[error("Invalid #[from_file] attribute format")]
    InvalidAttribute { span: Span },
}

impl FromFileError {
    pub fn to_compile_error(&self) -> TokenStream {
        let msg = self.to_string();
        match self {
            FromFileError::NotNamedStruct { span } | FromFileError::InvalidAttribute { span } => {
                quote_spanned!(*span => compile_error!(#msg))
            }
        }
    }
}
