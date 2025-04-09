// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use anyhow::{anyhow, Result};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use serde_json;

use crate::db::{Database, DbConnection};
use crate::events::blocking_events::{
    process_profile_block_event,
    process_profile_unblock_event,
    process_block_list_created_event
};

use super::listener::BlockchainEvent;

/// Handler for block list related blockchain events
pub struct BlockListEventHandler {
    /// Database connection
    db: Arc<Database>,
    /// Event receiver channel
    rx: mpsc::Receiver<BlockchainEvent>,
}

impl BlockListEventHandler {
    /// Create a new block list event handler
    pub fn new(db: Arc<Database>, rx: mpsc::Receiver<BlockchainEvent>, _worker_id: String) -> Self {
        Self {
            db,
            rx,
        }
    }
    
    /// Get a database connection from the pool
    async fn get_connection(&self) -> Result<DbConnection> {
        self.db.get_connection()
            .await
            .map_err(|e| anyhow!("Failed to get database connection: {}", e))
    }

    /// Process raw blockchain events
    async fn process_event(&self, event: BlockchainEvent) -> Result<()> {
        debug!("BlockList handler examining event: {}", event.event_type);
        
        // Only process events from the block_list module
        if !event.event_type.contains("::block_list::") {
            // Not from block_list module, skip it
            return Ok(());
        }
        
        // Log the raw event data for debugging
        info!("BlockList handler received event: {}", event.event_type);
        info!("Event data: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
        
        // Get a database connection
        let mut conn = self.get_connection().await?;
        
        // Process based on specific event type
        if event.event_type.contains("BlockListCreatedEvent") {
            info!("Processing BlockList creation event");
            process_block_list_created_event(&mut conn, &event.data).await?;
        } else if event.event_type.contains("BlockProfileEvent") {
            info!("Processing profile block event");
            process_profile_block_event(&mut conn, &event.data).await?;
        } else if event.event_type.contains("UnblockProfileEvent") {
            info!("Processing profile unblock event");
            process_profile_unblock_event(&mut conn, &event.data).await?;
        } else {
            // Unknown block_list event type
            debug!("Unknown block_list event type: {}", event.event_type);
        }
        
        Ok(())
    }
    
    /// Start listening for block list events
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting block list event handler");
        
        while let Some(event) = self.rx.recv().await {
            debug!("Received event: {:?}", event.event_type);
            
            if let Err(e) = self.process_event(event).await {
                error!("Error processing event: {}", e);
            }
        }
        
        warn!("Block list event handler channel closed");
        Ok(())
    }
}