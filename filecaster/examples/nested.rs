use filecaster::FromFile;
use std::fs;

#[derive(Debug, FromFile)]
pub struct InnerData {
    #[from_file(default = "inner default")]
    pub inner_key: String,
    #[from_file(default = 42)]
    pub inner_number: i32,
}

#[derive(Debug, FromFile)]
pub struct MyData {
    #[from_file(default = "default key")]
    pub key: String,
    #[from_file(default = 0)]
    pub number: i32,
    pub nested: InnerData,
}

fn main() {
    // Get the absolute current directory
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    // Path to the data directory
    let data_dir = current_dir.join("filecaster/examples/data");

    // Paths to JSON and TOML files
    let json_path = data_dir.join("nested.json");
    let toml_path = data_dir.join("nested.toml");

    // Read and parse JSON file
    let json_content = fs::read_to_string(&json_path)
        .unwrap_or_else(|e| panic!("Failed to read JSON file at {:?}: {}", json_path, e));
    let json_data: MyData = serde_json::from_str::<MyDataFile>(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON in {:?}: {}", json_path, e))
        .into();

    // Read and parse TOML file
    let toml_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Failed to read TOML file at {:?}: {}", toml_path, e));
    let toml_data: MyData = toml::from_str::<MyDataFile>(&toml_content)
        .unwrap_or_else(|e| panic!("Failed to parse TOML in {:?}: {}", toml_path, e))
        .into();

    // Output the parsed data
    dbg!(&json_data);
    dbg!(&toml_data);

    // Example assertions (adjust based on your actual file contents)
    assert_eq!(json_data.key, "json key");
    assert_eq!(json_data.number, 123);
    assert_eq!(json_data.nested.inner_key, "inner default");
    assert_eq!(json_data.nested.inner_number, 42);

    assert_eq!(toml_data.key, "toml key");
    assert_eq!(toml_data.number, 456);
    assert_eq!(toml_data.nested.inner_key, "inner toml key");
    assert_eq!(toml_data.nested.inner_number, 99);
}
