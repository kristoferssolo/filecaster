mod from_file;

pub(crate) use from_file::impl_from_file;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_error]
#[proc_macro_derive(FromFile, attributes(from_file))]
pub fn derive_from_file(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    impl_from_file(&inp)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
