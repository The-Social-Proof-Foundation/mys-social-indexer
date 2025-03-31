// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use anyhow::Result;
use futures::StreamExt;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use mys_sdk::{
    rpc_types::EventFilter,
    MysClientBuilder,
};

use crate::config::Config;
use crate::db::Database;
use crate::get_profile_package_address;

/// Type for events received from the blockchain
#[derive(Debug)]
pub struct BlockchainEvent {
    /// Transaction digest
    pub tx_digest: String,
    /// Event type
    pub event_type: String,
    /// Event data as JSON
    pub data: serde_json::Value,
    /// Timestamp from the blockchain
    pub timestamp_ms: u64,
}

/// Listener that connects to the blockchain and processes events
pub struct BlockchainEventListener {
    /// Configuration
    config: Config,
    /// Database connection
    db: Arc<Database>,
    /// Event handler channels
    event_senders: Mutex<Vec<mpsc::Sender<BlockchainEvent>>>,
}

impl BlockchainEventListener {
    /// Create a new blockchain event listener
    pub fn new(config: Config, db: Arc<Database>) -> Self {
        Self {
            config,
            db,
            event_senders: Mutex::new(Vec::new()),
        }
    }

    /// Register a new event handler
    pub async fn register_event_handler(&self, sender: mpsc::Sender<BlockchainEvent>) {
        let mut senders = self.event_senders.lock().await;
        senders.push(sender);
    }

    /// Process a blockchain event and forward it to all registered handlers
    async fn process_event(&self, event: BlockchainEvent) {
        let senders = self.event_senders.lock().await;
        for sender in senders.iter() {
            if let Err(e) = sender.send(event.clone()).await {
                error!("Failed to send event to handler: {}", e);
            }
        }
    }

    /// Start the blockchain event listener using websocket
    pub async fn start_ws_listener(&self) -> Result<()> {
        info!("Starting blockchain event listener using WebSocket");
        
        // Create MySocial client with WebSocket support
        let client = MysClientBuilder::default()
            .ws_url(&self.config.blockchain.ws_url)
            .build(&self.config.blockchain.rpc_url)
            .await?;
            
        info!("Connected to blockchain node: {}", self.config.blockchain.ws_url);
        
        // Get the profile package address
        let profile_address = get_profile_package_address();
        info!("Filtering events for profile package: {}", profile_address);
        
        // Create event filter for all events
        let event_filter = EventFilter::All([]);
        
        // Subscribe to events
        let mut event_stream = client.event_api().subscribe_event(event_filter).await?;
        info!("Successfully subscribed to profile events");
        
        // Process events as they arrive
        while let Some(event_result) = event_stream.next().await {
            match event_result {
                Ok(event) => {
                    debug!("Received event: {:?}", event);
                    
                    // Get timestamp with fallback
                    let timestamp_ms = event.timestamp_ms.unwrap_or_else(|| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64
                    });
                    
                    // Log the raw event for debugging
                    tracing::debug!("Raw blockchain event: {:?}", event);
                    
                    // Get the parsed JSON data
                    let parsed_data = event.parsed_json.clone();
                    
                    // Convert to blockchain event
                    let blockchain_event = BlockchainEvent {
                        tx_digest: event.id.tx_digest.to_string(),
                        event_type: event.type_.to_string(),
                        data: parsed_data,
                        timestamp_ms,
                    };
                    
                    // Process the event
                    self.process_event(blockchain_event).await;
                }
                Err(e) => {
                    error!("Error receiving event: {}", e);
                }
            }
        }
        
        warn!("Event stream ended unexpectedly");
        Ok(())
    }
    
    /// Start the blockchain event listener using polling
    pub async fn start_polling_listener(&self) -> Result<()> {
        info!("Starting blockchain event listener using polling");
        
        // Create MySocial client
        let client = MysClientBuilder::default()
            .build(&self.config.blockchain.rpc_url)
            .await?;
            
        info!("Connected to blockchain node: {}", self.config.blockchain.rpc_url);
        
        // Get the profile package address
        let profile_address = get_profile_package_address();
        info!("Filtering events for profile package: {}", profile_address);
        
        // Create event filter for all events
        let event_filter = EventFilter::All([]);
        
        // Create polling interval
        let mut interval = interval(Duration::from_millis(self.config.blockchain.poll_interval_ms));
        
        // Track the last seen event timestamp
        let mut last_seen_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis() as u64;
        
        // Poll for events
        loop {
            interval.tick().await;
            
            match client.event_api()
                .query_events(
                    event_filter.clone(),
                    None,
                    Some(self.config.blockchain.batch_size),
                    true, // descending order to get newest first
                ).await 
            {
                Ok(events) => {
                    // Process events in reverse order (oldest to newest)
                    for event in events.data.into_iter().rev() {
                        // Get the timestamp
                        let event_timestamp = event.timestamp_ms.unwrap_or(0);
                        
                        // Skip events we've already seen
                        if event_timestamp <= last_seen_timestamp {
                            continue;
                        }
                        
                        debug!("Processing event: {:?}", event);
                        
                        // Get timestamp with fallback
                        let timestamp_ms = event.timestamp_ms.unwrap_or_else(|| {
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64
                        });
                        
                        // Update the last seen timestamp
                        last_seen_timestamp = timestamp_ms;
                        
                        // Log the raw event for debugging
                        tracing::debug!("Raw blockchain event: {:?}", event);
                        
                        // Get the parsed JSON data
                        let parsed_data = event.parsed_json.clone();
                        
                        // Convert to blockchain event
                        let blockchain_event = BlockchainEvent {
                            tx_digest: event.id.tx_digest.to_string(),
                            event_type: event.type_.to_string(),
                            data: parsed_data,
                            timestamp_ms,
                        };
                        
                        // Process the event
                        self.process_event(blockchain_event).await;
                    }
                }
                Err(e) => {
                    error!("Error querying events: {}", e);
                }
            }
        }
    }
    
    /// Start the blockchain event listener using the preferred method
    pub async fn start(&self) -> Result<()> {
        // Try WebSocket first, fall back to polling if that fails
        match self.start_ws_listener().await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("WebSocket connection failed, falling back to polling: {}", e);
                self.start_polling_listener().await
            }
        }
    }
}

/// Allow cloning BlockchainEvent
impl Clone for BlockchainEvent {
    fn clone(&self) -> Self {
        Self {
            tx_digest: self.tx_digest.clone(),
            event_type: self.event_type.clone(),
            data: self.data.clone(),
            timestamp_ms: self.timestamp_ms,
        }
    }
}