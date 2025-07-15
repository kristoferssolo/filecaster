//! # filecaster
//!
//! `filecaster` is a small `proc-macro` crate that provides a derive‐macro
//! `#[derive(FromFile)]` to make it trivial to load partial configurations
//! from files, merge them with defaults, and get a fully‐populated struct.
//!
//! ## What it does
//!
//! For any struct with named fields, `#[derive(FromFile)]` generates:
//!
//! 1. A companion `<YourStruct>NameFile` struct in which each field is wrapped
//!    in `Option<...>`.
//! 2. A constructor `YourStruct::from_file(file: Option<YourStructFile>) -> YourStruct`
//!    that takes your partially‐filled file struct, fills in `None` fields
//!    with either:
//!    - an expression you supply via `#[from_file(default = ...)]`, or
//!    - `Default::default()` (requires `T: Default`)
//! 3. An implementation of `From<Option<YourStructFile>> for YourStruct`.
//!
//! Because each field in the file‐struct is optional, you can deserialize
//! e.g. JSON, YAML or TOML into it via Serde, then call `.from_file(...)`
//! to get your final struct.
//!
//! ## Optional per‐field defaults
//!
//! Use a `#[from_file(default = <expr>)]` attribute on any field to override
//! the fallback value. You may supply any expression valid in that struct’s
//! context. If you omit it, the macro will require `T: Default` and call
//! `unwrap_or_default()`.
//!
//! Example:
//!
//! ```rust
//! use filecaster::FromFile;
//!
//! #[derive(Debug, Clone, FromFile)]
//! struct AppConfig {
//!     /// If the user does not specify a host, use `"127.0.0.1"`.
//!     #[from_file(default = "127.0.0.1")]
//!     host: String,
//!
//!     /// Number of worker threads; defaults to `4`.
//!     #[from_file(default = 4)]
//!     workers: usize,
//!
//!     /// If not set, use `false`.
//!     auto_reload: bool,  // requires `bool: Default`
//! }
//!
//! let file_content = r#"
//!     {
//!         "host": "localhost"
//!     }
//! "#;
//!
//! let config_from_file = serde_json::from_str::<AppConfigFile>(file_content).unwrap();
//! // After deserializing the partial config from disk (e.g. with Serde):
//! let cfg = AppConfig::from_file(Some(config_from_file));
//! println!("{cfg:#?}");
//! ```
//!
//! ## Feature flags
//!
//! - `merge`
//!   If you enable the `merge` feature, the generated `<Name>File` struct will
//!   also derive `merge::Merge`, and you can layer multiple partial files
//!   together before calling `.from_file(...)`. Any field‐level merge strategy
//!   annotations (`#[merge(...)]`) are applied automatically.
//!
//! ## Limitations
//!  
//! - Only works on structs with _named_ fields (no tuple‐structs or enums).
//! - All fields without a `#[from_file(default = ...)]` must implement `Default`.
//!  
//! ## License
//!  
//! MIT OR Apache-2.0

mod from_file;

pub(crate) use from_file::impl_from_file;
use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;
use syn::{DeriveInput, parse_macro_input};

/// Implements the `FromFile` derive macro.
///
/// This macro processes the `#[from_file]` attribute on structs to generate
/// code for loading data from files.
#[proc_macro_error]
#[proc_macro_derive(FromFile, attributes(from_file))]
pub fn derive_from_file(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    impl_from_file(&inp)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
