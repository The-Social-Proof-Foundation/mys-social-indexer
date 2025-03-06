mod routes;
mod handlers;

use crate::config::Config;
use crate::db::Database;
use anyhow::Result;
use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, error};

/// Start the API server
pub async fn start_api_server(db: Arc<Database>) -> Result<()> {
    let config = Config::get();
    
    // Set up CORS
    let cors = if config.api.enable_cors {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::permissive()
    };
    
    // Create router with all routes
    let app = Router::new()
        // General routes
        .route("/health", get(handlers::health::health_check))
        .route("/metrics", get(handlers::metrics::get_metrics))
        
        // Profile routes
        .route("/api/profiles", get(handlers::profiles::get_profiles))
        .route("/api/profiles/:id", get(handlers::profiles::get_profile))
        .route("/api/profiles/:id/following", get(handlers::profiles::get_profile_following))
        .route("/api/profiles/:id/followers", get(handlers::profiles::get_profile_followers))
        .route("/api/profiles/:id/content", get(handlers::profiles::get_profile_content))
        .route("/api/profiles/:id/platforms", get(handlers::profiles::get_profile_platforms))
        
        // Platform routes
        .route("/api/platforms", get(handlers::platforms::get_platforms))
        .route("/api/platforms/:id", get(handlers::platforms::get_platform))
        .route("/api/platforms/:id/users", get(handlers::platforms::get_platform_users))
        .route("/api/platforms/:id/content", get(handlers::platforms::get_platform_content))
        .route("/api/platforms/:id/stats", get(handlers::platforms::get_platform_stats))
        
        // Content routes
        .route("/api/content", get(handlers::content::get_content))
        .route("/api/content/:id", get(handlers::content::get_content_details))
        .route("/api/content/:id/interactions", get(handlers::content::get_content_interactions))
        .route("/api/content/trending", get(handlers::content::get_trending_content))
        
        // IP routes
        .route("/api/ip", get(handlers::intellectual_property::get_ip_assets))
        .route("/api/ip/:id", get(handlers::intellectual_property::get_ip_details))
        .route("/api/ip/:id/licenses", get(handlers::intellectual_property::get_ip_licenses))
        
        // Fee routes
        .route("/api/fees/models", get(handlers::fees::get_fee_models))
        .route("/api/fees/models/:id", get(handlers::fees::get_fee_model_details))
        .route("/api/fees/recipients", get(handlers::fees::get_fee_recipients))
        .route("/api/fees/distributions", get(handlers::fees::get_fee_distributions))
        
        // Statistics routes
        .route("/api/stats/daily", get(handlers::statistics::get_daily_stats))
        .route("/api/stats/platforms", get(handlers::statistics::get_platform_stats))
        .route("/api/stats/overview", get(handlers::statistics::get_overview_stats))
        
        // Add state and middleware
        .with_state(db.clone())
        .layer(TraceLayer::new_for_http())
        .layer(cors);
        
    // Get bind address
    let addr = format!("{}:{}", config.api.host, config.api.port)
        .parse::<SocketAddr>()?;
        
    // Start server
    info!("Starting API server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
        
    Ok(())
}