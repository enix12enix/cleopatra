// src/config.rs
// Configuration handling module

use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub writer: WriterConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    #[serde(default = "default_wal")]
    pub wal: bool,
    #[serde(default = "default_wal_autocheckpoint")]
    pub wal_autocheckpoint: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WriterConfig {
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

fn default_wal() -> bool {
    true
}

fn default_wal_autocheckpoint() -> i32 {
    1000
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
        let config_path = format!("config/{}.toml", env);
        
        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        
        Ok(config)
    }
}