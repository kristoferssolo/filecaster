use filecaster::FromFile;
use std::fs;

#[derive(Debug, FromFile)]
pub struct MyData {
    #[from_file(default = "default key")]
    pub key: String,
    #[from_file(default = 0)]
    pub number: i64,
}

fn main() {
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let data_dir = current_dir.join("filecaster/examples/data");

    let json_path = data_dir.join("simple.json");
    let toml_path = data_dir.join("simple.toml");

    let json_content = fs::read_to_string(&json_path)
        .unwrap_or_else(|e| panic!("Failed to read JSON file at {:?}: {}", json_path, e));
    let json_data: MyData = serde_json::from_str::<MyDataFile>(&json_content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON in {:?}: {}", json_path, e))
        .into();

    assert_eq!(json_data.key, "json key".to_string());
    assert_eq!(json_data.number, 123);

    let toml_content = fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Failed to read TOML file at {:?}: {}", toml_path, e));
    let toml_data: MyData = toml::from_str::<MyDataFile>(&toml_content)
        .unwrap_or_else(|e| panic!("Failed to parse TOML in {:?}: {}", toml_path, e))
        .into();

    assert_eq!(toml_data.key, "toml key".to_string());
    assert_eq!(toml_data.number, 456);

    dbg!(&json_data);
    dbg!(&toml_data);
}
