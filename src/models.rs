// src/models.rs
// Define models here

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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