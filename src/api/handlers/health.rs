use axum::{extract::State, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::Database;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
    database_connected: bool,
}

/// Health check endpoint
pub async fn health_check(State(db): State<Arc<Database>>) -> Json<HealthResponse> {
    // Check database connection
    let db_connected = db.get_connection().await.is_ok();
    
    // Return health status
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database_connected: db_connected,
    })
}