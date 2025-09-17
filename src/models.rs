// src/models.rs
// Define models here

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use crate::writer::Writer;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Execution {
    pub id: Option<i64>,
    pub name: String,
    pub tag: Option<String>,
    pub created_by: Option<String>,
    pub time_created: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TestResult {
    pub id: Option<i64>,
    pub execution_id: i64,
    pub name: String,
    pub platform: String,
    pub description: Option<String>,
    pub status: String,
    pub execution_time: Option<i64>,
    pub counter: i64,
    pub log: Option<String>,
    pub screenshot_id: Option<i64>,
    pub created_by: Option<String>,
    pub time_created: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateExecution {
    pub name: String,
    pub tag: Option<String>,
    pub created_by: Option<String>,
    pub time_created: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionListResponse {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_next: bool,
    pub items: Vec<Execution>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionResultsResponse {
    pub execution_id: i64,
    pub summary: Option<Summary>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_next: bool,
    pub items: Vec<TestResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total: i64,
    pub pass: i64,
    pub fail: i64,
    pub ignor: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTestResult {
    pub execution_id: i64,
    pub name: String,
    pub platform: String,
    pub description: Option<String>,
    pub status: String,
    pub execution_time: Option<i64>,
    pub log: Option<String>,
    pub screenshot_id: Option<i64>,
    pub created_by: Option<String>,
    pub time_created: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamResponse {
    pub status: String,
    pub message: String,
    pub execution_id: i64,
    pub received: i64,
    pub inserted: i64,
    pub failed: i64,
    pub failed_items: Option<Vec<FailedItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailedItem {
    pub test_name: String,
    pub error: String,
    pub raw_payload: serde_json::Value,
}

/// Application state that holds shared resources
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub writer: Writer,
}