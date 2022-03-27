use std::fs::File;
use std::io::Read;

use serde_derive::Deserialize;
use tracing::error;

#[derive(Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub log_file: String,
    pub db_file: String,
    pub listen: String,
    pub log_level: String,
}

impl Config {
    // Read config file. Panic if any error occurs.
    pub fn from_file(file_path: &str) -> Config {
        let mut file = File::open(file_path)
            .unwrap_or_else(|err| {
                error!("Cannot open config file {}: {:?}", file_path, err);
                panic!();
            });
        let mut config = String::new();
        file.read_to_string(&mut config)
            .unwrap_or_else(|err| {
                error!("Cannot read config file {}: {:?}.", file_path, err);
                panic!();
            });
        return serde_json::from_str(config.as_str()).unwrap_or_else(|err| {
            error!("Cannot decode config file: {:?}.", err);
            panic!();
        });
    }
}