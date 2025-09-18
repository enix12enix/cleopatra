// Integration tests for the result API
// Assumes the server is already running

use reqwest;
use serde_json::json;

mod common {
    pub mod test_config;
}

#[tokio::test]
async fn test_create_result() {
    let test_config = common::test_config::TestConfig::from_file().expect("Failed to load test config");
    let client = reqwest::Client::new();
    
    // First, create an execution to associate the result with
    let create_execution_body = json!({
        "name": "Test Execution for Result",
        "tag": "result-test",
        "created_by": "test-user",
        "time_created": 1234567890
    });
    
    let execution_response = client
        .post(test_config.get_execution_api_url())
        .json(&create_execution_body)
        .send()
        .await
        .expect("Failed to create execution");
    
    assert_eq!(execution_response.status(), 201);
    
    let execution: serde_json::Value = execution_response.json().await.expect("Failed to parse execution JSON");
    let execution_id = execution["id"].as_i64().expect("Execution ID should be a number");
    
    // Now create a test result
    let create_result_body = json!({
        "execution_id": execution_id,
        "name": "test_login_functionality",
        "platform": "web",
        "description": "Test login with valid credentials",
        "status": "P",
        "execution_time": 1500,
        "log": "Login successful",
        "screenshot_id": 1001,
        "created_by": "test-user",
        "time_created": 1234567891
    });
    
    let response = client
        .post(test_config.get_result_api_url())
        .json(&create_result_body)
        .send()
        .await
        .expect("Failed to send create result request");
    
    assert_eq!(response.status(), 201);
    
    let result: serde_json::Value = response.json().await.expect("Failed to parse result JSON");
    
    assert_eq!(result["execution_id"], execution_id);
    assert_eq!(result["name"], "test_login_functionality");
    assert_eq!(result["platform"], "web");
    assert_eq!(result["status"], "P");
    assert_eq!(result["execution_time"], 1500);
    assert_eq!(result["counter"], 1); // First time, counter should be 1
    assert!(result["id"].is_number());
}

#[tokio::test]
async fn test_get_result_by_id() {
    let test_config = common::test_config::TestConfig::from_file().expect("Failed to load test config");
    let client = reqwest::Client::new();
    
    // First, create an execution
    let create_execution_body = json!({
        "name": "Test Execution for Get Result",
        "tag": "result-test",
        "created_by": "test-user",
        "time_created": 1234567890
    });
    
    let execution_response = client
        .post(test_config.get_execution_api_url())
        .json(&create_execution_body)
        .send()
        .await
        .expect("Failed to create execution");
    
    assert_eq!(execution_response.status(), 201);
    
    let execution: serde_json::Value = execution_response.json().await.expect("Failed to parse execution JSON");
    let execution_id = execution["id"].as_i64().expect("Execution ID should be a number");
    
    // Create a test result
    let create_result_body = json!({
        "execution_id": execution_id,
        "name": "test_get_result",
        "platform": "api",
        "description": "Test get result by ID",
        "status": "P",
        "execution_time": 800,
        "log": "Test passed",
        "screenshot_id": 1002,
        "created_by": "test-user",
        "time_created": 1234567892
    });
    
    let create_response = client
        .post(test_config.get_result_api_url())
        .json(&create_result_body)
        .send()
        .await
        .expect("Failed to create test result");
    
    assert_eq!(create_response.status(), 201);
    
    let created_result: serde_json::Value = create_response.json().await.expect("Failed to parse created result JSON");
    let result_id = created_result["id"].as_i64().expect("Result ID should be a number");
    
    // Now get the result by ID
    let response = client
        .get(test_config.get_result_by_id_api_url(result_id))
        .send()
        .await
        .expect("Failed to send get result request");
    
    assert_eq!(response.status(), 200);
    
    let result: serde_json::Value = response.json().await.expect("Failed to parse result JSON");
    
    assert_eq!(result["id"], result_id);
    assert_eq!(result["execution_id"], execution_id);
    assert_eq!(result["name"], "test_get_result");
    assert_eq!(result["platform"], "api");
    assert_eq!(result["status"], "P");
    assert_eq!(result["execution_time"], 800);
    assert_eq!(result["log"], "Test passed");
    assert_eq!(result["screenshot_id"], 1002);
    assert_eq!(result["created_by"], "test-user");
}

#[tokio::test]
async fn test_upsert_result() {
    let test_config = common::test_config::TestConfig::from_file().expect("Failed to load test config");
    let client = reqwest::Client::new();
    
    // First, create an execution
    let create_execution_body = json!({
        "name": "Test Execution for Upsert",
        "tag": "result-test",
        "created_by": "test-user",
        "time_created": 1234567890
    });
    
    let execution_response = client
        .post(test_config.get_execution_api_url())
        .json(&create_execution_body)
        .send()
        .await
        .expect("Failed to create execution");
    
    assert_eq!(execution_response.status(), 201);
    
    let execution: serde_json::Value = execution_response.json().await.expect("Failed to parse execution JSON");
    let execution_id = execution["id"].as_i64().expect("Execution ID should be a number");
    
    // Create a test result for the first time
    let create_result_body = json!({
        "execution_id": execution_id,
        "name": "test_upsert_functionality",
        "platform": "web",
        "description": "Test upsert functionality",
        "status": "P",
        "execution_time": 1200,
        "log": "First run",
        "screenshot_id": 1003,
        "created_by": "test-user",
        "time_created": 1234567893
    });
    
    let first_response = client
        .post(test_config.get_result_api_url())
        .json(&create_result_body)
        .send()
        .await
        .expect("Failed to create test result first time");
    
    assert_eq!(first_response.status(), 201);
    
    let first_result: serde_json::Value = first_response.json().await.expect("Failed to parse first result JSON");
    assert_eq!(first_result["counter"], 1); // First time, counter should be 1
    
    // Create the same test result again (should update, not insert)
    let update_result_body = json!({
        "execution_id": execution_id,
        "name": "test_upsert_functionality",
        "platform": "web",
        "description": "Test upsert functionality - updated",
        "status": "F",
        "execution_time": 1500,
        "log": "Second run - failed",
        "screenshot_id": 1004,
        "created_by": "test-user",
        "time_created": 1234567894
    });
    
    let second_response = client
        .post(test_config.get_result_api_url())
        .json(&update_result_body)
        .send()
        .await
        .expect("Failed to create test result second time");
    
    assert_eq!(second_response.status(), 201);
    
    let second_result: serde_json::Value = second_response.json().await.expect("Failed to parse second result JSON");
    
    // Should be the same ID as the first result
    assert_eq!(second_result["id"], first_result["id"]);
    // Counter should be incremented
    assert_eq!(second_result["counter"], 2);
    // Other fields should be updated
    assert_eq!(second_result["description"], "Test upsert functionality - updated");
    assert_eq!(second_result["status"], "F");
    assert_eq!(second_result["execution_time"], 1500);
    assert_eq!(second_result["log"], "Second run - failed");
    assert_eq!(second_result["screenshot_id"], 1004);
}