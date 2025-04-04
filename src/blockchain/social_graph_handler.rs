// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn, trace};

use crate::db::{Database, DbConnection};
use crate::events::{FollowEvent, UnfollowEvent};
use crate::schema;

use super::listener::BlockchainEvent;

/// Handlers for social graph related events
pub struct SocialGraphEventHandler {
    /// Database connection
    db: Arc<Database>,
    /// Event receiver channel
    rx: mpsc::Receiver<BlockchainEvent>,
}

impl SocialGraphEventHandler {
    /// Create a new social graph event handler
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
    
    /// Process a follow event - creates relationship and updates follow counts
    async fn process_follow_event(&self, event: &FollowEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing follow event details");
        
        let mut conn = self.get_connection().await?;
        
        // We always record the event in social_graph_events table, regardless of relationship status
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Create a social graph event record for history/auditing - we ALWAYS create this
                // even if the relationship can't be created yet
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                let social_graph_event = crate::models::social_graph::NewSocialGraphEvent {
                    event_type: "follow".to_string(),
                    follower_address: event.follower.clone(),
                    following_address: event.following.clone(),
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                    event_id,  // Use the event_id from blockchain
                    raw_event_data: serde_json::to_value(event).ok(), // Store original event
                };
                
                // Always insert the event record, no matter what happens with the relationship
                diesel::insert_into(schema::social_graph_events::table)
                    .values(&social_graph_event)
                    .execute(&mut conn)
                    .await?;
                
                // Now check if both profiles exist before creating a relationship
                debug!("Verifying profiles exist by profile_id");
                let follower_profile_exists = schema::profiles::table
                    .filter(schema::profiles::profile_id.eq(&event.follower))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if !follower_profile_exists {
                    info!("Follower profile not found: {}", event.follower);
                    // Still return Ok() since we've recorded the event
                    return Ok(());
                }
                
                let following_profile_exists = schema::profiles::table
                    .filter(schema::profiles::profile_id.eq(&event.following))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await
                    .unwrap_or(0) > 0;
                
                if !following_profile_exists {
                    info!("Following profile not found: {}", event.following);
                    // Still return Ok() since we've recorded the event
                    return Ok(());
                }
                
                // Check if relationship already exists
                let existing = schema::social_graph_relationships::table
                    .filter(schema::social_graph_relationships::follower_address.eq(&event.follower))
                    .filter(schema::social_graph_relationships::following_address.eq(&event.following))
                    .count()
                    .get_result::<i64>(&mut conn)
                    .await?;
                
                if existing > 0 {
                    debug!("Follow relationship already exists - ignoring");
                    return Ok(());
                }
                
                // Create relationship record
                let relationship = match event.into_relationship() {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Failed to create relationship: {}", e);
                        return Err(diesel::result::Error::RollbackTransaction);
                    }
                };
                
                // First, look up the owner_address for both follower and following profile IDs
                let follower_owner = schema::profiles::table
                    .filter(schema::profiles::profile_id.eq(&relationship.follower_address))
                    .select(schema::profiles::owner_address)
                    .first::<String>(&mut conn)
                    .await;
                
                let following_owner = schema::profiles::table
                    .filter(schema::profiles::profile_id.eq(&relationship.following_address))
                    .select(schema::profiles::owner_address)
                    .first::<String>(&mut conn)
                    .await;
                
                // Log for debugging at trace level only
                debug!("Verified profile ID mapping for follow event");
                
                // Continue only if we found both owner addresses
                if let (Ok(_follower_owner), Ok(_following_owner)) = (&follower_owner, &following_owner) {
                    // Insert relationship - using standard Diesel query DSL to ensure proper escaping
                    diesel::insert_into(schema::social_graph_relationships::table)
                        .values(&relationship)
                        .on_conflict((
                            schema::social_graph_relationships::follower_address, 
                            schema::social_graph_relationships::following_address
                        ))
                        .do_nothing()
                        .execute(&mut conn)
                        .await?;
                    
                    // Force recalculate the counts for the affected profiles based on actual relationships
                    diesel::sql_query(
                        "UPDATE profiles 
                         SET following_count = (
                             SELECT COUNT(*) FROM social_graph_relationships 
                             WHERE follower_address = $1
                         )
                         WHERE profile_id = $1"
                    )
                    .bind::<diesel::sql_types::Text, _>(&relationship.follower_address)
                    .execute(&mut conn)
                    .await?;
                    
                    diesel::sql_query(
                        "UPDATE profiles 
                         SET followers_count = (
                             SELECT COUNT(*) FROM social_graph_relationships 
                             WHERE following_address = $1
                         )
                         WHERE profile_id = $1"
                    )
                    .bind::<diesel::sql_types::Text, _>(&relationship.following_address)
                    .execute(&mut conn)
                    .await?;
                } else {
                    debug!("One or both profiles not found in the database, skipping relationship");
                }
                
                // Log success message with simpler format
                debug!("Successfully updated follow relationship and counts.");
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed follow event");
            
        Ok(())
    }
    
    /// Process an unfollow event - removes relationship and updates follow counts
    async fn process_unfollow_event(&self, event: &UnfollowEvent, blockchain_event: Option<&BlockchainEvent>) -> Result<()> {
        debug!("Processing unfollow event details");
        
        let mut conn = self.get_connection().await?;
        
        // We always record the event in social_graph_events table, regardless of relationship status
        // Start a transaction for atomicity
        conn.build_transaction()
            .run(|mut conn| Box::pin(async move {
                // Create a social graph event record for history/auditing - we ALWAYS create this
                // even if there's no relationship to delete
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                
                // Get event_id from blockchain_event if available
                let event_id = blockchain_event.map(|e| e.event_id.clone());
                
                let social_graph_event = crate::models::social_graph::NewSocialGraphEvent {
                    event_type: "unfollow".to_string(),
                    follower_address: event.follower.clone(),
                    following_address: event.unfollowed.clone(),
                    created_at: chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
                        .unwrap_or_else(|| chrono::Utc::now())
                        .naive_utc(),
                    event_id,  // Use the event_id from blockchain
                    raw_event_data: serde_json::to_value(event).ok(),
                };
                
                // Always insert the event record, no matter what
                diesel::insert_into(schema::social_graph_events::table)
                    .values(&social_graph_event)
                    .execute(&mut conn)
                    .await?;
                
                // Check if relationship exists
                let relationship = schema::social_graph_relationships::table
                    .filter(schema::social_graph_relationships::follower_address.eq(&event.follower))
                    .filter(schema::social_graph_relationships::following_address.eq(&event.unfollowed))
                    .select(schema::social_graph_relationships::id)
                    .first::<i32>(&mut conn)
                    .await;
                
                // If the relationship doesn't exist, still return Ok since we've recorded the event
                if let Err(diesel::result::Error::NotFound) = relationship {
                    debug!("Follow relationship does not exist for unfollow - ignoring");
                    return Ok(());
                }
                
                let relationship_id = relationship?;
                
                // Store follower and following addresses for counter updates
                let (follower_address, following_address) = {
                    let rel = schema::social_graph_relationships::table
                        .filter(schema::social_graph_relationships::id.eq(relationship_id))
                        .select((
                            schema::social_graph_relationships::follower_address, 
                            schema::social_graph_relationships::following_address
                        ))
                        .first::<(String, String)>(&mut conn)
                        .await?;
                    (rel.0, rel.1)
                };
                
                // First, look up the owner_address for both follower and following profile IDs
                let follower_owner = schema::profiles::table
                    .filter(schema::profiles::profile_id.eq(&follower_address))
                    .select(schema::profiles::owner_address)
                    .first::<String>(&mut conn)
                    .await;
                
                let following_owner = schema::profiles::table
                    .filter(schema::profiles::profile_id.eq(&following_address))
                    .select(schema::profiles::owner_address)
                    .first::<String>(&mut conn)
                    .await;
                
                // Log for debugging at trace level only
                debug!("Verified profile ID mapping for unfollow event");
                
                // Continue only if we found both owner addresses
                if let (Ok(_follower_owner), Ok(_following_owner)) = (&follower_owner, &following_owner) {
                    // Delete the relationship using proper Diesel delete with DSL
                    let deleted = diesel::delete(
                        schema::social_graph_relationships::table
                            .filter(schema::social_graph_relationships::follower_address.eq(&follower_address))
                            .filter(schema::social_graph_relationships::following_address.eq(&following_address))
                    )
                    .execute(&mut conn)
                    .await?;
                    
                    debug!("Deleted relationship, rows affected: {}", deleted);
                    
                    // Force recalculate the counts for the affected profiles based on actual relationships
                    diesel::sql_query(
                        "UPDATE profiles 
                         SET following_count = (
                             SELECT COUNT(*) FROM social_graph_relationships 
                             WHERE follower_address = $1
                         )
                         WHERE profile_id = $1"
                    )
                    .bind::<diesel::sql_types::Text, _>(&follower_address)
                    .execute(&mut conn)
                    .await?;
                    
                    diesel::sql_query(
                        "UPDATE profiles 
                         SET followers_count = (
                             SELECT COUNT(*) FROM social_graph_relationships 
                             WHERE following_address = $1
                         )
                         WHERE profile_id = $1"
                    )
                    .bind::<diesel::sql_types::Text, _>(&following_address)
                    .execute(&mut conn)
                    .await?;
                } else {
                    debug!("One or both profiles not found in the database, skipping unfollow");
                }
                
                // Log success message with simpler format
                debug!("Successfully updated unfollow relationship and counts.");
                
                Result::<_, diesel::result::Error>::Ok(())
            }))
            .await?;
            
        info!("Successfully processed unfollow event");
            
        Ok(())
    }
    
    /// Process raw blockchain events
    async fn process_event(&self, event: BlockchainEvent) -> Result<()> {
        debug!("Social graph handler examining event: {}", event.event_type);
        debug!("Event ID: {}", event.event_id);
        
        // Log the full event data at trace level only
        trace!("Event data: {}", serde_json::to_string_pretty(&event.data).unwrap_or_default());
        
        if event.event_type.contains("::social_graph::") || 
           event.event_type.contains("::FollowEvent") || 
           event.event_type.contains("::UnfollowEvent") {
            info!("Processing social graph event: {}", event.event_type);
            
            if event.event_type.ends_with("::FollowEvent") {
                match crate::events::parse_event::<FollowEvent>(&event.data) {
                    Ok(follow_event) => {
                        info!("Processing follow: {} -> {}", &follow_event.follower, &follow_event.following);
                        if let Err(e) = self.process_follow_event(&follow_event, Some(&event)).await {
                            error!("Failed to process follow event: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse follow event: {}", e);
                    }
                }
            } else if event.event_type.ends_with("::UnfollowEvent") {
                match crate::events::parse_event::<UnfollowEvent>(&event.data) {
                    Ok(unfollow_event) => {
                        info!("Processing unfollow: {} -> {}", &unfollow_event.follower, &unfollow_event.unfollowed);
                        if let Err(e) = self.process_unfollow_event(&unfollow_event, Some(&event)).await {
                            error!("Failed to process unfollow event: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse unfollow event: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Start listening for social graph events
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting social graph event handler");
        
        while let Some(event) = self.rx.recv().await {
            debug!("Received event: {:?}", event);
            
            if let Err(e) = self.process_event(event).await {
                error!("Error processing event: {}", e);
            }
        }
        
        warn!("Social graph event handler channel closed");
        Ok(())
    }
}