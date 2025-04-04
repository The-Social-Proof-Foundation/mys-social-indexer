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

/// Type for events received from the blockchain
#[derive(Debug)]
pub struct BlockchainEvent {
    /// Transaction digest
    pub tx_digest: String,
    /// Unique event ID (in format <digest>:<event_seq>)
    pub event_id: String,
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
    /// Event handler channels
    event_senders: Mutex<Vec<mpsc::Sender<BlockchainEvent>>>,
}

impl BlockchainEventListener {
    /// Create a new blockchain event listener
    pub fn new(config: Config, _db: Arc<Database>) -> Self {
        // Note: db parameter kept for API compatibility but not used
        Self {
            config,
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
        
        // Get the addresses of all packages to monitor
        let package_addresses = crate::get_monitored_package_addresses();
        for address in &package_addresses {
            info!("Monitoring events for package: {}", address);
        }
        
        // Create event filter for all events
        // This will capture all events - we'll filter by package and module in our handlers
        let event_filter = EventFilter::All([]);
        
        // Subscribe to events
        let mut event_stream = client.event_api().subscribe_event(event_filter).await?;
        info!("Successfully subscribed to blockchain events");
        
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
                    
                    // Log the complete raw event structure for detailed debugging
                    tracing::info!("Complete raw blockchain event JSON: {}", serde_json::to_string_pretty(&event).unwrap_or_default());
                    tracing::info!("Parsed JSON data: {}", serde_json::to_string_pretty(&parsed_data).unwrap_or_default());
                    
                    // Log all events that might be relevant
                    if event.type_.to_string().contains("::profile::") || 
                       event.type_.to_string().contains("::social_graph::") ||
                       event.type_.to_string().contains("::FollowEvent") ||
                       event.type_.to_string().contains("::UnfollowEvent") ||
                       event.type_.to_string().contains("::platform::") ||
                       event.type_.to_string().contains("::Platform") {
                        tracing::info!("SOCIAL/PLATFORM EVENT DETECTED - Analyzing structure...");
                        
                        // Log the event type
                        tracing::info!("Event type: {}", event.type_);
                        
                        // Try to look into the parsed_json structure
                        if let Some(obj) = parsed_data.as_object() {
                            tracing::info!("Top-level keys: {:?}", obj.keys().collect::<Vec<_>>());
                            
                            // Check if this contains a Move object with fields
                            if let Some(fields) = obj.get("fields") {
                                tracing::info!("Move object fields found: {}", serde_json::to_string_pretty(fields).unwrap_or_default());
                                
                                // Look specifically for content fields
                                if let Some(content) = obj.get("content") {
                                    tracing::info!("Content section found: {}", serde_json::to_string_pretty(content).unwrap_or_default());
                                    
                                    // Try to extract fields from content section
                                    if let Some(content_obj) = content.as_object() {
                                        if let Some(content_fields) = content_obj.get("fields") {
                                            tracing::info!("Content fields section found: {}", serde_json::to_string_pretty(content_fields).unwrap_or_default());
                                        }
                                    }
                                }
                                
                                // Look for specific fields we need
                                tracing::info!("Looking for specific fields...");
                                for field_name in ["bio", "profile_picture", "cover_photo"] {
                                    if let Some(field_value) = obj.get(field_name) {
                                        tracing::info!("Found '{}' at top level: {}", field_name, field_value);
                                    } else if let Some(fields_obj) = fields.as_object() {
                                        if let Some(field_value) = fields_obj.get(field_name) {
                                            tracing::info!("Found '{}' in fields section: {}", field_name, field_value);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Generate event ID
                    let event_id = format!("{}:{}", event.id.tx_digest, event.id.event_seq); 
                    
                    // Convert to blockchain event
                    let blockchain_event = BlockchainEvent {
                        tx_digest: event.id.tx_digest.to_string(),
                        event_id,
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
        
        // Get the addresses of all packages to monitor
        let package_addresses = crate::get_monitored_package_addresses();
        for address in &package_addresses {
            info!("Monitoring events for package: {}", address);
        }
        
        // Create event filter for all events
        // This will capture all events - we'll filter by package and module in our handlers
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
                        
                        // Generate the event ID in format <digest>:<event_seq>
                        let event_id = format!("{}:{}", event.id.tx_digest, event.id.event_seq);
                        
                        // Convert to blockchain event
                        let blockchain_event = BlockchainEvent {
                            tx_digest: event.id.tx_digest.to_string(),
                            event_id,
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
            event_id: self.event_id.clone(),
            event_type: self.event_type.clone(),
            data: self.data.clone(),
            timestamp_ms: self.timestamp_ms,
        }
    }
}