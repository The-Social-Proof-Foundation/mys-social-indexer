// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::config::Config;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection = Object<AsyncPgConnection>;

// Define migrations
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Database wrapper for connection pool access
#[derive(Clone)]
pub struct Database {
    pub pool: Arc<DbPool>,
}

impl Database {
    /// Create a new database instance
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }
    
    /// Get a connection from the pool
    pub async fn get_connection(&self) -> Result<DbConnection> {
        self.pool.get().await
            .map_err(|e| anyhow!("Failed to get database connection: {}", e))
    }
}

/// Sets up the database connection pool
pub async fn setup_connection_pool(config: &Config) -> Result<Arc<Database>> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database.url);
    
    // Create the pool with basic configuration
    let pool = Pool::builder(manager)
        .max_size(config.database.max_connections as usize)
        .build()?;
    
    // Test the connection
    let _conn = pool.get().await?;
    
    // Create and return the database
    Ok(Arc::new(Database::new(pool)))
}

/// Run database migrations
pub fn run_migrations(config: &Config) -> Result<()> {
    // Use a regular blocking connection for migrations
    let mut conn = PgConnection::establish(&config.database.url)?;
    
    // Run migrations
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!("Migration error: {}", e))?;
    
    Ok(())
}