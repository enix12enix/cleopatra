// Integration tests for the execution API
// Assumes the server is already running

use reqwest;
use serde_json::json;

mod common {
    pub mod test_config;
}

#[tokio::test]
async fn test_create_execution() {
    let test_config = common::test_config::TestConfig::from_file().expect("Failed to load test config");
    let client = reqwest::Client::new();
    
    let request_body = json!({
        "name": "Test Execution",
        "tag": "integration-test",
        "created_by": "test-user",
        "time_created": 1234567890
    });
    
    let response = client
        .post(test_config.get_execution_api_url())
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 201);
    
    let execution: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    
    assert_eq!(execution["name"], "Test Execution");
    assert_eq!(execution["tag"], "integration-test");
    assert_eq!(execution["created_by"], "test-user");
    assert!(execution["id"].is_number());
}

#[tokio::test]
async fn test_get_executions() {
    let test_config = common::test_config::TestConfig::from_file().expect("Failed to load test config");
    let client = reqwest::Client::new();
    
    // First create an execution
    let create_request_body = json!({
        "name": "Test Execution",
        "tag": "integration-test",
        "created_by": "test-user",
        "time_created": 1234567890
    });
    
    let _create_response = client
        .post(test_config.get_execution_api_url())
        .json(&create_request_body)
        .send()
        .await
        .expect("Failed to send create request");
    
    // Then get the executions
    let response = client
        .get(test_config.get_executions_api_url())
        .send()
        .await
        .expect("Failed to send get request");
    
    assert_eq!(response.status(), 200);
    
    let executions: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    
    // We should have at least one execution (the one we just created)
    assert!(executions["items"].as_array().unwrap().len() >= 1);
    
    // Find the execution we just created
    let items = executions["items"].as_array().unwrap();
    let created_execution = items.iter().find(|item| {
        item["name"] == "Test Execution" && 
        item["tag"] == "integration-test" && 
        item["created_by"] == "test-user"
    });
    
    assert!(created_execution.is_some());
}

#[tokio::test]
async fn test_get_executions_with_filter() {
    let test_config = common::test_config::TestConfig::from_file().expect("Failed to load test config");
    let client = reqwest::Client::new();
    
    // Create two executions with different creators
    let create_request_body1 = json!({
        "name": "Test Execution 1",
        "tag": "integration-test",
        "created_by": "test-user-1",
        "time_created": 1234567890
    });
    
    let create_request_body2 = json!({
        "name": "Test Execution 2",
        "tag": "integration-test",
        "created_by": "test-user-2",
        "time_created": 1234567891
    });
    
    // Create first execution
    let _create_response1 = client
        .post(test_config.get_execution_api_url())
        .json(&create_request_body1)
        .send()
        .await
        .expect("Failed to send create request 1");
    
    // Create second execution
    let _create_response2 = client
        .post(test_config.get_execution_api_url())
        .json(&create_request_body2)
        .send()
        .await
        .expect("Failed to send create request 2");
    
    // Get executions filtered by creator
    let response = client
        .get(format!("{}?created_by=test-user-1", test_config.get_executions_api_url()))
        .send()
        .await
        .expect("Failed to send filtered get request");
    
    assert_eq!(response.status(), 200);
    
    let executions: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    
    // We should have exactly one execution matching the filter
    assert_eq!(executions["items"].as_array().unwrap().len(), 1);
    
    let first_item = &executions["items"][0];
    assert_eq!(first_item["created_by"], "test-user-1");
    assert_eq!(first_item["name"], "Test Execution 1");
}