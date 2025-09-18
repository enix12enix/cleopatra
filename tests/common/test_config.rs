// Test configuration loader
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct TestConfig {
    pub api_base_url: String,
}

impl TestConfig {
    pub fn from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = "tests/test_config.toml";
        let config_str = fs::read_to_string(config_path)?;
        let config: TestConfig = toml::from_str(&config_str)?;
        Ok(config)
    }
    
    pub fn get_execution_api_url(&self) -> String {
        format!("{}/api/execution", self.api_base_url)
    }
    
    pub fn get_executions_api_url(&self) -> String {
        format!("{}/api/executions", self.api_base_url)
    }
}