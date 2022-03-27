use std::fs::File;
use std::io::Read;

use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub log_file: String,
    pub db_file: String,
}

impl Config {
    // Read config file. Panic if any error occurs.
    pub fn from_file(file_path: &str) -> Config {
        let mut file = File::open(file_path)
            .unwrap_or_else(|_| panic!("cannot open file {}", file_path));
        let mut config = String::new();
        file.read_to_string(&mut config)
            .unwrap_or_else(|_| panic!("cannot read config file {}", file_path));
        return serde_json::from_str(config.as_str())
            .expect("cannot decode config file in JSON");
    }
}