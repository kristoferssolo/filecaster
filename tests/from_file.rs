use filecaster::FromFile;

#[derive(Debug, Clone, PartialEq, FromFile)]
struct Simple {
    x: i32,
    #[from_file(default = "hello")]
    y: String,
}

#[derive(Debug, Clone, PartialEq, FromFile)]
struct NumericDefault {
    a: i32,
    #[from_file(default = 42)]
    b: i32,
}

#[test]
fn test_simple_defaults() {
    // No file passed -> all fields fall back to defaults
    let s = Simple::from_file(None);
    assert_eq!(
        s,
        Simple {
            x: 0,
            y: "hello".to_string(),
        }
    );
}

#[test]
fn test_simple_override() {
    // Manually construct the generated `SimpleFile` and override both fields
    let file = SimpleFile {
        x: Some(10),
        y: Some("world".to_string()),
    };
    let s = Simple::from_file(Some(file));
    assert_eq!(
        s,
        Simple {
            x: 10,
            y: "world".to_string(),
        }
    );
}

#[test]
fn test_simple_serde_empty() {
    // Deserialize from JSON missing both fields -> both None
    let json = "{}";
    let file: SimpleFile = serde_json::from_str(json).unwrap();
    let s = Simple::from_file(Some(file));
    assert_eq!(s.x, 0);
    assert_eq!(s.y, "hello".to_string());
}

#[test]
fn test_simple_serde_partial() {
    // Deserialize from JSON with only `x`
    let json = r#"{"x":5}"#;
    let file: SimpleFile = serde_json::from_str(json).unwrap();
    let s = Simple::from_file(Some(file));
    assert_eq!(s.x, 5);
    assert_eq!(s.y, "hello".to_string());
}

#[test]
fn test_simple_serde_full() {
    // Deserialize from JSON with both fields
    let json = r#"{"x":7,"y":"rust"}"#;
    let file: SimpleFile = serde_json::from_str(json).unwrap();
    let s = Simple::from_file(Some(file));
    assert_eq!(s.x, 7);
    assert_eq!(s.y, "rust".to_string());
}

#[test]
fn test_numeric_default() {
    // No file -> default `b = 42`
    let n = NumericDefault::from_file(None);
    assert_eq!(n, NumericDefault { a: 0, b: 42 });

    // Override both
    let file = NumericDefaultFile {
        a: Some(7),
        b: Some(99),
    };
    let n2 = NumericDefault::from_file(Some(file));
    assert_eq!(n2, NumericDefault { a: 7, b: 99 });
}

#[cfg(feature = "merge")]
mod merge_tests {
    use super::*;
    use merge::Merge;

    #[test]
    fn test_merge_simple_file() {
        let mut f1 = SimpleFile {
            x: Some(1),
            y: None,
        };
        let f2 = SimpleFile {
            x: None,
            y: Some("foo".to_string()),
        };
        f1.merge(f2);
        assert_eq!(f1.x, Some(1));
        assert_eq!(f1.y, Some("foo".to_string()));
    }
}
