use filecaster::FromFile;
use std::{fs::File, io::Write};
use tempfile::NamedTempFile;

#[derive(Debug, Clone, PartialEq, FromFile)]
pub struct Simple {
    x: i32,
    #[from_file(default = "hello")]
    y: String,
}

#[derive(Debug, Clone, PartialEq, FromFile)]
pub struct NumericDefault {
    a: i32,
    #[from_file(default = 42)]
    b: i32,
}

#[test]
fn test_json_tempfile_full() {
    let json = r#"{"x": 2, "y": "temp"}"#;
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp.as_file_mut(), "{}", json).unwrap();

    let file = File::open(tmp.path()).unwrap();
    let file_struct: SimpleFile = serde_json::from_reader(file).unwrap();
    let s = Simple::from_file(Some(file_struct));

    assert_eq!(
        s,
        Simple {
            x: 2,
            y: "temp".to_string()
        }
    );
}

#[test]
fn test_json_tempfile_partial() {
    let json = r#"{"x": 5}"#;
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp.as_file_mut(), "{}", json).unwrap();

    let file = File::open(tmp.path()).unwrap();
    let file_struct: SimpleFile = serde_json::from_reader(file).unwrap();
    let s = Simple::from_file(Some(file_struct));

    assert_eq!(
        s,
        Simple {
            x: 5,
            y: "hello".to_string()
        }
    );
}

#[test]
fn test_toml_tempfile_full() {
    let toml_str = r#"
        x = 7
        y = "toml_test"
    "#;
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp.as_file_mut(), "{}", toml_str).unwrap();

    let content = std::fs::read_to_string(tmp.path()).unwrap();
    let file_struct: SimpleFile = toml::from_str(&content).unwrap();
    let s = Simple::from_file(Some(file_struct));

    assert_eq!(
        s,
        Simple {
            x: 7,
            y: "toml_test".to_string()
        }
    );
}

#[test]
fn test_toml_tempfile_partial() {
    let toml_str = r#"x = 15"#;
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp.as_file_mut(), "{}", toml_str).unwrap();

    let content = std::fs::read_to_string(tmp.path()).unwrap();
    let file_struct: SimpleFile = toml::from_str(&content).unwrap();
    let s = Simple::from_file(Some(file_struct));

    assert_eq!(
        s,
        Simple {
            x: 15,
            y: "hello".to_string()
        }
    );
}

#[test]
fn test_numeric_default_toml() {
    let toml_str = r#"a = 100"#;
    let mut tmp = NamedTempFile::new().unwrap();
    write!(tmp.as_file_mut(), "{}", toml_str).unwrap();

    let content = std::fs::read_to_string(tmp.path()).unwrap();
    let file_struct: NumericDefaultFile = toml::from_str(&content).unwrap();
    let n = NumericDefault::from_file(Some(file_struct));

    assert_eq!(n, NumericDefault { a: 100, b: 42 });
}

#[cfg(feature = "merge")]
#[test]
fn test_merge_from_toml() {
    use merge::Merge;
    let toml1 = r#"x = 1"#;
    let toml2 = r#"y = "merged""#;

    let mut tmp1 = NamedTempFile::new().unwrap();
    write!(tmp1.as_file_mut(), "{}", toml1).unwrap();
    let content1 = std::fs::read_to_string(tmp1.path()).unwrap();
    let mut f1: SimpleFile = toml::from_str(&content1).unwrap();

    let mut tmp2 = NamedTempFile::new().unwrap();
    write!(tmp2.as_file_mut(), "{}", toml2).unwrap();
    let content2 = std::fs::read_to_string(tmp2.path()).unwrap();
    let f2: SimpleFile = toml::from_str(&content2).unwrap();

    f1.merge(f2);
    assert_eq!(f1.x, Some(1));
    assert_eq!(f1.y, Some("merged".to_string()));

    // Finally convert into the real struct
    let s = Simple::from_file(Some(f1));
    assert_eq!(
        s,
        Simple {
            x: 1,
            y: "merged".to_string()
        }
    );
}
