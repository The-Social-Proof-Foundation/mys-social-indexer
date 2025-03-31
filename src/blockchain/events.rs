// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use anyhow::{anyhow, Result};
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::db::{Database, DbConnection};
use crate::events::profile_events::ProfileCreatedEvent;
use crate::models::indexer::NewIndexerProgress;
use crate::schema;

use super::listener::BlockchainEvent;

/// Listener for profile events
pub struct ProfileEventListener {
    /// Database connection
    db: Arc<Database>,
    /// Event receiver channel
    rx: mpsc::Receiver<BlockchainEvent>,
    /// Worker ID for tracking progress
    worker_id: String,
}

impl ProfileEventListener {
    /// Create a new profile event listener
    pub fn new(db: Arc<Database>, rx: mpsc::Receiver<BlockchainEvent>, worker_id: String) -> Self {
        Self {
            db,
            rx,
            worker_id,
        }
    }
    
    /// Get a database connection from the pool
    async fn get_connection(&self) -> Result<DbConnection> {
        self.db.get_connection()
            .await
            .map_err(|e| anyhow!("Failed to get database connection: {}", e))
    }
    
    /// Update worker progress with timestamp
    async fn update_progress(&self, timestamp: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let now = Utc::now().naive_utc();
        
        // Check if the table exists, and if not, create it
        let table_exists: bool = diesel::dsl::select(
            diesel::dsl::sql::<diesel::sql_types::Bool>(
                "EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'indexer_progress')"
            )
        )
        .get_result(&mut conn)
        .await
        .unwrap_or(false);
        
        if !table_exists {
            tracing::info!("Creating indexer_progress table...");
            diesel::sql_query(
                "CREATE TABLE IF NOT EXISTS indexer_progress (
                    id VARCHAR PRIMARY KEY,
                    last_checkpoint_processed BIGINT NOT NULL DEFAULT 0,
                    last_processed_at TIMESTAMP NOT NULL DEFAULT NOW()
                )"
            )
            .execute(&mut conn)
            .await?;
        }
        
        let progress = NewIndexerProgress {
            id: self.worker_id.clone(),
            last_checkpoint_processed: timestamp as i64,
            last_processed_at: now,
        };
        
        diesel::insert_into(schema::indexer_progress::table)
            .values(&progress)
            .on_conflict(schema::indexer_progress::id)
            .do_update()
            .set((
                schema::indexer_progress::last_checkpoint_processed.eq(progress.last_checkpoint_processed),
                schema::indexer_progress::last_processed_at.eq(progress.last_processed_at),
            ))
            .execute(&mut conn)
            .await?;
            
        Ok(())
    }
    
    /// Try to manually parse a profile event from the raw JSON data
    async fn try_manual_profile_parse(&self, data: &serde_json::Value) -> Result<()> {
        info!("Manually extracting profile data from: {}", serde_json::to_string(data).unwrap_or_default());
        
        // Try different JSON paths to extract the profile data
        let profile = if let Some(obj) = data.as_object() {
            // Extract the fields we need
            let profile_id = obj.get("profile_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let owner = obj.get("owner_address").or(obj.get("owner")).and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let username = obj.get("username").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let display_name = obj.get("display_name").and_then(|v| v.as_str()).map(|s| s.to_string());
            let bio = obj.get("bio").and_then(|v| v.as_str()).map(|s| s.to_string());
            let profile_picture = obj.get("profile_picture").and_then(|v| v.as_str()).map(|s| s.to_string());
            
            // Create event manually
            ProfileCreatedEvent {
                profile_id,
                owner_address: owner,
                username,
                display_name,
                bio,
                profile_picture,
                cover_photo: None,
                created_at: chrono::Utc::now().timestamp() as u64,
            }
        } else {
            return Err(anyhow!("Data is not an object"));
        };
        
        // Process the manually constructed profile
        info!("Manually parsed profile: {:?}", profile);
        self.process_profile_created(&profile).await
    }
    
    /// Process a profile created event
    async fn process_profile_created(&self, event: &ProfileCreatedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_profile = event.into_model()?;
        
        // Insert the profile
        diesel::insert_into(schema::profiles::table)
            .values(&new_profile)
            .on_conflict(schema::profiles::username)
            .do_update()
            .set((
                schema::profiles::owner_address.eq(&new_profile.owner_address),
                schema::profiles::display_name.eq(&new_profile.display_name),
                schema::profiles::bio.eq(&new_profile.bio),
                schema::profiles::avatar_url.eq(&new_profile.avatar_url),
                schema::profiles::website_url.eq(&new_profile.website_url),
                schema::profiles::updated_at.eq(&new_profile.updated_at),
            ))
            .execute(&mut conn)
            .await?;
            
        info!("Processed profile created: {}", event.profile_id);
        Ok(())
    }
    
    /// Start listening for profile events
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting profile event listener");
        
        while let Some(event) = self.rx.recv().await {
            debug!("Received blockchain event: {:?}", event);
            
            // Check if this is a profile event
            if event.event_type.contains("::profile::") {
                info!("Processing profile event: {}", event.event_type);
                
                // Handle profile created event
                if event.event_type.ends_with("::ProfileCreatedEvent") {
                    // Log the raw event data for debugging
                    info!("Profile event detected with data: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
                    
                    match crate::events::parse_event::<ProfileCreatedEvent>(&event.data) {
                        Ok(profile_event) => {
                            info!("Successfully parsed profile event: {:?}", profile_event);
                            if let Err(e) = self.process_profile_created(&profile_event).await {
                                error!("Failed to process profile created event: {}", e);
                            }
                        },
                        Err(e) => {
                            error!("Failed to deserialize profile created event: {}", e);
                            
                            // Try to parse the profile event struct manually
                            info!("Attempting manual profile event parsing...");
                            let manual_parse = self.try_manual_profile_parse(&event.data).await;
                            if let Err(parse_err) = manual_parse {
                                error!("Manual parsing also failed: {}", parse_err);
                            }
                        }
                    }
                }
                // Add other profile event types as needed
                
                // Update progress after processing the event
                if let Err(e) = self.update_progress(event.timestamp_ms).await {
                    error!("Failed to update progress: {}", e);
                }
            }
        }
        
        warn!("Profile event listener channel closed");
        Ok(())
    }
}