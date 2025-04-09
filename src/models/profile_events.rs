// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::profile_events;
use crate::events::profile_event_types::{ProfileEventType, BlockAddedEvent, BlockRemovedEvent, PlatformJoinedEvent, PlatformLeftEvent};

/// Profile event model for database storage
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = profile_events)]
pub struct ProfileEvent {
    pub id: i32,
    pub event_type: String,
    pub profile_id: String,
    pub event_data: serde_json::Value,
    pub event_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// DTO for inserting a new profile event
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = profile_events)]
pub struct NewProfileEvent {
    pub event_type: String,
    pub profile_id: String,
    pub event_data: serde_json::Value,
    pub event_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl NewProfileEvent {
    /// Create a new profile event from a blockchain event
    pub fn from_blockchain_event(
        event_type: impl Into<String>,
        profile_id: String,
        event_data: serde_json::Value,
        event_id: Option<String>,
        timestamp: Option<u64>,
    ) -> Self {
        let now = chrono::Utc::now().naive_utc();
        let created_at = if let Some(ts) = timestamp {
            chrono::DateTime::from_timestamp(ts as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .naive_utc()
        } else {
            now
        };
        
        Self {
            event_type: event_type.into(),
            profile_id,
            event_data,
            event_id,
            created_at,
            updated_at: now,
        }
    }
    
    /// Create a new profile event for a block added event
    pub fn from_block_added(event: &BlockAddedEvent, event_id: Option<String>) -> Self {
        let now = chrono::Utc::now().naive_utc();
        let created_at = chrono::DateTime::from_timestamp(event.timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .naive_utc();
        
        Self {
            event_type: ProfileEventType::BlockAdded.to_str().to_string(),
            profile_id: event.blocker_profile_id.clone(),
            event_data: serde_json::json!({
                "blocker_profile_id": event.blocker_profile_id,
                "blocked_profile_id": event.blocked_profile_id,
                "timestamp": event.timestamp,
                "is_platform_block": false
            }),
            event_id,
            created_at,
            updated_at: now,
        }
    }
    
    /// Create a new profile event for a block removed event
    pub fn from_block_removed(event: &BlockRemovedEvent, event_id: Option<String>) -> Self {
        let now = chrono::Utc::now().naive_utc();
        let created_at = chrono::DateTime::from_timestamp(event.timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .naive_utc();
        
        Self {
            event_type: ProfileEventType::BlockRemoved.to_str().to_string(),
            profile_id: event.blocker_profile_id.clone(),
            event_data: serde_json::json!({
                "blocker_profile_id": event.blocker_profile_id,
                "blocked_profile_id": event.blocked_profile_id,
                "timestamp": event.timestamp,
                "is_platform_block": false
            }),
            event_id,
            created_at,
            updated_at: now,
        }
    }
    
    /// Create a new profile event for a platform joined event
    pub fn from_platform_joined(event: &PlatformJoinedEvent, event_id: Option<String>) -> Self {
        let now = chrono::Utc::now().naive_utc();
        let created_at = chrono::DateTime::from_timestamp(event.timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .naive_utc();
        
        Self {
            event_type: ProfileEventType::PlatformJoined.to_str().to_string(),
            profile_id: event.profile_id.clone(),
            event_data: serde_json::json!({
                "profile_id": event.profile_id,
                "platform_id": event.platform_id,
                "timestamp": event.timestamp
            }),
            event_id,
            created_at,
            updated_at: now,
        }
    }
    
    /// Create a new profile event for a platform left event
    pub fn from_platform_left(event: &PlatformLeftEvent, event_id: Option<String>) -> Self {
        let now = chrono::Utc::now().naive_utc();
        let created_at = chrono::DateTime::from_timestamp(event.timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .naive_utc();
        
        Self {
            event_type: ProfileEventType::PlatformLeft.to_str().to_string(),
            profile_id: event.profile_id.clone(),
            event_data: serde_json::json!({
                "profile_id": event.profile_id,
                "platform_id": event.platform_id,
                "timestamp": event.timestamp
            }),
            event_id,
            created_at,
            updated_at: now,
        }
    }
}

/// DTO for updating a profile event
#[derive(Debug, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = profile_events)]
pub struct UpdateProfileEvent {
    pub event_type: Option<String>,
    pub event_data: Option<serde_json::Value>,
    pub updated_at: NaiveDateTime,
}