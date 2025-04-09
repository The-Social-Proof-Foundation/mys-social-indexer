// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Profile event types - corresponds to the Move module events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileEventType {
    ProfileCreated,
    ProfileUpdated,
    ProfileTransferred,
    ServiceAuthorized,
    ServiceRevoked,
    // User blocks another user
    BlockAdded,
    // User unblocks another user
    BlockRemoved,
    // User joins a platform
    PlatformJoined,
    // User leaves a platform
    PlatformLeft,
}

impl ProfileEventType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            s if s.contains("::ProfileCreatedEvent") => Some(Self::ProfileCreated),
            s if s.contains("::ProfileUpdatedEvent") => Some(Self::ProfileUpdated),
            s if s.contains("::ProfileTransferredEvent") => Some(Self::ProfileTransferred),
            s if s.contains("::ServiceAuthorizedEvent") => Some(Self::ServiceAuthorized),
            s if s.contains("::ServiceRevokedEvent") => Some(Self::ServiceRevoked),
            s if s.contains("::BlockAddedEvent") || s.contains("::UserBlockEvent") => Some(Self::BlockAdded),
            s if s.contains("::BlockRemovedEvent") || s.contains("::UserUnblockEvent") => Some(Self::BlockRemoved),
            s if s.contains("::UserJoinedPlatformEvent") || s.contains("::PlatformJoinedEvent") => Some(Self::PlatformJoined),
            s if s.contains("::UserLeftPlatformEvent") || s.contains("::PlatformLeftEvent") => Some(Self::PlatformLeft),
            _ => None,
        }
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::ProfileCreated => "ProfileCreatedEvent",
            Self::ProfileUpdated => "ProfileUpdatedEvent",
            Self::ProfileTransferred => "ProfileTransferredEvent",
            Self::ServiceAuthorized => "ServiceAuthorizedEvent",
            Self::ServiceRevoked => "ServiceRevokedEvent",
            Self::BlockAdded => "BlockAddedEvent",
            Self::BlockRemoved => "BlockRemovedEvent",
            Self::PlatformJoined => "PlatformJoinedEvent",
            Self::PlatformLeft => "PlatformLeftEvent",
        }
    }
}

impl From<ProfileEventType> for String {
    fn from(event_type: ProfileEventType) -> Self {
        event_type.to_str().to_string()
    }
}

/// Helper method to extract a profile ID from an event
pub fn extract_profile_id(event_data: &Value) -> Option<String> {
    // Try standard format first
    let profile_id = event_data.get("profile_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    if profile_id.is_some() {
        return profile_id;
    }
    
    // Try blockchain object format with fields.profile_id
    if let Some(fields) = event_data.get("fields") {
        if let Some(profile_id) = fields.get("profile_id") {
            if let Some(id_str) = profile_id.as_str() {
                return Some(id_str.to_string());
            }
        }
    }
    
    // Try content.fields format
    if let Some(content) = event_data.get("content") {
        if let Some(fields) = content.get("fields") {
            if let Some(profile_id) = fields.get("profile_id") {
                if let Some(id_str) = profile_id.as_str() {
                    return Some(id_str.to_string());
                }
            }
        }
    }
    
    // Try with blocker_profile_id for block events
    let blocker_id = event_data.get("blocker_profile_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    if blocker_id.is_some() {
        return blocker_id;
    }
    
    // Try array/tuple formats that might be in the move structure
    if let Some(array) = event_data.as_array() {
        if !array.is_empty() {
            if let Some(id_str) = array[0].as_str() {
                return Some(id_str.to_string());
            }
        }
    }
    
    // Log failure for debugging
    tracing::warn!("Failed to extract profile_id from event data: {}", 
        serde_json::to_string_pretty(event_data).unwrap_or_default());
    
    None
}

// Block event definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockAddedEvent {
    pub blocker_profile_id: String,
    pub blocked_profile_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockRemovedEvent {
    pub blocker_profile_id: String,
    pub blocked_profile_id: String,
    pub timestamp: u64,
}

// Platform membership event definitions
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformJoinedEvent {
    pub profile_id: String,
    pub platform_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformLeftEvent {
    pub profile_id: String,
    pub platform_id: String,
    pub timestamp: u64,
}