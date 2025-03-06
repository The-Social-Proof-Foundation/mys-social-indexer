use anyhow::Result;
use dotenv::dotenv;
use mys_data_ingestion_core::{setup_single_workflow, FileProgressStore, WorkerPool};
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::signal;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use mys_social_indexer::config::Config;
use mys_social_indexer::db::init_database;
use mys_social_indexer::api;
use mys_social_indexer::worker::SocialIndexerWorker;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if present
    dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,mys_social_indexer=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Load configuration
    let config = Config::init()?;
    info!("Initialized configuration");
    
    // Initialize database
    let db = Arc::new(init_database().await?);
    info!("Connected to database");
    
    // Prepare termination signals
    let (term_sender, term_receiver) = oneshot::channel();
    
    // Set up worker for blockchain processing
    let checkpoint_url = config.indexer.checkpoint_url.clone();
    let initial_checkpoint = config.indexer.initial_checkpoint.unwrap_or(0);
    let worker_id = "social_indexer".to_string();
    
    // Create the worker with database access
    let worker = SocialIndexerWorker::new(db.clone(), worker_id.clone());
    
    // Create worker pool
    let worker_pool = WorkerPool::new(worker, worker_id, config.indexer.concurrency);
    
    // Start the blockchain indexer
    let executor_handle = tokio::spawn(async move {
        match setup_single_workflow(
            worker_pool,
            checkpoint_url,
            initial_checkpoint,
            config.indexer.concurrency,
            None,
        ).await {
            Ok((executor, _)) => {
                match executor.await {
                    Ok(_) => info!("Indexer finished successfully"),
                    Err(e) => error!("Indexer failed: {}", e),
                }
            },
            Err(e) => error!("Failed to set up indexer workflow: {}", e),
        }
    });
    
    // Start API server
    let api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_api_server(db).await {
            error!("API server error: {}", e);
        }
    });
    
    // Handle shutdown signals
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Shutdown signal received, initiating graceful shutdown");
                let _ = term_sender.send(());
            },
            Err(e) => error!("Failed to listen for shutdown signal: {}", e),
        }
    });
    
    // Wait for all tasks to complete
    let _ = tokio::join!(executor_handle, api_handle);
    
    info!("MySocial Indexer shutdown complete");
    Ok(())
}