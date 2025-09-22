// Integration tests for the execution API
// Assumes the server is already running

mod common {
    pub mod test_config;
    pub mod helper;
}

#[tokio::test]
async fn test_create_execution() {
    let execution_json = r#"{
        "name": "Test Execution",
        "tag": "integration-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;
    
    let execution = common::helper::create_execution(execution_json).await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");
    
    assert_eq!(execution.name, "Test Execution");
    assert_eq!(execution.tag, Some("integration-test".to_string()));
    assert_eq!(execution.created_by, Some("test-user".to_string()));
    assert!(execution.id.is_some());
}

#[tokio::test]
async fn test_get_executions() {
    // First create an execution
    let create_execution_json = r#"{
        "name": "Test Execution",
        "tag": "integration-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;
    
    common::helper::create_execution(create_execution_json).await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");
    
    // Then get the executions
    let executions_response = common::helper::get_executions().await
        .expect("Failed to get executions");
    
    // Verify that we got a successful response
    assert!(executions_response.total >= 1);
    assert!(!executions_response.items.is_empty());
    
    // Find the execution we just created
    let found_execution = executions_response.items.iter().find(|item| {
        item.name == "Test Execution" && 
        item.tag == Some("integration-test".to_string()) && 
        item.created_by == Some("test-user".to_string())
    });
    
    // Verify that the created execution is in the list
    assert!(found_execution.is_some(), "Created execution not found in the list of executions");
    let found_execution = found_execution.unwrap();
    
    // Verify the execution details
    assert_eq!(found_execution.name, "Test Execution");
    assert_eq!(found_execution.tag, Some("integration-test".to_string()));
    assert_eq!(found_execution.created_by, Some("test-user".to_string()));
    assert!(found_execution.id.is_some());
}

#[tokio::test]
async fn test_get_executions_with_filter() {
    // Create two executions with different creators
    let create_execution_json1 = r#"{
        "name": "Test Execution 1",
        "tag": "integration-test",
        "created_by": "test-user-1",
        "time_created": 1234567890
    }"#;
    
    let create_execution_json2 = r#"{
        "name": "Test Execution 2",
        "tag": "integration-test",
        "created_by": "test-user-2",
        "time_created": 1234567891
    }"#;
    
    // Create first execution
    let _created_execution1 = common::helper::create_execution(create_execution_json1).await
        .expect("Failed to create execution 1")
        .expect("Expected execution 1 to be created");
    
    // Create second execution
    let _created_execution2 = common::helper::create_execution(create_execution_json2).await
        .expect("Failed to create execution 2")
        .expect("Expected execution 2 to be created");
    
    // Get executions filtered by creator
    let mut filters = std::collections::HashMap::new();
    filters.insert("created_by".to_string(), "test-user-1".to_string());
    
    let filtered_executions = common::helper::get_executions_with_filters(filters).await
        .expect("Failed to get filtered executions");
    
    // Verify that we got exactly one execution matching the filter
    assert_eq!(filtered_executions.items.len(), 1);
    assert_eq!(filtered_executions.total, 1);
    
    // Verify the filtered execution details
    let filtered_execution = &filtered_executions.items[0];
    assert_eq!(filtered_execution.name, "Test Execution 1");
    assert_eq!(filtered_execution.created_by, Some("test-user-1".to_string()));
    assert_eq!(filtered_execution.tag, Some("integration-test".to_string()));
}