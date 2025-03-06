use crate::config::Config;
use anyhow::Result;
use diesel::PgConnection;
use diesel_async::{
    pooled_connection::{AsyncDieselConnectionManager, PoolError},
    AsyncPgConnection, RunQueryDsl,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::time::Duration;
use deadpool::Runtime;
use tracing::{error, info};

pub type DbPool = deadpool::managed::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub type DbConnection = deadpool::managed::Object<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Database manager for the indexer
pub struct Database {
    pool: DbPool,
}

impl Database {
    /// Create a new database manager with connection pool
    pub async fn new() -> Result<Self> {
        let config = Config::get();
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&config.database.url);
        
        // Configure pool with connection parameters
        let pool = DbPool::builder(manager)
            .max_size(config.database.max_connections)
            .runtime(Runtime::Tokio1)
            .build()?;
            
        // Create database instance
        let db = Self { pool };
        
        // Test connection and run migrations
        db.initialize().await?;
        
        Ok(db)
    }
    
    /// Initialize the database by testing connection and running migrations
    async fn initialize(&self) -> Result<()> {
        // Test connection by getting a connection from the pool
        let _conn = self.get_connection().await?;
        info!("Successfully connected to the database");
        
        // Run migrations
        self.run_migrations()?;
        
        Ok(())
    }
    
    /// Run database migrations
    fn run_migrations(&self) -> Result<()> {
        let config = Config::get();
        let mut conn = PgConnection::establish(&config.database.url)?;
        
        // Apply migrations
        conn.run_pending_migrations(MIGRATIONS)?;
        info!("Database migrations applied successfully");
        
        Ok(())
    }
    
    /// Get a database connection from the pool
    pub async fn get_connection(&self) -> Result<DbConnection, PoolError> {
        self.pool.get().await
    }
    
    /// Get the database connection pool reference
    pub fn get_pool(&self) -> &DbPool {
        &self.pool
    }
}

/// Initialize database connection pool and run migrations
pub async fn init_database() -> Result<Database> {
    Database::new().await
}