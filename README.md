# derive(FromFile)

Procedural macro to derive configuration from files, with optional merging capabilities.

## Features

- **Derive Configuration:** Easily load configuration from files into your Rust structs.
- **Default Values:** Specify default values for struct fields using the `#[default = "..."]` attribute.
- **Optional Merging:** When the `merge` feature is enabled, allows merging multiple configuration sources.

## Usage

```toml
[dependencies]
filecaster = "0.1"
```

Example:

```rust
use filecaster::FromFile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize, FromFile)]
pub struct MyConfig {
    pub host: String,
    #[default = "8080"]
    pub port: u16,
    #[default = "false"]
    pub enabled: bool,
}

fn main() {
    // Simulate loading from a file (e.g., JSON, YAML)
    let file_content = r#"
        {
            "host": "localhost"
        }
    "#;

    let config_from_file: MyConfig = serde_json::from_str(file_content).unwrap();
    let config = MyConfig::from_file(Some(config_from_file));

    println!("Config: {:?}", config);
    // Expected output: Config { host: "localhost", port: 8080, enabled: false }
}
```

## Documentation

Full documentation is available at [docs.rs](https://docs.rs/filecaster).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
