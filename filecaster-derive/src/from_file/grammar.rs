use unsynn::*;

keyword! {
    pub KwStruct = "struct";
    pub KwPub = "pub";
}

/*
pub struct Foo {
    #[attr("value")]
    pub bar: String,
}
*/
unsynn! {

    pub struct Attribute {
        pub path: Ident, // attr
        pub tokens: ParenthesisGroupContaining<TokenStream> // "value"
    }

    pub struct AttributeGroup {
        pub pound: Pound, // #
        pub bracket_group: BracketGroupContaining<Attribute> // [attr("value")]
    }

    pub struct Field {
        pub attrs: Option<Vec<AttributeGroup>>, // #[attr("value")]
        pub vis: Optional<KwPub>, // pub
        pub name: Ident, // bar
        pub colon: Colon, // :
        pub ty: Ident// String
    }

    pub struct StructBody(pub CommaDelimitedVec<Field>); // all fields

    pub struct StructDef {
        pub vis: Option<KwPub>, // pub
        pub kw_struct: KwStruct, // "struct" keyword
        pub name: Ident, // Foo
        pub generics: Optional<BracketGroupContaining<TokenStream>>,
        pub body: BraceGroupContaining<StructBody>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::assert_some;

    const SAMPLE: &str = r#"
        pub struct Foo {
            #[attr("value")]
            pub bar: String,
            #[attr("number")]
            pub baz: i32
        }
"#;

    #[test]
    fn parse_attribute_roundup() {
        let mut iter = r#"#[attr("value")]"#.to_token_iter();
        let attr = iter
            .parse::<AttributeGroup>()
            .expect("failed to parse Attribute");

        assert_eq!(attr.pound.tokens_to_string(), "#".tokens_to_string());

        let s = attr.bracket_group.tokens_to_string();
        assert!(s.contains("attr"));
        assert!(s.contains("\"value\""));

        let rt = attr.tokens_to_string();
        assert!(rt.contains("attr"));
        assert!(rt.contains("\"value\""));
    }

    #[test]
    fn parse_field_with_attr_and_vid_and_roundtrip() {
        let mut iter = r#"#[attr("value")] pub bar: String"#.to_token_iter();
        let field = iter.parse::<Field>().expect("failed to parse Field");

        assert_some!(&field.attrs);
        let attrs = field.attrs.as_ref().unwrap();
        assert_eq!(attrs.len(), 1);
        assert_eq!(field.name.tokens_to_string(), "bar".tokens_to_string());
        assert_eq!(field.ty.tokens_to_string(), "String".tokens_to_string());

        let tokens = field.tokens_to_string();
        assert!(tokens.contains("attr"));
        assert!(tokens.contains("bar"));
        assert!(tokens.contains("String"));
    }

    #[test]
    fn parse_struct_def_and_inspect_body() {
        let mut iter = SAMPLE.to_token_iter();

        let sdef = iter
            .parse::<StructDef>()
            .expect("failed to parse StructDef");

        assert_eq!(
            sdef.kw_struct.tokens_to_string(),
            "struct".tokens_to_string()
        );
        assert_eq!(sdef.name.tokens_to_string(), "Foo".tokens_to_string());

        let body = &sdef.body.content.0.0;
        assert_eq!(body.len(), 2);

        let field = &body[0].value;
        assert_some!(field.attrs.as_ref());
        assert_eq!(field.name.tokens_to_string(), "bar".tokens_to_string());
        assert_eq!(field.ty.tokens_to_string(), "String".tokens_to_string());

        let out = sdef.tokens_to_string();
        assert!(out.contains("pub"));
        assert!(out.contains("struct"));
        assert!(out.contains("Foo"));
        assert!(out.contains("attr"));
        assert!(out.contains("bar"));
        assert!(out.contains("String"));
    }

    #[test]
    fn parse_failure_for_incomplete_struct() {
        let mut iter = "pub struct Foo".to_token_iter();
        let res = iter.parse::<StructDef>();

        assert!(res.is_err(), "expected parse error for incomplete struct");
    }
}
