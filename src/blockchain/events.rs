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
        info!("Manually extracting profile data from: {}", serde_json::to_string_pretty(data).unwrap_or_default());
        
        // Try different JSON paths to extract the profile data
        let profile = if let Some(obj) = data.as_object() {
            // Look for fields container in Move event structure
            let fields_container = if let Some(fields) = obj.get("fields") {
                fields.as_object()
            } else {
                Some(obj)
            };
            
            let fields = fields_container.unwrap_or(obj);
            
            // Log all available fields for debugging
            info!("Available fields: {:?}", fields.keys().collect::<Vec<_>>());
            
            // Extract the fields we need
            let profile_id = fields.get("profile_id").and_then(|v| v.as_str())
                .or_else(|| obj.get("profile_id").and_then(|v| v.as_str()))
                .unwrap_or("unknown").to_string();
                
            let owner = fields.get("owner_address").or(fields.get("owner")).and_then(|v| v.as_str())
                .or_else(|| obj.get("owner_address").or(obj.get("owner")).and_then(|v| v.as_str()))
                .unwrap_or("unknown").to_string();
                
            let username = fields.get("username").and_then(|v| v.as_str())
                .or_else(|| obj.get("username").and_then(|v| v.as_str()))
                .unwrap_or("unknown").to_string();
                
            let display_name = fields.get("display_name").and_then(|v| v.as_str())
                .or_else(|| obj.get("display_name").and_then(|v| v.as_str()))
                .map(|s| s.to_string());
                
            // Look for bio in various locations and formats
            let bio_value = fields.get("bio").or_else(|| obj.get("bio"));
            let bio = if let Some(b) = bio_value {
                if b.is_string() {
                    Some(b.as_str().unwrap().to_string())
                } else if b.is_object() {
                    // Try to extract string value from complex object
                    b.get("vec").and_then(|v| v.get(0)).and_then(|s| s.get("string"))
                     .and_then(|s| s.as_str()).map(|s| s.to_string())
                     .or_else(|| Some("".to_string()))  // Default to empty if we can't extract
                } else {
                    None
                }
            } else {
                None
            };
            
            // Extract profile photo with similar logic
            let profile_photo_value = fields.get("profile_picture")
                .or(fields.get("profile_photo"))
                .or(fields.get("avatar_url"))
                .or_else(|| obj.get("profile_picture"))
                .or_else(|| obj.get("profile_photo"))
                .or_else(|| obj.get("avatar_url"));
                
            let profile_photo = if let Some(p) = profile_photo_value {
                if p.is_string() {
                    Some(p.as_str().unwrap().to_string())
                } else if p.is_object() {
                    // Extract URL from complex object
                    p.get("fields").and_then(|f| f.get("url"))
                     .and_then(|u| u.get("vec")).and_then(|v| v.get(0))
                     .and_then(|s| s.get("string")).and_then(|s| s.as_str())
                     .map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            
            // Extract cover photo with similar logic
            let cover_photo_value = fields.get("cover_photo")
                .or_else(|| obj.get("cover_photo"));
                
            let cover_photo = if let Some(c) = cover_photo_value {
                if c.is_string() {
                    Some(c.as_str().unwrap().to_string())
                } else if c.is_object() {
                    // Extract URL from complex object
                    c.get("fields").and_then(|f| f.get("url"))
                     .and_then(|u| u.get("vec")).and_then(|v| v.get(0))
                     .and_then(|s| s.get("string")).and_then(|s| s.as_str())
                     .map(|s| s.to_string())
                } else {
                    None
                }
            } else {
                None
            };
            
            // Log extracted fields
            info!("Extracted profile_id: {}", profile_id);
            info!("Extracted owner: {}", owner);
            info!("Extracted username: {}", username);
            info!("Extracted display_name: {:?}", display_name);
            info!("Extracted bio: {:?}", bio);
            info!("Extracted profile_photo: {:?}", profile_photo);
            info!("Extracted cover_photo: {:?}", cover_photo);
            
            // Create event manually
            ProfileCreatedEvent {
                profile_id,
                owner_address: owner,
                username: Some(username),
                display_name: display_name.unwrap_or_default(),
                bio,
                profile_photo,
                cover_photo,
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
                schema::profiles::profile_photo.eq(&new_profile.profile_photo),
                schema::profiles::website.eq(&new_profile.website),
                schema::profiles::updated_at.eq(&new_profile.updated_at),
                schema::profiles::cover_photo.eq(&new_profile.cover_photo),
                schema::profiles::profile_id.eq(&new_profile.profile_id),
                schema::profiles::sensitive_data_updated_at.eq(&new_profile.sensitive_data_updated_at),
                // Sensitive fields
                schema::profiles::birthdate.eq(&new_profile.birthdate),
                schema::profiles::current_location.eq(&new_profile.current_location),
                schema::profiles::raised_location.eq(&new_profile.raised_location),
                schema::profiles::phone.eq(&new_profile.phone),
                schema::profiles::email.eq(&new_profile.email),
                schema::profiles::gender.eq(&new_profile.gender),
                schema::profiles::political_view.eq(&new_profile.political_view),
                schema::profiles::religion.eq(&new_profile.religion),
                schema::profiles::education.eq(&new_profile.education),
                schema::profiles::primary_language.eq(&new_profile.primary_language),
                schema::profiles::relationship_status.eq(&new_profile.relationship_status),
                schema::profiles::x_username.eq(&new_profile.x_username),
                schema::profiles::mastodon_username.eq(&new_profile.mastodon_username),
                schema::profiles::facebook_username.eq(&new_profile.facebook_username),
                schema::profiles::reddit_username.eq(&new_profile.reddit_username),
                schema::profiles::github_username.eq(&new_profile.github_username)
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