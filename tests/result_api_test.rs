// Integration tests for the result API
// Assumes the server is already running
mod common;

use cleopatra::models::Status;

#[tokio::test]
async fn test_create_result() {
    let create_execution_json = r#"{
        "name": "Test Execution for Result",
        "tag": "result-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;
    
    let execution = common::helper::create_execution(create_execution_json).await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");
    
    let execution_id = execution.id.expect("Execution should have an ID");
    
    let create_result_json = format!(r#"{{
        "execution_id": {},
        "name": "test_login_functionality",
        "platform": "web",
        "description": "Test login with valid credentials",
        "status": "P",
        "execution_time": 1500,
        "log": "Login successful",
        "screenshot_id": 1001,
        "created_by": "test-user",
        "time_created": 1234567891
    }}"#, execution_id);
    
    let result_response = common::helper::create_result(&create_result_json).await
        .expect("Failed to create result")
        .expect("Expected result response to be created");
    
    assert_eq!(result_response.status, "delivered");
}

#[tokio::test]
async fn test_get_result_by_id() {
    let create_execution_body = r#"{
        "name": "Test Execution for Get Result",
        "tag": "result-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;
    
    let execution = common::helper::create_execution(create_execution_body).await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");
    
    let execution_id = execution.id.expect("Execution should have an ID");
    
    let create_result_json = format!(r#"{{
        "execution_id": {},
        "name": "test_get_result",
        "platform": "api",
        "description": "Test get result by ID",
        "status": "P",
        "execution_time": 800,
        "log": "Test passed",
        "screenshot_id": 1002,
        "created_by": "test-user",
        "time_created": 1234567892
    }}"#, execution_id);
    
    common::helper::create_result(&create_result_json).await
        .expect("Failed to create test result")
        .expect("Expected test result to be created");

    common::helper::wait();
    
    let results = common::helper::get_results(execution_id).await
        .expect("Failed to get results by execution ID")
        .expect("Expected results to be found");
    
    assert_eq!(results.len(), 1);
    
    let result = &results[0];
    assert_eq!(result.execution_id, execution_id);
    assert_eq!(result.name, "test_get_result");
    assert_eq!(result.platform, "api");
    assert_eq!(result.description.as_ref().unwrap(), "Test get result by ID");
    assert_eq!(result.status, Status::P);
    assert_eq!(result.execution_time, Some(800));
    assert_eq!(result.log.as_ref().unwrap(), "Test passed");
    assert_eq!(result.screenshot_id, Some(1002));
    assert_eq!(result.created_by.as_ref().unwrap(), "test-user");
}

#[tokio::test]
async fn test_upsert_result() {
    let create_execution_body = r#"{
        "name": "Test Execution for Upsert",
        "tag": "result-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;
    
    let execution = common::helper::create_execution(create_execution_body).await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");
    
    let execution_id = execution.id.expect("Execution should have an ID");
    
    let create_result_json = format!(r#"{{
        "execution_id": {},
        "name": "test_upsert_functionality",
        "platform": "web",
        "description": "Test failed",
        "status": "F",
        "execution_time": 1000,
        "log": "First run",
        "screenshot_id": 1003,
        "created_by": "test-user",
        "time_created": 1234567893
    }}"#, execution_id);
    
    common::helper::create_result(&create_result_json).await
        .expect("Failed to create test result first time")
        .expect("Expected first test result to be created");

    common::helper::wait();

    let update_result_json = format!(r#"{{
        "execution_id": {},
        "name": "test_upsert_functionality",
        "platform": "web",
        "description": "Test passed",
        "status": "P",
        "execution_time": 1200,
        "log": "Second run",
        "created_by": "test-user",
        "time_created": 12345678999
    }}"#, execution_id);

    common::helper::create_result(&update_result_json).await
        .expect("Failed to update test result")
        .expect("Expected second test result to be update");

    common::helper::wait();
    
    let results = common::helper::get_results(execution_id).await
        .expect("Failed to get results by execution ID")
        .expect("Expected results to be found");
    
    assert_eq!(results.len(), 1);
    
    let result = &results[0];
    assert_eq!(result.execution_id, execution_id);
    assert_eq!(result.name, "test_upsert_functionality");
    assert_eq!(result.platform, "web");
    assert_eq!(result.description.as_ref().unwrap(), "Test passed");
    assert_eq!(result.status, Status::P);
    assert_eq!(result.execution_time, Some(1200));
    assert_eq!(result.log.as_ref().unwrap(), "Second run");
    assert!(result.screenshot_id.is_none());
    assert_eq!(result.created_by.as_ref().unwrap(), "test-user");
}

#[tokio::test]
async fn test_update_result_status() {
    let create_execution_json = r#"{
        "name": "Test Execution for Update Status",
        "tag": "status-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;
    
    let execution = common::helper::create_execution(create_execution_json).await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");
    
    let execution_id = execution.id.expect("Execution should have an ID");
    
    let create_result_json = format!(r#"{{
        "execution_id": {},
        "name": "test_update_status",
        "platform": "api",
        "description": "Initial status test",
        "status": "P",
        "execution_time": 1000,
        "log": "Initial run",
        "created_by": "test-user",
        "time_created": 1234567891
    }}"#, execution_id);
    
    common::helper::create_result(&create_result_json).await
        .expect("Failed to create test result")
        .expect("Expected test result to be created");
    
    common::helper::wait();
    
    let results = common::helper::get_results(execution_id).await
        .expect("Failed to get results by execution ID")
        .expect("Expected results to be found");
    
    assert_eq!(results.len(), 1);
    let initial_result = &results[0];
    assert_eq!(initial_result.status, Status::P);
    let result_id = initial_result.id.expect("Result should have an ID");
    
    common::helper::update_test_result(result_id, "F".to_string()).await
        .expect("Failed to update test result status");
    
    common::helper::wait();
    
    let updated_result = common::helper::get_result(result_id).await
        .expect("Failed to get updated result")
        .expect("Expected result to be found");
    
    assert_eq!(updated_result.status, Status::F);
}