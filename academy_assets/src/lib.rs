include!(env!("ASSETS"));

pub const CONFIG_TOML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../config.toml"));
