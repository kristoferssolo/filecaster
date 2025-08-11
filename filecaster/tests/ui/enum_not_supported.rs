use filecaster::FromFile;

#[derive(FromFile)]
enum MyEnum {
    A,
    B,
}

fn main() {}
