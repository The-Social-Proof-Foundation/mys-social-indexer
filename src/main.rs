// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

use mys_social_indexer::{
    api,
    blockchain::{BlockchainEventListener, ProfileEventListener, SocialGraphEventHandler, PlatformEventHandler},
    config::Config,
    db,
    set_mysocial_package_address,
    get_mysocial_package_address,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    info!("Starting MySocial indexer...");
    
    // Load config from environment
    let config = Config::from_env();
    
    // Set MySocial package address from environment variable if provided
    let env_var_names = ["MYSOCIAL_PACKAGE_ADDRESS", "PROFILE_PACKAGE_ADDRESS", "PLATFORM_PACKAGE_ADDRESS"];
    
    let mut address_set = false;
    for var_name in env_var_names {
        if let Ok(address) = std::env::var(var_name) {
            set_mysocial_package_address(address.clone());
            info!("Set MySocial package address to {} (from {})", address, var_name);
            address_set = true;
            break;
        }
    }
    
    if !address_set {
        info!("Using default MySocial package address: {}", get_mysocial_package_address());
    }
    
    // Run database migrations
    info!("Running database migrations...");
    if let Err(e) = db::run_migrations(&config) {
        error!("Failed to run migrations: {}", e);
        return Err(e);
    }
    
    // Set up database connection pool
    info!("Setting up database connection pool...");
    let db_pool = db::setup_connection_pool(&config).await?;
    
    // Create event channels
    let (profile_tx, profile_rx) = mpsc::channel(100);
    let (social_graph_tx, social_graph_rx) = mpsc::channel(100);
    let (platform_tx, platform_rx) = mpsc::channel(100);
    
    // Create the blockchain event listener
    let blockchain_listener = Arc::new(BlockchainEventListener::new(config.clone(), db_pool.clone()));
    
    // Register event handlers
    blockchain_listener.register_event_handler(profile_tx).await;
    blockchain_listener.register_event_handler(social_graph_tx).await;
    blockchain_listener.register_event_handler(platform_tx).await;
    
    // Create and start profile event listener
    let mut profile_listener = ProfileEventListener::new(
        db_pool.clone(),
        profile_rx,
        "profile-worker".to_string(),
    );
    
    // Create and start social graph event handler
    let mut social_graph_handler = SocialGraphEventHandler::new(
        db_pool.clone(),
        social_graph_rx,
        "social-graph-worker".to_string(),
    );
    
    // Create and start platform event handler
    let mut platform_handler = PlatformEventHandler::new(
        db_pool.clone(),
        platform_rx,
        "platform-worker".to_string(),
    );
    
    let profile_handle = tokio::spawn(async move {
        if let Err(e) = profile_listener.start().await {
            error!("Profile event listener error: {}", e);
        }
    });
    
    let social_graph_handle = tokio::spawn(async move {
        if let Err(e) = social_graph_handler.start().await {
            error!("Social graph handler error: {}", e);
        }
    });
    
    let platform_handle = tokio::spawn(async move {
        if let Err(e) = platform_handler.start().await {
            error!("Platform handler error: {}", e);
        }
    });
    
    // Start the blockchain event listener
    let blockchain_handle = tokio::spawn({
        let listener = blockchain_listener.clone();
        async move {
            if let Err(e) = listener.start().await {
                error!("Blockchain event listener error: {}", e);
            }
        }
    });
    
    // Start the API server
    let api_handle = tokio::spawn(async move {
        if let Err(e) = api::setup_api_server(&config, db_pool).await {
            error!("API server error: {}", e);
        }
    });
    
    // Wait for all tasks to complete (they should run indefinitely)
    tokio::select! {
        _ = profile_handle => {
            error!("Profile event listener terminated unexpectedly");
        }
        _ = social_graph_handle => {
            error!("Social graph handler terminated unexpectedly");
        }
        _ = platform_handle => {
            error!("Platform handler terminated unexpectedly");
        }
        _ = blockchain_handle => {
            error!("Blockchain event listener terminated unexpectedly");
        }
        _ = api_handle => {
            error!("API server terminated unexpectedly");
        }
    }
    
    info!("Indexer terminated");
    
    Ok(())
}