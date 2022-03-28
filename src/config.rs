use std::fs::File;
use std::io::Read;

use serde_derive::Deserialize;
use tracing::error;

pub const DEFAULT_LOG_LEVEL: &str = "DEBUG";

#[derive(Deserialize)]
pub struct Config {
    pub bot_token: String,
    #[serde(default = "Config::default_log_file")]
    pub log_file: String,
    #[serde(default = "Config::default_db_file")]
    pub db_file: String,
    #[serde(default = "Config::default_listen")]
    pub listen: String,
    #[serde(default = "Config::default_log_level")]
    pub log_level: String,
    #[serde(default = "Config::default_max_body_size")]
    pub max_body_size: u64,
}

impl Config {
    fn default_log_level() -> String {
        String::from(DEFAULT_LOG_LEVEL)
    }

    fn default_log_file() -> String {
        String::new() // empty string means not logging to file
    }

    fn default_max_body_size() -> u64 {
        1024 * 16
    }

    fn default_listen() -> String {
        String::from("localhost:8080")
    }

    fn default_db_file() -> String {
        String::from("kimikuri.db")
    }
}


impl Config {
    // Read config file. Panic if any error occurs.
    pub fn from_file(file_path: &str) -> Config {
        let mut file = File::open(file_path)
            .unwrap_or_else(|err| {
                error!("Cannot open config file {}: {:?}", file_path, err);
                panic!("Cannot open config file {}: {:?}", file_path, err);
            });
        let mut config = String::new();
        file.read_to_string(&mut config)
            .unwrap_or_else(|err| {
                error!("Cannot read config file {}: {:?}.", file_path, err);
                panic!("Cannot read config file {}: {:?}.", file_path, err);
            });
        return serde_json::from_str(config.as_str()).unwrap_or_else(|err| {
            error!("Cannot decode config file: {:?}.", err);
            panic!("Cannot decode config file: {:?}.", err);
        });
    }
}