use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tracing::info;

/// Global configuration instance
static CONFIG: OnceCell<Config> = OnceCell::new();

/// Configuration for the MySocial indexer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Indexer configuration
    pub indexer: IndexerConfig,
    
    /// API server configuration
    pub api: ApiConfig,
    
    /// Metrics configuration
    pub metrics: MetricsConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    
    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

/// Indexer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    /// URL to fetch checkpoints from
    pub checkpoint_url: String,
    
    /// Initial checkpoint to start processing from
    pub initial_checkpoint: Option<u64>,
    
    /// Number of concurrent workers for processing
    pub concurrency: usize,
    
    /// Path to save local progress
    pub progress_file_path: PathBuf,
    
    /// Monitoring interval in seconds
    pub monitoring_interval: u64,
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Host to bind API server to
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
    
    /// Enable CORS
    pub enable_cors: bool,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    
    /// Metrics endpoint port
    pub port: u16,
}

impl Config {
    /// Initialize configuration from environment variables
    pub fn init() -> Result<&'static Self> {
        let config = Config {
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                    "postgres://postgres:postgres@localhost:5432/mys_social_indexer".to_string()
                }),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                connection_timeout: env::var("DATABASE_CONNECTION_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            },
            indexer: IndexerConfig {
                checkpoint_url: env::var("CHECKPOINT_URL")
                    .unwrap_or_else(|_| "https://checkpoints.mainnet.mysocial.io".to_string()),
                initial_checkpoint: env::var("INITIAL_CHECKPOINT")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                concurrency: env::var("CONCURRENCY")
                    .unwrap_or_else(|_| "5".to_string())
                    .parse()
                    .unwrap_or(5),
                progress_file_path: PathBuf::from(
                    env::var("PROGRESS_FILE_PATH").unwrap_or_else(|_| "/tmp/social_indexer_progress".to_string()),
                ),
                monitoring_interval: env::var("MONITORING_INTERVAL")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30),
            },
            api: ApiConfig {
                host: env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("API_PORT")
                    .unwrap_or_else(|_| "3000".to_string())
                    .parse()
                    .unwrap_or(3000),
                enable_cors: env::var("ENABLE_CORS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            metrics: MetricsConfig {
                enabled: env::var("METRICS_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                port: env::var("METRICS_PORT")
                    .unwrap_or_else(|_| "9000".to_string())
                    .parse()
                    .unwrap_or(9000),
            },
        };

        // Log loaded configuration
        info!("Loaded configuration: {:?}", config);

        // Store config in the global instance
        CONFIG.set(config).expect("Failed to set global config");
        
        Ok(CONFIG.get().expect("Config not initialized"))
    }

    /// Get the global configuration instance
    pub fn get() -> &'static Self {
        CONFIG.get().expect("Config not initialized")
    }
}