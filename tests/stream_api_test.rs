// Integration tests for the stream API
// Assumes the server is already running

mod common;

#[tokio::test]
async fn test_stream_results() {
    let create_execution_body = r#"{
        "name": "Test Execution for Stream",
        "tag": "stream-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;

    let execution = common::helper::create_execution(create_execution_body)
        .await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");

    let execution_id = execution.id.expect("Execution ID should be a number");

    let test_results: Vec<&str> = vec![
        r#"{"name":"test_login_functionality","platform":"web","description":"Test login with valid credentials","status":"P","execution_time":1500,"log":"Login successful","screenshot_id":1001,"created_by":"test-user","time_created":1234567891}"#,
        r#"{"name":"test_signup_functionality","platform":"web","description":"Test signup flow","status":"F","execution_time":2300,"log":"Signup failed","screenshot_id":1002,"created_by":"test-user","time_created":1234567892}"#,
        r#"{"name":"test_password_reset","platform":"api","description":"Test password reset functionality","status":"P","execution_time":1800,"log":"Password reset successful","screenshot_id":1003,"created_by":"test-user","time_created":1234567893}"#,
    ];

    let stream_response = common::helper::stream_create_results(execution_id, test_results)
        .await
        .expect("Failed to send stream request")
        .expect("Expected stream response");

    assert_eq!(stream_response.status, "C");
    assert_eq!(stream_response.execution_id, execution_id);
    assert_eq!(stream_response.received, 3);
    assert_eq!(stream_response.inserted, 3);
    assert_eq!(stream_response.failed, 0);
    assert!(stream_response.failed_items.is_none());
}

#[tokio::test]
async fn test_stream_results_with_invalid_data() {
    // First, create an execution to associate the results with
    let create_execution_body = r#"{
        "name": "Test Execution for Stream Invalid Data",
        "tag": "stream-test",
        "created_by": "test-user",
        "time_created": 1234567890
    }"#;

    let execution = common::helper::create_execution(create_execution_body)
        .await
        .expect("Failed to create execution")
        .expect("Expected execution to be created");

    let execution_id = execution.id.expect("Execution ID should be a number");

    let invalid_result =  r#"{"name":"invalid_test","platform":"api","description":"Test password reset functionality","status":"X","execution_time":1800,"log":"Password reset successful","screenshot_id":1003,"created_by":"test-user","time_created":1234567893}"#;

    // Prepare test result data with one invalid entry (invalid status)
   let test_results: Vec<&str> = vec![
    r#"{"name":"test_login_functionality","platform":"web","description":"Test login with valid credentials","status":"P","execution_time":1500,"log":"Login successful","screenshot_id":1001,"created_by":"test-user","time_created":1234567891}"#,
    r#"{"name":"test_signup_functionality","platform":"web","description":"Test signup flow","status":"F","execution_time":2300,"log":"Signup failed","screenshot_id":1002,"created_by":"test-user","time_created":1234567892}"#,
    invalid_result,
   ];
    // Send the stream request using the helper function
    let stream_response = common::helper::stream_create_results(execution_id, test_results)
        .await
        .expect("Failed to send stream request")
        .expect("Expected stream response");

    // Verify the response - should have partial success
    assert_eq!(stream_response.status, "P"); // Partial success
    assert_eq!(stream_response.execution_id, execution_id);
    assert_eq!(stream_response.received, 3);
    assert_eq!(stream_response.inserted, 2); // Only 2 valid items should be processed
    assert_eq!(stream_response.failed, 1); // 1 item should fail

    // Verify the failed item details
    let failed_items = stream_response.failed_items.unwrap();
    assert_eq!(failed_items.len(), 1);
    assert!(failed_items[0].error.contains("unknown variant `X`"));
    assert_eq!(<std::option::Option<std::string::String> as Clone>::clone(&failed_items[0].raw_payload).unwrap(),  invalid_result);
}
