// src/models.rs
// Define models here

use std::ops::Deref;

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
    pub status: Status,
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
pub struct CreateTestResultBase {
    pub name: String,
    pub platform: String,
    pub description: Option<String>,
    pub status: Status,
    pub execution_time: Option<i64>,
    pub log: Option<String>,
    pub screenshot_id: Option<i64>,
    pub created_by: Option<String>,
    pub time_created: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTestResult {
    pub execution_id: i64,
    #[serde(flatten)]
    pub base: CreateTestResultBase,
}

impl CreateTestResult {
   pub fn from_json(
        json: &str,
        execution_id: i64,
    ) -> Result<Self, serde_json::Error> {
        let base: CreateTestResultBase = serde_json::from_str(json)?;
        Ok(CreateTestResult { execution_id, base })
    }
}

impl Deref for CreateTestResult {
    type Target = CreateTestResultBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTestResultResponse {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamResponse {
    pub status: String,
    pub execution_id: i64,
    pub received: i64,
    pub inserted: i64,
    pub failed: i64,
    pub failed_items: Option<Vec<FailedItem>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailedItem {
    pub error: String,
    pub raw_payload: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub roles: Vec<String>, // user roles
    pub exp: usize, // expiration timestamp
}

// for extracting auth user from JWT, not used now
// pub struct AuthUser(pub Claims);

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
#[sqlx(type_name = "TEXT")]
pub enum Status {
    P, // passed
    F, // failed
    I, // ignored
}