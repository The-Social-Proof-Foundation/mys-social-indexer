// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
// Serde json utilities

use crate::db::{Database, DbConnection};
use crate::events::platform_events::PlatformEventType;
use crate::models::platform::*;
use crate::schema;

use super::listener::BlockchainEvent;

// Helper functions for extracting fields from blockchain events
fn extract_string_field(data: &serde_json::Value, field_name: &str) -> String {
    // Try direct access
    if let Some(value) = data.get(field_name) {
        if let Some(s) = value.as_str() {
            return s.to_string();
        }
    }
    
    // Try nested fields
    if field_name.contains('.') {
        let parts: Vec<&str> = field_name.split('.').collect();
        let mut current = data;
        
        for part in parts {
            if let Some(next) = current.get(part) {
                current = next;
            } else {
                return String::new();
            }
        }
        
        if let Some(s) = current.as_str() {
            return s.to_string();
        }
        
        // Try as number (for status)
        if let Some(n) = current.as_u64() {
            return n.to_string();
        }
    }
    
    // Try accessing any field that might match
    for (key, value) in data.as_object().unwrap_or(&serde_json::Map::new()) {
        if key.contains(field_name) || field_name.contains(key) {
            if let Some(s) = value.as_str() {
                return s.to_string();
            }
        }
    }
    
    String::new()
}

fn extract_string_array(data: &serde_json::Value, field_name: &str) -> Vec<String> {
    // Try direct access
    if let Some(value) = data.get(field_name) {
        if let Some(arr) = value.as_array() {
            return arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
    }
    
    // Try as a single string
    let single = extract_string_field(data, field_name);
    if !single.is_empty() {
        return vec![single];
    }
    
    Vec::new()
}

fn extract_number_field(data: &serde_json::Value, field_name: &str) -> Option<u8> {
    // Try direct access
    if let Some(value) = data.get(field_name) {
        if let Some(n) = value.as_u64() {
            return Some(n as u8);
        }
    }
    
    // Try nested fields
    if field_name.contains('.') {
        let parts: Vec<&str> = field_name.split('.').collect();
        let mut current = data;
        
        for part in parts {
            if let Some(next) = current.get(part) {
                current = next;
            } else {
                return None;
            }
        }
        
        if let Some(n) = current.as_u64() {
            return Some(n as u8);
        }
    }
    
    // Try as string
    let str_val = extract_string_field(data, field_name);
    if !str_val.is_empty() {
        if let Ok(n) = str_val.parse::<u8>() {
            return Some(n);
        }
    }
    
    None
}

/// Handler for platform-related blockchain events
pub struct PlatformEventHandler {
    /// Database connection
    db: Arc<Database>,
    /// Event receiver channel
    rx: mpsc::Receiver<BlockchainEvent>,
}

impl PlatformEventHandler {
    /// Create a new platform event handler
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
    
    /// Process a platform created event
    async fn process_platform_created_event(&self, event: &PlatformCreatedEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing platform created event");
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::PlatformCreated.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Check if platform already exists
                let platform_exists = schema::platforms::table
                    .filter(schema::platforms::platform_id.eq(&event.platform_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if platform_exists {
                    debug!("Platform already exists: {}", event.platform_id);
                    // Update existing platform with new data if needed
                    let platform_update = UpdatePlatform {
                        name: Some(event.name.clone()),
                        tagline: Some(event.tagline.clone()),
                        description: event.description.clone(), // Use the description from the event
                        logo: event.logo.clone(), // Use the logo from the event
                        terms_of_service: Some(event.terms_of_service.clone()),
                        privacy_policy: Some(event.privacy_policy.clone()),
                        platform_names: Some(serde_json::to_value(&event.platforms).unwrap_or_default()),
                        links: Some(serde_json::to_value(&event.links).unwrap_or_default()),
                        status: Some(event.status.status as i16),
                        release_date: Some(event.release_date.clone()),
                        shutdown_date: None,
                        updated_at: Some(chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc()),
                        is_approved: None, // Don't change approval status on update
                        approval_changed_at: None, // Don't change approval timestamp
                        approved_by: None, // Don't change approver
                    };
                    
                    diesel::update(schema::platforms::table)
                        .filter(schema::platforms::platform_id.eq(&event.platform_id))
                        .set(&platform_update)
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Updated existing platform: {}", event.platform_id);
                } else {
                    // Create new platform
                    let new_platform = NewPlatform {
                        platform_id: event.platform_id.clone(),
                        name: event.name.clone(),
                        tagline: event.tagline.clone(),
                        description: event.description.clone(), // Use the description from the event
                        logo: event.logo.clone(), // Use the logo from the event
                        developer_address: event.developer.clone(),
                        terms_of_service: Some(event.terms_of_service.clone()),
                        privacy_policy: Some(event.privacy_policy.clone()),
                        platform_names: Some(serde_json::to_value(&event.platforms).unwrap_or_default()),
                        links: Some(serde_json::to_value(&event.links).unwrap_or_default()),
                        status: event.status.status as i16,
                        release_date: Some(event.release_date.clone()),
                        shutdown_date: None,
                        created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc(),
                        updated_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc(),
                        is_approved: false, // New platforms are not approved by default
                        approval_changed_at: None, // No approval change yet
                        approved_by: None, // No approver yet
                    };
                    
                    // Insert platform
                    diesel::insert_into(schema::platforms::table)
                        .values(&new_platform)
                        .execute(&mut conn)
                        .await?;
                    
                    // Add developer as a moderator
                    let new_moderator = NewPlatformModerator {
                        platform_id: event.platform_id.clone(),
                        moderator_address: event.developer.clone(),
                        added_by: event.developer.clone(), // Developer adds themselves
                        created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc(),
                    };
                    
                    // Insert developer as moderator
                    diesel::insert_into(schema::platform_moderators::table)
                        .values(&new_moderator)
                        .on_conflict((
                            schema::platform_moderators::platform_id, 
                            schema::platform_moderators::moderator_address
                        ))
                        .do_nothing() // If already exists, do nothing
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Created new platform: {}", event.platform_id);
                }
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed platform created event");
            
        Ok(())
    }
    
    /// Process a platform updated event
    async fn process_platform_updated_event(&self, event: &PlatformUpdatedEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing platform updated event");
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::PlatformUpdated.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Check if platform exists
                let platform_exists = schema::platforms::table
                    .filter(schema::platforms::platform_id.eq(&event.platform_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if platform_exists {
                    // Update existing platform
                    let platform_update = UpdatePlatform {
                        name: Some(event.name.clone()),
                        tagline: Some(event.tagline.clone()),
                        description: Some(event.description.clone()),
                        logo: None, // Not in updated event
                        terms_of_service: Some(event.terms_of_service.clone()),
                        privacy_policy: Some(event.privacy_policy.clone()),
                        platform_names: Some(serde_json::to_value(&event.platforms).unwrap_or_default()),
                        links: Some(serde_json::to_value(&event.links).unwrap_or_default()),
                        status: Some(event.status.status as i16),
                        release_date: Some(event.release_date.clone()),
                        shutdown_date: event.shutdown_date.clone().map(Some).unwrap_or(None),
                        updated_at: Some(chrono::DateTime::from_timestamp(event.updated_at as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc()),
                        is_approved: None, // Don't change approval status on regular update
                        approval_changed_at: None, // Don't change approval timestamp
                        approved_by: None, // Don't change approver
                    };
                    
                    diesel::update(schema::platforms::table)
                        .filter(schema::platforms::platform_id.eq(&event.platform_id))
                        .set(&platform_update)
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Updated platform: {}", event.platform_id);
                } else {
                    // Platform doesn't exist, this is unusual but we'll create it
                    warn!("Platform update for non-existent platform: {}", event.platform_id);
                    
                    // Create platform with limited information from update event
                    // (we don't have developer info in the update event)
                    let new_platform = NewPlatform {
                        platform_id: event.platform_id.clone(),
                        name: event.name.clone(),
                        tagline: event.tagline.clone(),
                        description: Some(event.description.clone()),
                        logo: None,
                        developer_address: "unknown".to_string(), // We don't have this info
                        terms_of_service: Some(event.terms_of_service.clone()),
                        privacy_policy: Some(event.privacy_policy.clone()),
                        platform_names: Some(serde_json::to_value(&event.platforms).unwrap_or_default()),
                        links: Some(serde_json::to_value(&event.links).unwrap_or_default()),
                        status: event.status.status as i16,
                        release_date: Some(event.release_date.clone()),
                        shutdown_date: event.shutdown_date.clone().map(Some).unwrap_or(None),
                        created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc(),
                        updated_at: chrono::DateTime::from_timestamp(event.updated_at as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc(),
                        is_approved: false, // New platforms are not approved by default
                        approval_changed_at: None, // No approval change yet
                        approved_by: None, // No approver yet
                    };
                    
                    diesel::insert_into(schema::platforms::table)
                        .values(&new_platform)
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Created missing platform from update event: {}", event.platform_id);
                }
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed platform updated event");
            
        Ok(())
    }
    
    /// Process a moderator added event
    async fn process_moderator_added_event(&self, event: &ModeratorAddedEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing moderator added event");
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::ModeratorAdded.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Check if platform exists
                let platform_exists = schema::platforms::table
                    .filter(schema::platforms::platform_id.eq(&event.platform_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if !platform_exists {
                    // Create a placeholder platform if it doesn't exist
                    warn!("Moderator added for non-existent platform: {}", event.platform_id);
                    
                    let new_platform = NewPlatform {
                        platform_id: event.platform_id.clone(),
                        name: format!("Unknown Platform ({})", event.platform_id),
                        tagline: "Platform metadata not available".to_string(),
                        description: None,
                        logo: None,
                        developer_address: event.added_by.clone(), // Assume the adder is the developer
                        terms_of_service: None,
                        privacy_policy: None,
                        platform_names: None,
                        links: None,
                        status: PLATFORM_STATUS_DEVELOPMENT, // Default to development status
                        release_date: None,
                        shutdown_date: None,
                        created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc(),
                        updated_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc(),
                        is_approved: false, // New platforms are not approved by default
                        approval_changed_at: None, // No approval change yet
                        approved_by: None, // No approver yet
                    };
                    
                    diesel::insert_into(schema::platforms::table)
                        .values(&new_platform)
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Created placeholder platform for moderator: {}", event.platform_id);
                }
                
                // Add moderator to platform
                let new_moderator = NewPlatformModerator {
                    platform_id: event.platform_id.clone(),
                    moderator_address: event.moderator_address.clone(),
                    added_by: event.added_by.clone(),
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert moderator with conflict handling
                diesel::insert_into(schema::platform_moderators::table)
                    .values(&new_moderator)
                    .on_conflict((
                        schema::platform_moderators::platform_id, 
                        schema::platform_moderators::moderator_address
                    ))
                    .do_nothing() // If already exists, do nothing
                    .execute(&mut conn)
                    .await?;
                
                info!("Added moderator {} to platform {}", event.moderator_address, event.platform_id);
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed moderator added event");
            
        Ok(())
    }
    
    /// Process a moderator removed event
    async fn process_moderator_removed_event(&self, event: &ModeratorRemovedEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing moderator removed event");
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::ModeratorRemoved.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Remove moderator from platform
                diesel::delete(
                    schema::platform_moderators::table
                        .filter(schema::platform_moderators::platform_id.eq(&event.platform_id))
                        .filter(schema::platform_moderators::moderator_address.eq(&event.moderator_address))
                )
                .execute(&mut conn)
                .await?;
                
                info!("Removed moderator {} from platform {}", event.moderator_address, event.platform_id);
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed moderator removed event");
            
        Ok(())
    }
    
    /// Process a profile blocked event
    async fn process_profile_blocked_event(&self, event: &PlatformBlockedProfileEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing profile blocked event");
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::ProfileBlocked.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Check if this platform-profile relationship already exists
                let existing_relationship = schema::platform_blocked_profiles::table
                    .filter(schema::platform_blocked_profiles::platform_id.eq(&event.platform_id))
                    .filter(schema::platform_blocked_profiles::profile_id.eq(&event.profile_id))
                    .first::<PlatformBlockedProfile>(&mut conn)
                    .await;
                
                match existing_relationship {
                    Ok(_) => {
                        // Delete the existing record - we'll insert a new one to reset the timestamps
                        diesel::delete(schema::platform_blocked_profiles::table)
                            .filter(schema::platform_blocked_profiles::platform_id.eq(&event.platform_id))
                            .filter(schema::platform_blocked_profiles::profile_id.eq(&event.profile_id))
                            .execute(&mut conn)
                            .await?;
                        
                        info!("Deleted existing block relationship to refresh timestamp");
                        
                        // Create new blocked profile relationship below
                    },
                    Err(diesel::result::Error::NotFound) => {
                        // No existing relationship - we'll create a new one
                    },
                    Err(e) => {
                        error!("Error checking for existing block relationship: {}", e);
                        return Err(e);
                    }
                }
                
                // Create new blocked profile relationship
                let new_blocked_profile = (
                    schema::platform_blocked_profiles::platform_id.eq(event.platform_id.clone()),
                    schema::platform_blocked_profiles::profile_id.eq(event.profile_id.clone()),
                    schema::platform_blocked_profiles::blocked_by.eq(event.blocked_by.clone()),
                    schema::platform_blocked_profiles::created_at.eq(
                        chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc()
                    )
                );
                
                diesel::insert_into(schema::platform_blocked_profiles::table)
                    .values(new_blocked_profile)
                    .execute(&mut conn)
                    .await?;
                
                info!("Created new blocked profile relationship: {} on platform {}", event.profile_id, event.platform_id);
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed profile blocked event");
            
        Ok(())
    }
    
    /// Process a profile unblocked event
    async fn process_profile_unblocked_event(&self, event: &PlatformUnblockedProfileEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing profile unblocked event");
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::ProfileUnblocked.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Delete the block relationship entirely instead of updating it
                let deleted_count = diesel::delete(schema::platform_blocked_profiles::table)
                    .filter(schema::platform_blocked_profiles::platform_id.eq(&event.platform_id))
                    .filter(schema::platform_blocked_profiles::profile_id.eq(&event.profile_id))
                    .execute(&mut conn)
                    .await?;
                
                if deleted_count > 0 {
                    info!("Deleted block relationship: {} on platform {}", event.profile_id, event.platform_id);
                } else {
                    warn!("No block relationship found to delete: {} on platform {}", event.profile_id, event.platform_id);
                }
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed profile unblocked event");
            
        Ok(())
    }
    
    /// Process a platform approval changed event
    async fn process_platform_approval_changed_event(&self, event: &PlatformApprovalChangedEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing platform approval changed event for platform: {}", event.platform_id);
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::PlatformApprovalChanged.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Check if platform exists
                let platform_exists = schema::platforms::table
                    .filter(schema::platforms::platform_id.eq(&event.platform_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if platform_exists {
                    // Get timestamp from event
                    let approval_changed_at = chrono::DateTime::from_timestamp(event.changed_at as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc();
                    
                    // Update platform approval status
                    let platform_update = UpdatePlatform {
                        name: None,
                        tagline: None,
                        description: None,
                        logo: None,
                        terms_of_service: None,
                        privacy_policy: None,
                        platform_names: None,
                        links: None,
                        status: None,
                        release_date: None,
                        shutdown_date: None,
                        updated_at: Some(approval_changed_at),
                        is_approved: Some(event.is_approved),
                        approval_changed_at: Some(approval_changed_at),
                        approved_by: Some(event.approved_by.clone()),
                    };
                    
                    diesel::update(schema::platforms::table)
                        .filter(schema::platforms::platform_id.eq(&event.platform_id))
                        .set(&platform_update)
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Updated platform approval status: platform_id={}, is_approved={}", 
                        event.platform_id, event.is_approved);
                } else {
                    warn!("Platform not found for approval change: {}", event.platform_id);
                }
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed platform approval changed event");
            
        Ok(())
    }
    
    /// Process a user joined platform event
    async fn process_user_joined_platform_event(&self, event: &UserJoinedPlatformEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing user joined platform event");
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::UserJoinedPlatform.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Check if the platform is approved - only approved platforms can be joined
                let platform_is_approved = schema::platforms::table
                    .filter(schema::platforms::platform_id.eq(&event.platform_id))
                    .select(schema::platforms::is_approved)
                    .first::<bool>(&mut conn)
                    .await
                    .unwrap_or(false);
                
                if !platform_is_approved {
                    warn!("Ignoring join event for non-approved platform: {}", event.platform_id);
                    return Ok(());
                }
                
                // Check if the profile is blocked by the platform
                let profile_is_blocked = schema::platform_blocked_profiles::table
                    .filter(schema::platform_blocked_profiles::platform_id.eq(&event.platform_id))
                    .filter(schema::platform_blocked_profiles::profile_id.eq(&event.profile_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if profile_is_blocked {
                    warn!("Ignoring join event for blocked profile: {} in platform {}", event.profile_id, event.platform_id);
                    return Ok(());
                }
                
                // Check if membership already exists
                let membership_exists = schema::platform_memberships::table
                    .filter(schema::platform_memberships::platform_id.eq(&event.platform_id))
                    .filter(schema::platform_memberships::profile_id.eq(&event.profile_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if !membership_exists {
                    // Create new membership
                    let new_membership = NewPlatformMembership {
                        platform_id: event.platform_id.clone(),
                        profile_id: event.profile_id.clone(),
                        joined_at: chrono::DateTime::from_timestamp(event.timestamp as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                            .naive_utc(),
                    };
                    
                    // Insert membership
                    diesel::insert_into(schema::platform_memberships::table)
                        .values(new_membership)
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Created new platform membership: {} -> {}", event.profile_id, event.platform_id);
                    
                    // Also create a profile event for this action to track in profile history
                    let platform_join_event = crate::events::profile_event_types::PlatformJoinedEvent {
                        profile_id: event.profile_id.clone(),
                        platform_id: event.platform_id.clone(),
                        timestamp: event.timestamp,
                    };
                    
                    // We need to get the event ID again since it was moved in the platform_event
                    let event_id_for_profile = blockchain_event.map(|e| e.event_id.clone());
                    
                    let profile_event = crate::models::profile_events::NewProfileEvent::from_platform_joined(
                        &platform_join_event,
                        event_id_for_profile
                    );
                    
                    // Insert into profile events table
                    diesel::insert_into(schema::profile_events::table)
                        .values(&profile_event)
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Created profile event for platform join: {} -> {}", event.profile_id, event.platform_id);
                }
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed user joined platform event");
            
        Ok(())
    }
    
    /// Process a user left platform event
    async fn process_user_left_platform_event(&self, event: &UserLeftPlatformEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing user left platform event");
        
        let mut conn = self.get_connection().await?;
        
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Store event for historical record
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                // Create new platform event record
                let platform_event = NewPlatformEvent {
                    event_type: PlatformEventType::UserLeftPlatform.to_str().to_string(),
                    platform_id: event.platform_id.clone(),
                    event_data: serde_json::to_value(event).unwrap_or_default(),
                    event_id,
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                };
                
                // Insert platform event
                diesel::insert_into(schema::platform_events::table)
                    .values(&platform_event)
                    .execute(&mut conn)
                    .await?;
                
                // Update existing membership if it exists
                let membership_exists = schema::platform_memberships::table
                    .filter(schema::platform_memberships::platform_id.eq(&event.platform_id))
                    .filter(schema::platform_memberships::profile_id.eq(&event.profile_id))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if membership_exists {
                    // Delete the membership record
                    diesel::delete(schema::platform_memberships::table)
                        .filter(schema::platform_memberships::platform_id.eq(&event.platform_id))
                        .filter(schema::platform_memberships::profile_id.eq(&event.profile_id))
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Deleted platform membership for user leaving: {} -> {}", event.profile_id, event.platform_id);
                    
                    // Also create a profile event for this action to track in profile history
                    let platform_left_event = crate::events::profile_event_types::PlatformLeftEvent {
                        profile_id: event.profile_id.clone(),
                        platform_id: event.platform_id.clone(),
                        timestamp: event.timestamp,
                    };
                    
                    // We need to get the event ID again since it was moved in the platform_event
                    let event_id_for_profile = blockchain_event.map(|e| e.event_id.clone());
                    
                    let profile_event = crate::models::profile_events::NewProfileEvent::from_platform_left(
                        &platform_left_event,
                        event_id_for_profile
                    );
                    
                    // Insert into profile events table
                    diesel::insert_into(schema::profile_events::table)
                        .values(&profile_event)
                        .execute(&mut conn)
                        .await?;
                    
                    info!("Created profile event for platform leave: {} -> {}", event.profile_id, event.platform_id);
                }
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed user left platform event");
            
        Ok(())
    }
    
    /// Process raw blockchain events
    async fn process_event(&self, event: BlockchainEvent) -> Result<()> {
        debug!("Platform handler examining event: {}", event.event_type);
        
        // Skip BlockProfileEvents - let them be handled by the profile handler
        if event.event_type.contains("BlockProfileEvent") {
            info!("🚨 Platform handler skipping BlockProfileEvent: {}", event.event_type);
            info!("🚨 Event data: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
            return Ok(());
        }
        
        // Log the raw event data for debugging
        info!("Platform handler received event: {}", event.event_type);
        info!("Event data: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
        
        // Use the PlatformEventType from_str method which handles package prefixes
        if let Some(event_type) = crate::events::platform_events::PlatformEventType::from_str(&event.event_type) {
            info!("Identified platform event type: {:?}", event_type);
            
            match event_type {
                PlatformEventType::PlatformCreated => {
                    info!("Processing PlatformCreated event");
                    // Log complete event data for debugging
                    info!("PlatformCreated event data: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
                    
                    // First try normal deserialization
                    match serde_json::from_value::<PlatformCreatedEvent>(event.data.clone()) {
                        Ok(platform_event) => {
                            self.process_platform_created_event(&platform_event, Some(&event)).await?;
                        },
                        Err(e) => {
                            warn!("Failed to deserialize PlatformCreatedEvent normally: {}", e);
                            
                            // Try to extract fields manually if normal deserialization fails
                            let mut platform_event = PlatformCreatedEvent {
                                platform_id: extract_string_field(&event.data, "platform_id"),
                                name: extract_string_field(&event.data, "name"),
                                tagline: extract_string_field(&event.data, "tagline"),
                                description: {
                                    // Simple description extraction
                                    let desc = extract_string_field(&event.data, "description");
                                    if !desc.is_empty() {
                                        Some(desc)
                                    } else {
                                        None
                                    }
                                },
                                developer: extract_string_field(&event.data, "developer"),
                                logo: {
                                    // Simple logo extraction
                                    let logo = extract_string_field(&event.data, "logo");
                                    if !logo.is_empty() {
                                        Some(logo)
                                    } else {
                                        None
                                    }
                                },
                                terms_of_service: extract_string_field(&event.data, "terms_of_service"),
                                privacy_policy: extract_string_field(&event.data, "privacy_policy"),
                                platforms: extract_string_array(&event.data, "platforms"),
                                links: extract_string_array(&event.data, "links"),
                                status: PlatformStatus { 
                                    status: extract_number_field(&event.data, "status.status").unwrap_or(0) 
                                },
                                release_date: extract_string_field(&event.data, "release_date"),
                            };
                            
                            // If platform_id is empty, try other formats
                            if platform_event.platform_id.is_empty() {
                                platform_event.platform_id = event.data.get("platform_id")
                                    .and_then(|v| v.as_str())
                                    .map(String::from)
                                    .unwrap_or_default();
                            }
                            
                            info!("Manually extracted platform event: {:?}", platform_event);
                            self.process_platform_created_event(&platform_event, Some(&event)).await?;
                        }
                    }
                },
                PlatformEventType::PlatformUpdated => {
                    info!("Processing PlatformUpdated event");
                    let platform_event: PlatformUpdatedEvent = serde_json::from_value(event.data.clone())?;
                    self.process_platform_updated_event(&platform_event, Some(&event)).await?;
                },
                PlatformEventType::ModeratorAdded => {
                    info!("Processing ModeratorAdded event");
                    let platform_event: ModeratorAddedEvent = serde_json::from_value(event.data.clone())?;
                    self.process_moderator_added_event(&platform_event, Some(&event)).await?;
                },
                PlatformEventType::ModeratorRemoved => {
                    info!("Processing ModeratorRemoved event");
                    let platform_event: ModeratorRemovedEvent = serde_json::from_value(event.data.clone())?;
                    self.process_moderator_removed_event(&platform_event, Some(&event)).await?;
                },
                PlatformEventType::ProfileBlocked => {
                    info!("Processing ProfileBlocked event");
                    let platform_event: PlatformBlockedProfileEvent = serde_json::from_value(event.data.clone())?;
                    self.process_profile_blocked_event(&platform_event, Some(&event)).await?;
                },
                PlatformEventType::ProfileUnblocked => {
                    info!("Processing ProfileUnblocked event");
                    let platform_event: PlatformUnblockedProfileEvent = serde_json::from_value(event.data.clone())?;
                    self.process_profile_unblocked_event(&platform_event, Some(&event)).await?;
                },
                PlatformEventType::PlatformApprovalChanged => {
                    info!("Processing PlatformApprovalChanged event");
                    let platform_event: PlatformApprovalChangedEvent = serde_json::from_value(event.data.clone())?;
                    self.process_platform_approval_changed_event(&platform_event, Some(&event)).await?;
                },
                PlatformEventType::UserJoinedPlatform => {
                    info!("Processing UserJoinedPlatform event");
                    let platform_event: UserJoinedPlatformEvent = serde_json::from_value(event.data.clone())?;
                    self.process_user_joined_platform_event(&platform_event, Some(&event)).await?;
                },
                PlatformEventType::UserLeftPlatform => {
                    info!("Processing UserLeftPlatform event");
                    let platform_event: UserLeftPlatformEvent = serde_json::from_value(event.data.clone())?;
                    self.process_user_left_platform_event(&platform_event, Some(&event)).await?;
                },
            }
        } else {
            // Check if it contains platform in the event name for debugging
            if event.event_type.to_lowercase().contains("platform") {
                info!("Found potential platform event but type not recognized: {}", event.event_type);
                info!("Event data: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
            }
            debug!("Not a recognized platform event: {}", event.event_type);
        }
        
        Ok(())
    }
    
    /// Start listening for platform events
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting platform event handler");
        
        while let Some(event) = self.rx.recv().await {
            debug!("Received event: {:?}", event.event_type);
            
            if let Err(e) = self.process_event(event).await {
                error!("Error processing event: {}", e);
            }
        }
        
        warn!("Platform event handler channel closed");
        Ok(())
    }
}