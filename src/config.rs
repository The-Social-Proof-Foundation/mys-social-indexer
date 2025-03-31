// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub blockchain: BlockchainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub poll_interval_ms: u64,
    pub batch_size: usize,
}

impl Config {
    pub fn from_env() -> Self {
        // Load .env file if present
        let _ = dotenv::dotenv();

        Config {
            database: DatabaseConfig {
                // Provide a default localhost PostgreSQL URL
                url: env::var("DATABASE_URL").unwrap_or_else(|_| 
                    "postgres://postgres:postgres@localhost:5432/myso_social_indexer".to_string()
                ),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .expect("DATABASE_MAX_CONNECTIONS must be a number"),
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .expect("SERVER_PORT must be a number"),
            },
            blockchain: BlockchainConfig {
                rpc_url: env::var("RPC_URL")
                    .unwrap_or_else(|_| "http://localhost:9000".to_string()),
                ws_url: env::var("WS_URL")
                    .unwrap_or_else(|_| "ws://localhost:9000".to_string()),
                poll_interval_ms: env::var("POLL_INTERVAL_MS")
                    .unwrap_or_else(|_| "5000".to_string()) // 5 seconds by default
                    .parse()
                    .expect("POLL_INTERVAL_MS must be a number"),
                batch_size: env::var("EVENT_BATCH_SIZE")
                    .unwrap_or_else(|_| "50".to_string()) // 50 events per batch by default
                    .parse()
                    .expect("EVENT_BATCH_SIZE must be a number"),
            },
        }
    }
}