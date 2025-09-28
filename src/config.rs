// src/config.rs
// Configuration handling module

use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub writers: HashMap<String, WriterConfig>,
    pub auth: AuthConfig,
    pub data_retention: HashMap<String, DataRetentionConfig>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    #[serde(default = "default_auth_enabled")]
    pub enabled: bool,
    pub secret_path: Option<String>,
    pub algorithm: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DataRetentionConfig {
    #[serde(default = "default_maintenance_enabled")]
    pub enabled: bool,
    #[serde(default = "default_data_retention_days")]
    pub period_in_day: u32,
    #[serde(default = "default_maintenance_cron")]
    pub cron: String,
}

fn default_wal() -> bool {
    true
}

fn default_wal_autocheckpoint() -> i32 {
    1000
}

fn default_auth_enabled() -> bool {
    false
}

fn default_maintenance_enabled() -> bool {
    false
}

fn default_data_retention_days() -> u32 {
    90
}

fn default_maintenance_cron() -> String {
    "0 0 3 * * Sun".to_string()
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
        let config_path = env::var("APP_CONFIG").unwrap_or_else(|_| format!("config/{}.toml", env));
        
        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        
        // Validate auth config if auth is enabled
        if config.auth.enabled && (config.auth.secret_path.is_none() || config.auth.algorithm.is_none()) {
            return Err("Auth is enabled but secret_path and algorithm are required".into());
        }
        
        Ok(config)
    }
}