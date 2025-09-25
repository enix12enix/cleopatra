// Test helper functions
use std::{collections::HashMap, thread};
use std::time::Duration;
use reqwest;
use serde_json::Value;
use cleopatra::models::{Execution, TestResult, CreateTestResultResponse, StreamResponse, ExecutionListResponse};
use anyhow::Result;

/// Get test results for a given execution ID by calling the API
/// Returns a vector of result objects, or None if no results are found
#[allow(dead_code)]
pub async fn get_results(execution_id: i64) -> Result<Option<Vec<TestResult>>> {
    let config = crate::common::test_config::TestConfig::from_file()
        .map_err(|e| anyhow::anyhow!("Failed to load test config: {}", e))?;
    let url = config.get_execution_result_api_url(execution_id);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await?;
    
    let status = response.status();
    if status.is_success() {
        let json: Value = response.json().await?;
        
        if let Some(items) = json["items"].as_array() {
            let results: Vec<TestResult> = items
                .iter()
                .filter_map(|item| {
                    serde_json::from_value(item.clone()).ok()
                })
                .collect();
            
            if results.is_empty() {
                Ok(None)
            } else {
                Ok(Some(results))
            }
        } else {
            Ok(None)
        }
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("API request failed with status {}: {}", status, error_text)
    }
}

/// Get a specific test result by its ID by calling the API
/// Returns the test result object, or None if no result is found
#[allow(dead_code)]
pub async fn get_result(result_id: i64) -> Result<Option<TestResult>> {
    let config = crate::common::test_config::TestConfig::from_file()
        .map_err(|e| anyhow::anyhow!("Failed to load test config: {}", e))?;
    
    // Make the HTTP request
    let client = reqwest::Client::new();
    let response = client
        .get(&config.get_result_by_id_api_url(result_id))
        .send()
        .await?;
    
    let status = response.status();
    if status.is_success() {
        let json: Value = response.json().await?;
        let result: TestResult = serde_json::from_value(json)?;
        Ok(Some(result))
    } else if status == reqwest::StatusCode::NOT_FOUND {
        Ok(None)
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("API request failed with status {}: {}", status, error_text)
    }
}

#[allow(dead_code)]
pub async fn create_execution(execution_json: &str) -> Result<Option<Execution>> {
    let config = crate::common::test_config::TestConfig::from_file()
        .map_err(|e| anyhow::anyhow!("Failed to load test config: {}", e))?;
    let _parsed: serde_json::Value = serde_json::from_str(execution_json)?;
    
    let client = reqwest::Client::new();
    let response = client
        .post(&config.get_execution_api_url())
        .header("Content-Type", "application/json")
        .body(execution_json.to_string())
        .send()
        .await?;
    
    let status = response.status();
    if status.is_success() {
        let json: Value = response.json().await?;
        let execution: Execution = serde_json::from_value(json)?;
        Ok(Some(execution))
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("API request failed with status {}: {}", status, error_text)
    }
}

/// Get executions by calling the API
/// Returns a list of executions
#[allow(dead_code)]
pub async fn get_executions() -> Result<ExecutionListResponse> {
    let config = crate::common::test_config::TestConfig::from_file()
        .map_err(|e| anyhow::anyhow!("Failed to load test config: {}", e))?;
    
    let client = reqwest::Client::new();
    let response = client
        .get(&config.get_executions_api_url())
        .send()
        .await?;
    
    let status = response.status();
    if status.is_success() {
        let json: Value = response.json().await?;
        let executions: ExecutionListResponse = serde_json::from_value(json)?;
        Ok(executions)
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("API request failed with status {}: {}", status, error_text)
    }
}

/// Get executions by calling the API with filters
/// Takes a hashmap of filters to apply to the request
/// Returns a list of executions
#[allow(dead_code)]
pub async fn get_executions_with_filters(filters: &HashMap<String, String>) -> Result<ExecutionListResponse> {
    let config = crate::common::test_config::TestConfig::from_file()
        .map_err(|e| anyhow::anyhow!("Failed to load test config: {}", e))?;
    
    // Build the URL with query parameters
    let base_url = config.get_executions_api_url();
    let mut url = base_url;
    
    if !filters.is_empty() {
        let query_params: Vec<String> = filters
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect();
        
        url.push('?');
        url.push_str(&query_params.join("&"));
    }
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await?;
    
    let status = response.status();
    if status.is_success() {
        let json: Value = response.json().await?;
        let executions: ExecutionListResponse = serde_json::from_value(json)?;
        Ok(executions)
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("API request failed with status {}: {}", status, error_text)
    }
}

#[allow(dead_code)]
pub async fn create_result(request_json: &str) -> Result<Option<CreateTestResultResponse>> {
    let config = crate::common::test_config::TestConfig::from_file()
        .map_err(|e| anyhow::anyhow!("Failed to load test config: {}", e))?;
    let _parsed: serde_json::Value = serde_json::from_str(request_json)?;
    
    let client = reqwest::Client::new();
    let response = client
        .post(&config.get_result_api_url())
        .header("Content-Type", "application/json")
        .body(request_json.to_string())
        .send()
        .await?;
    
    let status = response.status();
    if status.is_success() {
        let json: Value = response.json().await?;
        let result_response: CreateTestResultResponse = serde_json::from_value(json)?;
        Ok(Some(result_response))
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("API request failed with status {}: {}", status, error_text)
    }
}

/// Create multiple test results by calling the stream API
/// Takes an execution ID and a vector of JSON strings representing the test results to create
/// Returns the stream response, or None if creation failed
#[allow(dead_code)]
pub async fn stream_create_results(execution_id: i64, results: Vec<&str>) -> Result<Option<StreamResponse>> {
    let config = crate::common::test_config::TestConfig::from_file()
        .map_err(|e| anyhow::anyhow!("Failed to load test config: {}", e))?;
    
    // Join the JSON strings with newlines to create NDJSON format
    let ndjson_body = results.join("\n");
    
    // Make the HTTP request
    let client = reqwest::Client::new();
    let response = client
        .post(&config.get_stream_api_url(execution_id))
        .header("Content-Type", "application/x-ndjson")
        .body(ndjson_body)
        .send()
        .await?;
    
    let status = response.status();
    if status.is_success() {
        let json: Value = response.json().await?;
        let stream_response: StreamResponse = serde_json::from_value(json)?;
        Ok(Some(stream_response))
    } else {
        let error_text = response.text().await?;
        anyhow::bail!("API request failed with status {}: {}", status, error_text)
    }
}

#[allow(dead_code)]
pub fn wait() {
    thread::sleep(Duration::from_secs(3));
}