use filecaster::FromFile;

#[derive(Debug, Default, Clone, PartialEq, FromFile)]
pub struct Coordinates {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, PartialEq, FromFile)]
pub struct Parent {
    #[from_file(default = "Foo")]
    name: String,
    coordinates: Coordinates,
}
