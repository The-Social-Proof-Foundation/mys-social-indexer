// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde_json;
use tracing::{info, error};
use serde::{Deserialize, Serialize};

use crate::db::DbConnection;
use crate::models::blocking::{
    PlatformBlockedProfileEvent, PlatformUnblockedProfileEvent,
    UserBlockEvent, UserUnblockEvent
};
use crate::schema::profile_events;
use crate::models::profile_events::NewProfileEvent;
use crate::events::profile_event_types::{BlockAddedEvent, BlockRemovedEvent};

/// Event emitted when a BlockList is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockListCreatedEvent {
    /// ID of the BlockList object
    pub block_list_id: String,
    /// Address of the wallet that owns this block list
    pub owner: String,
}

/// Process a profile block event
pub async fn process_profile_block_event(
    conn: &mut DbConnection,
    event_data: &serde_json::Value,
) -> Result<()> {
    // Log the raw event data for debugging
    info!(
        "Processing profile block event (raw data): {:?}",
        event_data
    );
    
    // Try parsing different event formats
    info!("Processing block event");
    
    // Parse the UserBlockEvent 
    let block_event = match serde_json::from_value::<UserBlockEvent>(event_data.clone()) {
        Ok(evt) => {
            info!("Successfully parsed UserBlockEvent: blocker={}, blocked={}", 
                evt.blocker, evt.blocked);
            evt
        },
        Err(e) => {
            info!("Failed to parse UserBlockEvent: {}", e);
            
            // Extract directly from fields
            let empty_map = serde_json::Map::new();
            let obj = event_data.as_object().unwrap_or(&empty_map);
            
            // Try standard "fields" container
            if let Some(fields_obj) = obj.get("fields").and_then(|f| f.as_object()) {
                if let (Some(blocker), Some(blocked)) = (
                    fields_obj.get("blocker").and_then(|v| v.as_str()),
                    fields_obj.get("blocked").and_then(|v| v.as_str())
                ) {
                    info!("Found fields in standard location: blocker={}, blocked={}", blocker, blocked);
                    
                    UserBlockEvent {
                        blocker: blocker.to_string(),
                        blocked: blocked.to_string(),
                    }
                } else {
                    // Try direct object keys
                    let blocker = obj.get("blocker").and_then(|v| v.as_str())
                        .unwrap_or_default().to_string();
                        
                    let blocked = obj.get("blocked").and_then(|v| v.as_str())
                        .unwrap_or_default().to_string();
                    
                    UserBlockEvent { 
                        blocker, 
                        blocked,
                    }
                }
            } else {
                // Try direct object keys
                let blocker = obj.get("blocker").and_then(|v| v.as_str())
                    .unwrap_or_default().to_string();
                    
                let blocked = obj.get("blocked").and_then(|v| v.as_str())
                    .unwrap_or_default().to_string();
                
                UserBlockEvent { 
                    blocker, 
                    blocked,
                }
            }
        }
    };
    
    // Check if all required fields are present
    if block_event.blocker.is_empty() || block_event.blocked.is_empty() {
        info!("Missing required fields in block event, skipping");
        return Ok(());
    }
    
    info!(
        "Processing profile block event: {} blocked {}",
        block_event.blocker, block_event.blocked
    );
    
    // First check if there's already an entry for this block
    use crate::schema::profiles_blocked::dsl::*;
    
    let existing = profiles_blocked
        .filter(blocker_profile_id.eq(&block_event.blocker))
        .filter(blocked_profile_id.eq(&block_event.blocked))
        .first::<crate::models::blocking::ProfileBlock>(conn)
        .await
        .optional()?;
        
    if let Some(_) = existing {
        // Update the existing entry
        diesel::update(profiles_blocked)
            .filter(blocker_profile_id.eq(&block_event.blocker))
            .filter(blocked_profile_id.eq(&block_event.blocked))
            .set((
                is_blocked.eq(true),
                unblocked_at.eq::<Option<chrono::NaiveDateTime>>(None),
            ))
            .execute(conn)
            .await?;
            
        info!("Updated existing block record");
    } else {
        // Insert a new entry
        info!("Attempting to insert new block record with blocker={}, blocked={}", 
            block_event.blocker, block_event.blocked);
            
        // Use profiles_blocked more explicitly to avoid confusion
        use crate::schema::profiles_blocked;
        
        // Build the values to insert
        let values = (
            profiles_blocked::blocker_profile_id.eq(block_event.blocker.clone()),
            profiles_blocked::blocked_profile_id.eq(block_event.blocked.clone()),
            profiles_blocked::created_at.eq(Utc::now().naive_utc()),
            profiles_blocked::is_blocked.eq(true),
        );
        
        // Log the exact SQL values being inserted (for debugging)
        info!("SQL Insert values: blocker_profile_id={}, blocked_profile_id={}, created_at=now(), is_blocked=true", 
            block_event.blocker, block_event.blocked);
            
        let result = diesel::insert_into(profiles_blocked::table)
            .values(values)
            .execute(conn)
            .await;
            
        match result {
            Ok(rows) => {
                info!("Successfully created {} new block record(s)", rows);
            },
            Err(e) => {
                error!("Failed to insert block record: {}", e);
                // Try to determine what went wrong
                if e.to_string().contains("duplicate key") {
                    info!("This appears to be a duplicate key error - record might already exist");
                    
                    // Try to update instead of insert
                    let update_result = diesel::update(profiles_blocked)
                        .filter(blocker_profile_id.eq(&block_event.blocker))
                        .filter(blocked_profile_id.eq(&block_event.blocked))
                        .set((
                            is_blocked.eq(true),
                            unblocked_at.eq::<Option<chrono::NaiveDateTime>>(None),
                        ))
                        .execute(conn)
                        .await;
                        
                    match update_result {
                        Ok(rows) => info!("Updated {} existing block records instead", rows),
                        Err(update_err) => error!("Update fallback also failed: {}", update_err)
                    }
                } else if e.to_string().contains("violates foreign key") {
                    error!("This appears to be a foreign key constraint violation. Check if the profiles exist in the database.");
                } else {
                    // Return the error
                    return Err(anyhow::anyhow!("Database error: {}", e));
                }
            }
        };
        
        // Create a profile_events entry to track in user history
        let block_timestamp = chrono::Utc::now().timestamp() as u64;
        
        // Create block event for profile_events
        let profile_block_event = BlockAddedEvent {
            blocker_profile_id: block_event.blocker.clone(),
            blocked_profile_id: block_event.blocked.clone(),
            timestamp: block_timestamp,
        };
        
        // Create profile event for blocking
        let profile_event = NewProfileEvent::from_block_added(
            &profile_block_event,
            None // No event ID available
        );
        
        // Insert into profile_events
        let result = diesel::insert_into(profile_events::table)
            .values(&profile_event)
            .execute(conn)
            .await;
            
        match result {
            Ok(_) => {
                info!("Successfully created profile_events record for block event");
            },
            Err(e) => {
                error!("Failed to insert block event into profile_events: {}", e);
            }
        }
    }
    
    Ok(())
}

/// Process a profile unblock event
pub async fn process_profile_unblock_event(
    conn: &mut DbConnection,
    event_data: &serde_json::Value,
) -> Result<()> {
    // Log the raw event data for debugging
    info!(
        "Processing profile unblock event (raw data): {:?}",
        event_data
    );
    
    // Try to parse the event data
    let unblock_event = match serde_json::from_value::<UserUnblockEvent>(event_data.clone()) {
        Ok(evt) => {
            info!(
                "Successfully parsed unblock event: blocker={}, unblocked={}",
                evt.blocker, evt.unblocked
            );
            evt
        },
        Err(e) => {
            info!("Failed to parse UserUnblockEvent: {}", e);
            
            // Extract directly from JSON
            let empty_map = serde_json::Map::new();
            let obj = event_data.as_object().unwrap_or(&empty_map);
            
            // Try to extract from fields container
            if let Some(fields_obj) = obj.get("fields").and_then(|f| f.as_object()) {
                // Try to extract blocker and unblocked
                let blocker = fields_obj.get("blocker").and_then(|v| v.as_str())
                    .unwrap_or_default().to_string();
                    
                let unblocked = fields_obj.get("unblocked").and_then(|v| v.as_str())
                    .unwrap_or_default().to_string();
                
                UserUnblockEvent { blocker, unblocked }
            } else {
                // Try root-level fields directly
                let blocker = obj.get("blocker").and_then(|v| v.as_str())
                    .unwrap_or_default().to_string();
                    
                let unblocked = obj.get("unblocked").and_then(|v| v.as_str())
                    .unwrap_or_default().to_string();
                
                UserUnblockEvent { blocker, unblocked }
            }
        }
    };
    
    // Check if all required fields are present
    if unblock_event.blocker.is_empty() || unblock_event.unblocked.is_empty() {
        info!("Missing required fields in unblock event, skipping");
        return Ok(());
    }
    
    info!(
        "Processing profile unblock event: {} unblocked {}",
        unblock_event.blocker, unblock_event.unblocked
    );
    
    // Update the block record to mark as unblocked
    let result = diesel::update(crate::schema::profiles_blocked::table)
        .filter(
            crate::schema::profiles_blocked::blocker_profile_id.eq(unblock_event.blocker.clone())
        )
        .filter(
            crate::schema::profiles_blocked::blocked_profile_id.eq(unblock_event.unblocked.clone())
        )
        .set((
            crate::schema::profiles_blocked::is_blocked.eq(false),
            crate::schema::profiles_blocked::unblocked_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)
        .await;
        
    match result {
        Ok(rows) => {
            info!("Updated {} block records to unblocked status", rows);
            if rows == 0 {
                info!("Note: No rows were updated - the block record may not exist");
            }
            
            // Create a profile_events entry to track in user history
            let unblock_timestamp = chrono::Utc::now().timestamp() as u64;
            
            // Create block removed event for profile_events
            let profile_unblock_event = BlockRemovedEvent {
                blocker_profile_id: unblock_event.blocker.clone(),
                blocked_profile_id: unblock_event.unblocked.clone(),
                timestamp: unblock_timestamp,
            };
            
            // Create profile event for unblocking
            let profile_event = NewProfileEvent::from_block_removed(
                &profile_unblock_event,
                None // No event ID available
            );
            
            // Insert into profile_events
            let event_result = diesel::insert_into(profile_events::table)
                .values(&profile_event)
                .execute(conn)
                .await;
                
            match event_result {
                Ok(_) => {
                    info!("Successfully created profile_events record for unblock event");
                },
                Err(e) => {
                    error!("Failed to insert unblock event into profile_events: {}", e);
                }
            }
        },
        Err(e) => {
            error!("Failed to update profiles_blocked table: {}", e);
            return Err(anyhow::anyhow!("Database error: {}", e));
        }
    }
    
    Ok(())
}

/// Record platform block/unblock events in profile_events instead of using a separate platforms_blocked table
/// This is now handled through the profile_events table for history tracking

/// Process a platform block event - stores in profile_events table instead
pub async fn process_platform_block_event(
    conn: &mut DbConnection,
    event_data: &serde_json::Value,
) -> Result<()> {
    // First log the raw event data to see what's coming from the blockchain
    info!(
        "Processing platform block event (raw data): {:?}",
        event_data
    );
    
    // Try to parse the event data
    let block_event = match serde_json::from_value::<PlatformBlockedProfileEvent>(event_data.clone()) {
        Ok(evt) => {
            info!(
                "Successfully parsed blockchain event: platform_id={}, profile_id={}, blocked_by={}",
                evt.platform_id, evt.profile_id, evt.blocked_by
            );
            evt
        },
        Err(e) => {
            // When parsing fails, try to extract fields directly from the raw event
            info!("Failed to parse event normally, trying direct extraction: {}", e);
            
            // Create an event object using fields directly from the event_data JSON
            let event_platform_id = event_data.get("platform_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default().to_string();
                
            let event_profile_id = event_data.get("profile_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default().to_string();
                
            let event_blocked_by = event_data.get("blocked_by")
                .and_then(|v| v.as_str())
                .unwrap_or_default().to_string();
            
            info!(
                "Manually extracted platform_id={}, profile_id={}, blocked_by={}",
                event_platform_id, event_profile_id, event_blocked_by
            );
            
            PlatformBlockedProfileEvent {
                platform_id: event_platform_id,
                profile_id: event_profile_id,
                blocked_by: event_blocked_by,
            }
        }
    };
    
    // Check if all required fields are present
    if block_event.platform_id.is_empty() || block_event.profile_id.is_empty() || block_event.blocked_by.is_empty() {
        info!("Missing required fields in platform block event, skipping");
        return Ok(());
    }
    
    info!(
        "Processing platform block event: Platform {} blocked profile {} by {}",
        block_event.platform_id, block_event.profile_id, block_event.blocked_by
    );
    
    // Store this in profile_events instead of platforms_blocked
    let block_timestamp = chrono::Utc::now().timestamp() as u64;
    
    // Create record in profile_events - we'll use BlockAdded event type
    // with custom fields for platform blocking
    let profile_event = NewProfileEvent::from_blockchain_event(
        crate::events::profile_event_types::ProfileEventType::BlockAdded,
        block_event.profile_id.clone(),
        serde_json::json!({
            "platform_id": block_event.platform_id,
            "blocked_by": block_event.blocked_by,
            "timestamp": block_timestamp,
            "is_platform_block": true
        }),
        None, // No event ID available
        Some(block_timestamp)
    );
    
    // Insert into profile_events
    let result = diesel::insert_into(crate::schema::profile_events::table)
        .values(&profile_event)
        .execute(conn)
        .await;
        
    match result {
        Ok(_) => {
            info!("Created profile_events record for platform block event");
        },
        Err(e) => {
            error!("Failed to insert platform block event into profile_events: {}", e);
            return Err(anyhow::anyhow!("Database error: {}", e));
        }
    }
    
    Ok(())
}

/// Process a platform unblock event - stores in profile_events table instead
pub async fn process_platform_unblock_event(
    conn: &mut DbConnection,
    event_data: &serde_json::Value,
) -> Result<()> {
    // First log the raw event data to see what's coming from the blockchain
    info!(
        "Processing platform unblock event (raw data): {:?}",
        event_data
    );
    
    // Try to parse the event data
    let unblock_event = match serde_json::from_value::<PlatformUnblockedProfileEvent>(event_data.clone()) {
        Ok(evt) => {
            info!(
                "Successfully parsed blockchain event: platform_id={}, profile_id={}, unblocked_by={}",
                evt.platform_id, evt.profile_id, evt.unblocked_by
            );
            evt
        },
        Err(e) => {
            // When parsing fails, try to extract fields directly from the raw event
            info!("Failed to parse event normally, trying direct extraction: {}", e);
            
            // Create an event object using fields directly from the event_data JSON
            let event_platform_id = event_data.get("platform_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default().to_string();
                
            let event_profile_id = event_data.get("profile_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default().to_string();
                
            let event_unblocked_by = event_data.get("unblocked_by")
                .and_then(|v| v.as_str())
                .unwrap_or_default().to_string();
            
            info!(
                "Manually extracted platform_id={}, profile_id={}, unblocked_by={}",
                event_platform_id, event_profile_id, event_unblocked_by
            );
            
            PlatformUnblockedProfileEvent {
                platform_id: event_platform_id,
                profile_id: event_profile_id,
                unblocked_by: event_unblocked_by,
            }
        }
    };
    
    // Check if all required fields are present
    if unblock_event.platform_id.is_empty() || unblock_event.profile_id.is_empty() {
        info!("Missing required fields in platform unblock event, skipping");
        return Ok(());
    }
    
    info!(
        "Processing platform unblock event: Platform {} unblocked profile {}",
        unblock_event.platform_id, unblock_event.profile_id
    );
    
    // Store this in profile_events instead of platforms_blocked
    let unblock_timestamp = chrono::Utc::now().timestamp() as u64;
    
    // Create record in profile_events - we'll use BlockRemoved event type
    // with custom fields for platform unblocking
    let profile_event = NewProfileEvent::from_blockchain_event(
        crate::events::profile_event_types::ProfileEventType::BlockRemoved,
        unblock_event.profile_id.clone(),
        serde_json::json!({
            "platform_id": unblock_event.platform_id,
            "unblocked_by": unblock_event.unblocked_by,
            "timestamp": unblock_timestamp,
            "is_platform_block": true
        }),
        None, // No event ID available
        Some(unblock_timestamp)
    );
    
    // Insert into profile_events
    let result = diesel::insert_into(crate::schema::profile_events::table)
        .values(&profile_event)
        .execute(conn)
        .await;
        
    match result {
        Ok(_) => {
            info!("Created profile_events record for platform unblock event");
        },
        Err(e) => {
            error!("Failed to insert platform unblock event into profile_events: {}", e);
            return Err(anyhow::anyhow!("Database error: {}", e));
        }
    }
    
    Ok(())
} 

/// Process a block list created event
pub async fn process_block_list_created_event(
    conn: &mut DbConnection,
    event_data: &serde_json::Value,
) -> Result<()> {
    // Log the raw event data
    info!(
        "Processing block list created event (raw data): {:?}",
        event_data
    );
    
    // Try to parse the event data with more thorough field extraction
    let block_list_event = match serde_json::from_value::<BlockListCreatedEvent>(event_data.clone()) {
        Ok(evt) => {
            info!(
                "Successfully parsed blockchain event: block_list_id={}, owner={}",
                evt.block_list_id, evt.owner
            );
            evt
        },
        Err(e) => {
            // When parsing fails, try to extract fields directly from the raw event
            info!("Failed to parse event normally, trying direct extraction: {}", e);
            
            // Create a longer-lived Map
            let empty_map = serde_json::Map::new();
            
            // Try to extract from root or fields container
            let obj = event_data.as_object().unwrap_or(&empty_map);
            
            // Look for fields container in Move event structure
            let fields = if let Some(fields) = obj.get("fields").and_then(|f| f.as_object()) {
                fields
            } else {
                obj
            };
            
            // Create an event object using fields directly from the event_data JSON
            let block_list_id = fields.get("block_list_id")
                .and_then(|v| v.as_str())
                .or_else(|| fields.get("id").and_then(|v| v.as_str()))
                .or_else(|| obj.get("block_list_id").and_then(|v| v.as_str()))
                .or_else(|| obj.get("id").and_then(|v| v.as_str()))
                .unwrap_or_default().to_string();
                
            let owner = fields.get("owner")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("owner").and_then(|v| v.as_str()))
                .unwrap_or_default().to_string();
            
            info!(
                "Manually extracted block_list_id={}, owner={}",
                block_list_id, owner
            );
            
            BlockListCreatedEvent {
                block_list_id,
                owner,
            }
        }
    };
    
    // Check if all required fields are present
    if block_list_event.block_list_id.is_empty() || block_list_event.owner.is_empty() {
        info!("Missing required fields in block list created event, skipping");
        return Ok(());
    }
    
    info!(
        "Processing block list created event: BlockList {} created for owner {}",
        block_list_event.block_list_id, block_list_event.owner
    );
    
    // Update the profile to set the block list address
    use crate::schema::profiles;
    use crate::models::profile::UpdateProfile;
    
    // First, log the current profile in the database
    match diesel::dsl::select(
        diesel::dsl::exists(
            profiles::table
                .filter(profiles::owner_address.eq(&block_list_event.owner))
        )
    )
    .get_result::<bool>(conn)
    .await {
        Ok(exists) => {
            info!("Profile with owner_address {} exists in database: {}", block_list_event.owner, exists);
            
            if !exists {
                info!("Could not find profile with owner_address {}, cannot update", block_list_event.owner);
                return Ok(());
            }
            
            // If we found a profile by owner_address, update it
            info!("Updating profile with owner_address {}", block_list_event.owner);
            
            let update = UpdateProfile {
                display_name: None,
                bio: None,
                profile_photo: None,
                website: None,
                cover_photo: None,
                sensitive_data_updated_at: None,
                followers_count: None,
                following_count: None,
                birthdate: None,
                current_location: None,
                raised_location: None,
                phone: None,
                email: None,
                gender: None,
                political_view: None,
                religion: None,
                education: None,
                primary_language: None,
                relationship_status: None,
                x_username: None,
                mastodon_username: None,
                facebook_username: None,
                reddit_username: None,
                github_username: None,
                block_list_address: Some(block_list_event.block_list_id.clone()),
            };
            
            diesel::update(profiles::table)
                .filter(profiles::owner_address.eq(&block_list_event.owner))
                .set(&update)
                .execute(conn)
                .await?;
            
            info!(
                "Updated profile with owner_address {} with block list address {}",
                block_list_event.owner, block_list_event.block_list_id
            );
        },
        Err(e) => {
            info!("Error checking if profile exists: {}", e);
        }
    }
    
    Ok(())
}