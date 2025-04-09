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
use crate::events::blocking_events;
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

    /// Process platform block event
    async fn process_platform_block_event(&self, event_data: &serde_json::Value) -> Result<()> {
        let mut conn = self.get_connection().await?;
        blocking_events::process_platform_block_event(&mut conn, event_data).await
    }

    /// Process platform unblock event
    async fn process_platform_unblock_event(&self, event_data: &serde_json::Value) -> Result<()> {
        let mut conn = self.get_connection().await?;
        blocking_events::process_platform_unblock_event(&mut conn, event_data).await
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
            // Handle platform blocking events
            else if event.event_type.ends_with("::PlatformBlockedProfileEvent") {
                info!("Processing platform block event: {}", event.event_type);
                if let Err(e) = self.process_platform_block_event(&event.data).await {
                    error!("Failed to process platform block event: {}", e);
                }
                
                // Update progress after processing the event
                if let Err(e) = self.update_progress(event.timestamp_ms).await {
                    error!("Failed to update progress: {}", e);
                }
            }
            // Handle platform unblocking events
            else if event.event_type.ends_with("::PlatformUnblockedProfileEvent") {
                info!("Processing platform unblock event: {}", event.event_type);
                if let Err(e) = self.process_platform_unblock_event(&event.data).await {
                    error!("Failed to process platform unblock event: {}", e);
                }
                
                // Update progress after processing the event
                if let Err(e) = self.update_progress(event.timestamp_ms).await {
                    error!("Failed to update progress: {}", e);
                }
            }
            // Handle profile blocking events from block_list module
            else if event.event_type.ends_with("::UserBlockEvent") {
                info!("⚠️ DETECTED USER BLOCK EVENT: {}", event.event_type);
                info!("⚠️ EVENT DATA: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
                
                // Try to extract information from any possible structure
                let blocker_value = if let Some(obj) = event.data.as_object() {
                    // Try fields.blocker
                    if let Some(fields) = obj.get("fields").and_then(|f| f.as_object()) {
                        if let Some(blocker) = fields.get("blocker").and_then(|v| v.as_str()) {
                            info!("📌 Found blocker in fields.blocker: {}", blocker);
                            blocker
                        } else if let Some(blocker) = obj.get("blocker").and_then(|v| v.as_str()) {
                            info!("📌 Found blocker at root level: {}", blocker);
                            blocker
                        } else {
                            info!("❌ Could not find blocker in standard locations");
                            "unknown"
                        }
                    } else if let Some(blocker) = obj.get("blocker").and_then(|v| v.as_str()) {
                        info!("📌 Found blocker at root level: {}", blocker);
                        blocker
                    } else {
                        info!("❌ Could not find blocker in any location");
                        "unknown"
                    }
                } else {
                    info!("❌ Event data is not an object");
                    "unknown"
                };
                
                let blocked_value = if let Some(obj) = event.data.as_object() {
                    // Try fields.blocked
                    if let Some(fields) = obj.get("fields").and_then(|f| f.as_object()) {
                        if let Some(blocked) = fields.get("blocked").and_then(|v| v.as_str()) {
                            info!("📌 Found blocked in fields.blocked: {}", blocked);
                            blocked
                        } else if let Some(blocked) = obj.get("blocked").and_then(|v| v.as_str()) {
                            info!("📌 Found blocked at root level: {}", blocked);
                            blocked
                        } else {
                            info!("❌ Could not find blocked in standard locations");
                            "unknown"
                        }
                    } else if let Some(blocked) = obj.get("blocked").and_then(|v| v.as_str()) {
                        info!("📌 Found blocked at root level: {}", blocked);
                        blocked
                    } else {
                        info!("❌ Could not find blocked in any location");
                        "unknown"
                    }
                } else {
                    info!("❌ Event data is not an object");
                    "unknown"
                };
                
                // Check for the new module_name field
                let module_name = if let Some(obj) = event.data.as_object() {
                    if let Some(fields) = obj.get("fields").and_then(|f| f.as_object()) {
                        if let Some(module) = fields.get("module_name").and_then(|v| v.as_str()) {
                            info!("📌 Found module_name in fields.module_name: {}", module);
                            Some(module.to_string())
                        } else {
                            info!("❌ Could not find module_name in fields");
                            None
                        }
                    } else {
                        info!("❌ Could not find fields container");
                        None
                    }
                } else {
                    info!("❌ Event data is not an object");
                    None
                };
                
                info!("🔵 BLOCK EVENT SUMMARY: blocker={}, blocked={}, module={:?}", 
                    blocker_value, blocked_value, module_name);
                
                info!("------------------------------------------------------------------------------");
                info!("DETECTED BLOCK PROFILE EVENT: {}", event.event_type);
                info!("------------------------------------------------------------------------------");
                
                let mut conn = self.get_connection().await?;
                
                // Log extensive database connection info
                info!("Verifying database connection and schema...");
                
                // Check database version
                let db_verification = diesel::dsl::select(diesel::dsl::sql::<diesel::sql_types::Text>("version()"))
                    .get_result::<String>(&mut conn)
                    .await;
                
                match db_verification {
                    Ok(ver) => info!("Connected to database: {}", ver),
                    Err(e) => error!("Failed to verify database connection: {}", e),
                }
                
                // Check if profiles_blocked table exists
                let table_check = diesel::dsl::select(
                    diesel::dsl::sql::<diesel::sql_types::Bool>(
                        "EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'profiles_blocked')"
                    )
                )
                .get_result::<bool>(&mut conn)
                .await;
                
                match table_check {
                    Ok(exists) => {
                        if exists {
                            info!("✅ 'profiles_blocked' table exists in the database");
                            
                            // Use a more compatible approach to check the table schema
                            let col_count = diesel::dsl::select(
                                diesel::dsl::sql::<diesel::sql_types::BigInt>(
                                    "COUNT(*) FROM information_schema.columns WHERE table_name = 'profiles_blocked'"
                                )
                            )
                            .get_result::<i64>(&mut conn)
                            .await;
                            
                            match col_count {
                                Ok(count) => {
                                    info!("'profiles_blocked' table has {} columns", count);
                                    
                                    // Check if we have our required columns
                                    let required_cols = diesel::dsl::select(
                                        diesel::dsl::sql::<diesel::sql_types::Bool>(
                                            "EXISTS (SELECT 1 FROM information_schema.columns 
                                             WHERE table_name = 'profiles_blocked' 
                                             AND column_name = 'blocker_profile_id')"
                                        )
                                    )
                                    .get_result::<bool>(&mut conn)
                                    .await;
                                    
                                    match required_cols {
                                        Ok(has_cols) => {
                                            if has_cols {
                                                info!("✅ Required columns exist in profiles_blocked table");
                                            } else {
                                                error!("❌ Required column 'blocker_profile_id' not found in profiles_blocked table");
                                            }
                                        },
                                        Err(e) => error!("Failed to check for required columns: {}", e),
                                    }
                                    
                                    // Count existing records
                                    let record_count = diesel::dsl::select(
                                        diesel::dsl::sql::<diesel::sql_types::BigInt>(
                                            "COUNT(*) FROM profiles_blocked"
                                        )
                                    )
                                    .get_result::<i64>(&mut conn)
                                    .await;
                                    
                                    match record_count {
                                        Ok(count) => info!("Current records in profiles_blocked: {}", count),
                                        Err(e) => error!("Failed to count records: {}", e),
                                    }
                                },
                                Err(e) => error!("Failed to check table structure: {}", e),
                            }
                        } else {
                            error!("❌ 'profiles_blocked' table DOES NOT exist in the database!");
                            
                            // List some available tables
                            let table_count = diesel::dsl::select(
                                diesel::dsl::sql::<diesel::sql_types::BigInt>(
                                    "COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'"
                                )
                            )
                            .get_result::<i64>(&mut conn)
                            .await;
                            
                            match table_count {
                                Ok(count) => info!("Found {} tables in the database", count),
                                Err(e) => error!("Failed to count tables: {}", e),
                            }
                        }
                    },
                    Err(e) => error!("Failed to check if table exists: {}", e),
                }
                
                // Log the raw event data to debug JSON structure with full details
                let pretty_json = serde_json::to_string_pretty(&event.data).unwrap_or_default();
                info!("BLOCK EVENT RAW DATA:\n{}", pretty_json);
                
                // Try to process the event
                match blocking_events::process_profile_block_event(&mut conn, &event.data).await {
                    Ok(_) => {
                        info!("✅ Successfully processed profile block event");
                        
                        // Verify the database entry was created
                        use diesel::prelude::*;
                        use diesel_async::RunQueryDsl;
                        use crate::schema::profiles_blocked::dsl::*;
                        
                        // Extract blocker and blocked from the event data
                        let blocker_value = if let Some(obj) = event.data.as_object() {
                            if let Some(fields) = obj.get("fields").and_then(|f| f.as_object()) {
                                fields.get("blocker").and_then(|v| v.as_str()).unwrap_or_default()
                            } else {
                                obj.get("blocker").and_then(|v| v.as_str()).unwrap_or_default()
                            }
                        } else {
                            ""
                        };
                        
                        let blocked_value = if let Some(obj) = event.data.as_object() {
                            if let Some(fields) = obj.get("fields").and_then(|f| f.as_object()) {
                                fields.get("blocked").and_then(|v| v.as_str()).unwrap_or_default()
                            } else {
                                obj.get("blocked").and_then(|v| v.as_str()).unwrap_or_default()
                            }
                        } else {
                            ""
                        };
                        
                        if !blocker_value.is_empty() && !blocked_value.is_empty() {
                            let query = profiles_blocked
                                .filter(blocker_profile_id.eq(blocker_value))
                                .filter(blocked_profile_id.eq(blocked_value))
                                .select(id);
                                
                            info!("Executing verification query for blocker={}, blocked={}", 
                                blocker_value, blocked_value);
                                
                            // Use count to see if any records exist
                            let count_result = query.count().get_result::<i64>(&mut conn).await;
                            
                            match count_result {
                                Ok(count) => {
                                    if count > 0 {
                                        info!("✅ Verified database entry exists - found {} records", count);
                                    } else {
                                        error!("❌ No database entries found for this block relationship");
                                    }
                                },
                                Err(e) => error!("❌ Failed to verify database entry: {}", e),
                            }
                        }
                    },
                    Err(e) => {
                        error!("❌ Failed to process profile block event: {}", e);
                    }
                }
                
                // Update progress after processing the event
                if let Err(e) = self.update_progress(event.timestamp_ms).await {
                    error!("Failed to update progress: {}", e);
                }
            }
            // Handle profile unblocking events - only match UserUnblockEvent
            else if event.event_type.ends_with("::UserUnblockEvent") {
                info!("Processing profile unblock event: {}", event.event_type);
                let mut conn = self.get_connection().await?;
                
                // Log the raw event data to debug JSON structure
                info!("Raw unblock event data: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
                
                if let Err(e) = blocking_events::process_profile_unblock_event(&mut conn, &event.data).await {
                    error!("Failed to process profile unblock event: {}", e);
                } else {
                    info!("Successfully processed profile unblock event");
                }
                
                // Update progress after processing the event
                if let Err(e) = self.update_progress(event.timestamp_ms).await {
                    error!("Failed to update progress: {}", e);
                }
            }
            // Handle BlockList creation events
            else if event.event_type.ends_with("::BlockListCreatedEvent") {
                info!("Processing block list created event: {}", event.event_type);
                let mut conn = self.get_connection().await?;
                if let Err(e) = blocking_events::process_block_list_created_event(&mut conn, &event.data).await {
                    error!("Failed to process block list created event: {}", e);
                }
                
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