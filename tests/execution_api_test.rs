// Integration tests for the execution API
// Assumes the server is already running

mod common;

use std::collections::HashMap;

use fake::Fake;
use fake::faker::lorem::en::Sentence;
use fake::faker::name::en::Name;

#[tokio::test]
async fn test_create_execution() {
    let execution_json = r#"{
        "name": "Test Execution",
        "tag": "integration-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;

    let execution = common::helper::create_execution(execution_json)
        .await
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

    common::helper::create_execution(create_execution_json)
        .await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");

    let executions_response = common::helper::get_executions()
        .await
        .expect("Failed to get executions");

    assert!(executions_response.total >= 1);
    assert!(!executions_response.items.is_empty());

    let found_execution = executions_response.items.iter().find(|item| {
        item.name == "Test Execution"
            && item.tag == Some("integration-test".to_string())
            && item.created_by == Some("test-user".to_string())
    });

    assert!(
        found_execution.is_some(),
        "Created execution not found in the list of executions"
    );
    let found_execution = found_execution.unwrap();

    assert_eq!(found_execution.name, "Test Execution");
    assert_eq!(found_execution.tag, Some("integration-test".to_string()));
    assert_eq!(found_execution.created_by, Some("test-user".to_string()));
    assert!(found_execution.id.is_some());
}

#[tokio::test]
async fn test_get_executions_with_filter() {
    // Create two executions with different creators
    let created_by_1: String = Name().fake();
    let create_execution_json1 = format!(
        r#"{{
            "name": "Test Execution 1",
            "tag": "integration-test",
            "created_by": "{}",
            "time_created": 1234567890
        }}"#,
        created_by_1
    );

    common::helper::create_execution(&create_execution_json1)
            .await
            .expect(&format!("Failed to create execution {}", create_execution_json1))
            .expect(&format!("Expected execution {} to be created", create_execution_json1));

    let created_by_2: String = Name().fake();
    for i in 0..21 {
        let name: String = Sentence(2..8).fake();

        let create_execution_json = format!(
            r#"{{
                "name": "{}",
                "tag": "api-test",
                "created_by": "{}",
                "time_created": {}
            }}"#,
            name,
            created_by_2,
            1234567890 + i as u64
        );

        common::helper::create_execution(&create_execution_json)
            .await
            .expect(&format!("Failed to create execution {}", i + 1))
            .expect(&format!("Expected execution {} to be created", i + 1));
    }

    // get executions created by created_by_1
    let mut filters = HashMap::new();
    filters.insert("created_by".to_string(), created_by_1);

    let filtered_executions = common::helper::get_executions_with_filters(&filters)
        .await
        .expect("Failed to get filtered executions");

    assert_eq!(filtered_executions.items.len(), 1);
    assert_eq!(filtered_executions.total, 1);

    let filtered_execution = &filtered_executions.items[0];
    assert_eq!(filtered_execution.name, "Test Execution 1");
    assert_eq!(filtered_execution.tag, Some("integration-test".to_string()));


    // get executions created by created_by_2
    filters.insert("created_by".to_string(), created_by_2);
    filters.insert("limit".to_string(), "10".to_string());
    let filtered_executions2 = common::helper::get_executions_with_filters(&filters)
        .await
        .expect("Failed to get filtered executions");
    assert_eq!(filtered_executions2.items.len(), 10);
    assert_eq!(filtered_executions2.total, 21);
}

#[tokio::test]
async fn test_get_suggested_executions() {
    let execution_names = vec![
        "login_test",
        "login_validation",
        "logout_test",
        "longer_prefix_test",
        "other_execution",
    ];

    for name in execution_names {
        let create_execution_json = format!(r#"{{
            "name": "{}",
            "tag": "suggest-test",
            "created_by": "test-user",
            "time_created": 1234567890
        }}"#, name);

        common::helper::create_execution(&create_execution_json)
            .await
            .expect("Failed to create execution")
            .expect("Expected execution to be created");
    }

    common::helper::wait();

    let suggest_response = common::helper::get_executions_suggest("log")
        .await
        .expect("Failed to get execution suggestions")
        .expect("Expected suggestions response");

    assert_eq!(suggest_response.query, "log");
    assert_eq!(suggest_response.limit, 5);
    
    assert!(!suggest_response.suggestions.is_empty());
    
    assert!(!suggest_response.suggestions.is_empty());
    
    for suggestion in &suggest_response.suggestions {
        let name = suggestion.name.to_lowercase();
        assert!(name.starts_with(&"log".to_lowercase()));
    }

    let empty_suggest_response = common::helper::get_executions_suggest("")
        .await
        .expect("Failed to get execution suggestions for empty query")
        .expect("Expected suggestions response");

    assert_eq!(empty_suggest_response.query, "");
    
    assert!(empty_suggest_response.suggestions.is_empty());
}
