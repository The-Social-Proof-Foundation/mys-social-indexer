// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

pub mod routes;
pub mod handlers;

use axum::{
    routing::{get},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

use std::sync::Arc;
use crate::db::Database;
use crate::config::Config;

/// Setup the API server
pub async fn setup_api_server(config: &Config, db: Arc<Database>) -> anyhow::Result<()> {
    let app = create_router(db);
    
    // Create socket address
    let addr = SocketAddr::new(
        config.server.host.parse()?,
        config.server.port,
    );
    
    // Start server
    tracing::info!("Starting API server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Create the API router
fn create_router(db: Arc<Database>) -> Router {
    // Get a clone of the unwrapped pool for API handlers
    let pool = db.pool.as_ref().clone();
    
    Router::new()
        // Health routes
        .route("/health", get(handlers::health::health_check))
        
        // Profile routes
        .route("/profiles", get(handlers::profiles::get_profiles))
        .route("/profiles/:address", get(handlers::profiles::get_profile_by_address))
        .route("/profiles/username/:username", get(handlers::profiles::get_profile_by_username))
        
        // Add shared state
        .with_state(pool)
        
        // Add tracing
        .layer(TraceLayer::new_for_http())
}