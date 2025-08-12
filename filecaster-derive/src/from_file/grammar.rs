use unsynn::*;

unsynn! {
    pub struct Field {
        pub attrs: Vec<TokenStream>,
        pub name: Ident,
        pub colon: Colon,
        pub ty: TokenStream
    }

    pub struct StructBody(pub CommaDelimitedVec<Field>);

    pub struct StructDef {
        pub vis: TokenStream,
        pub kw_struct: Ident,
        pub name: Ident,
        pub generics: TokenStream,
        pub body: BraceGroupContaining<StructBody>
    }
}
