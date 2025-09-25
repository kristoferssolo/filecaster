use filecaster::FromFile;
use std::fs;

#[derive(Debug, FromFile)]
pub struct MyData {
    #[from_file(default = "default key")]
    pub key: String,
    pub number: i32,
    pub exists: bool,
}

fn main() {
    // Get the absolute current directory
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    // Path to the data directory
    let data_dir = current_dir.join("filecaster/examples/data");

    // Paths to JSON and TOML files
    let json_path = data_dir.join("simple.json");
    let toml_path = data_dir.join("simple.toml");

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
    assert_eq!(json_data.key, "json key".to_string());
    assert_eq!(json_data.number, 123);
    assert!(!json_data.exists); // `bool::default()` is `false`

    assert_eq!(toml_data.key, "default key".to_string());
    assert_eq!(toml_data.number, 456);
    assert!(toml_data.exists);
}
