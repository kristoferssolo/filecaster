use filecaster::FromFile;

#[derive(FromFile)]
struct MyTuple(i32, String);

fn main() {}
