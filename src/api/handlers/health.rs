// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use crate::db::DbPool;

/// Health check endpoint
pub async fn health_check(State(db_pool): State<DbPool>) -> impl IntoResponse {
    // Check database connection
    match db_pool.get().await {
        Ok(_) => {
            // Database connection is successful
            (
                StatusCode::OK,
                Json(json!({
                    "status": "healthy",
                    "message": "API server is running"
                }))
            )
        },
        Err(e) => {
            // Database connection failed
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unhealthy",
                    "message": format!("Database connection failed: {}", e)
                }))
            )
        }
    }
}