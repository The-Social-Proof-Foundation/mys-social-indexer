use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::Database;

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a success response with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    /// Create an error response with message
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Convert a result to an API response
pub fn result_to_response<T, E>(result: Result<T, E>) -> ApiResponse<T>
where
    E: std::fmt::Display,
{
    match result {
        Ok(data) => ApiResponse::success(data),
        Err(err) => ApiResponse::error(err.to_string()),
    }
}

/// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            limit: Some(10),
            offset: Some(0),
        }
    }
}

/// Standard pagination implementation
impl PaginationParams {
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(10).min(100)
    }
    
    pub fn offset(&self) -> usize {
        self.offset.unwrap_or(0)
    }
}