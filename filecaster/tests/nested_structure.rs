use filecaster::FromFile;

#[derive(Debug, Clone, PartialEq, FromFile)]
pub struct Coordinates {
    x: i32,
    y: i32,
}

impl Coordinates {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl CoordinatesFile {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
        }
    }
}

#[derive(Debug, Clone, PartialEq, FromFile)]
struct Wrapper {
    parent: Parent,
}

// And one more level
#[derive(Debug, Clone, PartialEq, FromFile)]
struct DoubleWrapper {
    wrapper: Wrapper,
}

#[derive(Debug, Clone, PartialEq, FromFile)]
pub struct Parent {
    #[from_file(default = "Foo")]
    name: String,
    coordinates: Coordinates,
}

#[test]
fn parent_all_defaults() {
    let p = Parent::from_file(None);
    assert_eq!(p.name, "Foo".to_string());
    assert_eq!(p.coordinates, Coordinates::new(0, 0));
}

#[test]
fn parent_partial_shadow_merges_defaults() {
    let shadow = ParentFile {
        name: None,
        coordinates: Some(CoordinatesFile::new(1, 2)),
    };
    let p = Parent::from_file(Some(shadow));
    assert_eq!(p.name, "Foo".to_string());
    assert_eq!(p.coordinates, Coordinates::new(1, 2));
}

#[test]
fn parent_full_shadow_overrides_everything() {
    let shadow = ParentFile {
        name: Some("Bar".into()),
        coordinates: Some(CoordinatesFile::new(42, 24)),
    };
    let p = Parent::from_file(Some(shadow));
    assert_eq!(p.name, "Bar".to_string());
    assert_eq!(p.coordinates, Coordinates::new(42, 24));
}

#[test]
fn wrapper_all_defaults() {
    // None → WrapperFile::default() → parent = Parent::from_file(None)
    let w = Wrapper::from_file(None);
    assert_eq!(w.parent.name, "Foo".to_string());
    assert_eq!(w.parent.coordinates, Coordinates::new(0, 0));
}

#[test]
fn wrapper_partial_parent() {
    // We supply only coordinates
    let shadow = WrapperFile {
        parent: Some(ParentFile {
            name: None,
            coordinates: Some(CoordinatesFile::new(5, -2)),
        }),
    };
    let w = Wrapper::from_file(Some(shadow));
    assert_eq!(w.parent.name, "Foo".to_string());
    assert_eq!(w.parent.coordinates, Coordinates::new(5, -2));
}

#[test]
fn wrapper_full_parent_override() {
    let shadow = WrapperFile {
        parent: Some(ParentFile {
            name: Some("Baz".into()),
            coordinates: Some(CoordinatesFile::new(1, 1)),
        }),
    };
    let w = Wrapper::from_file(Some(shadow));
    assert_eq!(w.parent.name, "Baz".to_string());
    assert_eq!(w.parent.coordinates, Coordinates::new(1, 1));
}

#[test]
fn double_wrapper_all_defaults() {
    let dw = DoubleWrapper::from_file(None);
    assert_eq!(dw.wrapper.parent.name, "Foo".to_string());
    assert_eq!(dw.wrapper.parent.coordinates, Coordinates::new(0, 0));
}

#[test]
fn double_wrapper_partial_deep() {
    let shadow = DoubleWrapperFile {
        wrapper: Some(WrapperFile {
            parent: Some(ParentFile {
                name: None,
                coordinates: Some(CoordinatesFile::new(10, 20)),
            }),
        }),
    };
    let dw = DoubleWrapper::from_file(Some(shadow));
    assert_eq!(dw.wrapper.parent.name, "Foo".to_string());
    assert_eq!(dw.wrapper.parent.coordinates, Coordinates::new(10, 20));
}

#[test]
fn double_wrapper_full_override_deep() {
    let shadow = DoubleWrapperFile {
        wrapper: Some(WrapperFile {
            parent: Some(ParentFile {
                name: Some("Deep".into()),
                coordinates: Some(CoordinatesFile::new(3, 4)),
            }),
        }),
    };
    let dw = DoubleWrapper::from_file(Some(shadow));
    assert_eq!(dw.wrapper.parent.name, "Deep".to_string());
    assert_eq!(dw.wrapper.parent.coordinates, Coordinates::new(3, 4));
}
