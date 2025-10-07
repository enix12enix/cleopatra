// src/lib.rs
// Library crate for the Cleopatra application exposed to tests

pub mod models;

pub use models::{Execution, TestResult, CreateTestResultResponse, StreamResponse, ExecutionListResponse, Status, SuggestedItem};