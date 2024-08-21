use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, io::BufReader, path::Path};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_id: i32,
    pub api_hash: String,
    pub phone: String,
    pub user_agent: Option<TelegramUserAgent>,
    pub server: Option<ApiServer>,
    pub db_path: String,
    pub files_path: String,
    pub enc_key: String,
    pub access_key: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TelegramUserAgent {
    pub language: String,
    pub device: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApiServer {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader)?;
        Ok(u)
    }
}
