// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info};

use mys_social_indexer::{
    api,
    blockchain::{BlockchainEventListener, ProfileEventListener},
    config::Config,
    db,
    set_profile_package_address,
    get_profile_package_address,
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
    
    // Set profile package address from environment variable if provided
    if let Ok(address) = std::env::var("PROFILE_PACKAGE_ADDRESS") {
        set_profile_package_address(address.clone());
        info!("Set profile package address to {}", address);
    } else {
        info!("Using default profile package address: {}", get_profile_package_address());
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
    
    // Create event channel for profile events
    let (tx, rx) = mpsc::channel(100);
    
    // Create the blockchain event listener
    let blockchain_listener = Arc::new(BlockchainEventListener::new(config.clone(), db_pool.clone()));
    
    // Register profile event handler
    blockchain_listener.register_event_handler(tx).await;
    
    // Create and start profile event listener
    let mut profile_listener = ProfileEventListener::new(
        db_pool.clone(),
        rx,
        "profile-worker".to_string(),
    );
    
    let profile_handle = tokio::spawn(async move {
        if let Err(e) = profile_listener.start().await {
            error!("Profile event listener error: {}", e);
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