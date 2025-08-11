use filecaster::FromFile;

#[derive(FromFile)]
struct MyStruct {
    #[from_file(unknown)]
    field: i32,
}

fn main() {}
