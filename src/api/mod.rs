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
        .route("/recent-profiles", get(handlers::profiles::latest_profiles))
        .route("/profile/:address", get(handlers::profiles::get_profile_by_address))
        .route("/profile/username/:username", get(handlers::profiles::get_profile_by_username))
        
        // Social graph routes
        .route("/profile/following/:profile_id", get(handlers::social_graph::get_following))
        .route("/profile/followers/:profile_id", get(handlers::social_graph::get_followers))
        .route("/profile/is-following/:follower_profile_id/:following_profile_id", get(handlers::social_graph::check_following))
        .route("/profile/stats/:profile_id", get(handlers::social_graph::get_follow_stats))
        
        // Profile blocking routes
        .route("/profile/blocked/:profile_id", get(handlers::blocking::get_blocked_profiles))
        .route("/profile/is-blocked/:blocker_profile_id/:blocked_profile_id", get(handlers::blocking::check_profile_blocked))
        
        // Profile events routes
        .route("/profile-events/:profile_id", get(handlers::profile_events::get_profile_events))
        .route("/profile-events/:profile_id/platforms", get(handlers::profile_events::get_platform_memberships))
        .route("/profile-events/:profile_id/blocking", get(handlers::profile_events::get_blocking_history))
        
        // Platform routes
        .route("/platforms", get(handlers::platforms::get_platforms))
        .route("/platforms/approved", get(handlers::platforms::get_approved_platforms))
        .route("/platform/:platform_id", get(handlers::platforms::get_platform_by_id))
        .route("/platform/:platform_id/approval", get(handlers::platforms::get_platform_approval_status))
        .route("/platform/:platform_id/moderators", get(handlers::platforms::get_platform_moderators))
        .route("/platform/:platform_id/blocked", get(handlers::platforms::get_platform_blocked_profiles))
        
        // Platform blocking routes
        .route("/platforms/blocked-by/:profile_id", get(handlers::blocking::get_blocked_platforms))
        .route("/platform/is-blocked/:profile_id/:platform_id", get(handlers::blocking::check_platform_blocked))

        // Add shared state
        .with_state(pool)
        
        // Add tracing
        .layer(TraceLayer::new_for_http())
}