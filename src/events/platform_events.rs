// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Platform event types - corresponds to the Move module events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformEventType {
    PlatformCreated,
    PlatformUpdated,
    ModeratorAdded,
    ModeratorRemoved,
    ProfileBlocked,
    ProfileUnblocked,
    PlatformApprovalChanged,
    UserJoinedPlatform,
    UserLeftPlatform,
}

impl PlatformEventType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            s if s.contains("::PlatformCreatedEvent") => Some(Self::PlatformCreated),
            s if s.contains("::PlatformUpdatedEvent") => Some(Self::PlatformUpdated),
            s if s.contains("::ModeratorAddedEvent") => Some(Self::ModeratorAdded),
            s if s.contains("::ModeratorRemovedEvent") => Some(Self::ModeratorRemoved),
            s if s.contains("::PlatformBlockedProfileEvent") => Some(Self::ProfileBlocked),
            s if s.contains("::PlatformUnblockedProfileEvent") => Some(Self::ProfileUnblocked),
            s if s.contains("::PlatformApprovalChangedEvent") => Some(Self::PlatformApprovalChanged),
            s if s.contains("::UserJoinedPlatformEvent") => Some(Self::UserJoinedPlatform),
            s if s.contains("::UserLeftPlatformEvent") => Some(Self::UserLeftPlatform),
            _ => None,
        }
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::PlatformCreated => "PlatformCreatedEvent",
            Self::PlatformUpdated => "PlatformUpdatedEvent",
            Self::ModeratorAdded => "ModeratorAddedEvent",
            Self::ModeratorRemoved => "ModeratorRemovedEvent",
            Self::ProfileBlocked => "PlatformBlockedProfileEvent",
            Self::ProfileUnblocked => "PlatformUnblockedProfileEvent",
            Self::PlatformApprovalChanged => "PlatformApprovalChangedEvent",
            Self::UserJoinedPlatform => "UserJoinedPlatformEvent",
            Self::UserLeftPlatform => "UserLeftPlatformEvent",
        }
    }
}

/// Helper method to extract a platform ID from an event
pub fn extract_platform_id(event_data: &Value) -> Option<String> {
    // Try standard format first
    let platform_id = event_data.get("platform_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    if platform_id.is_some() {
        return platform_id;
    }
    
    // Try blockchain object format with fields.platform_id
    if let Some(fields) = event_data.get("fields") {
        if let Some(platform_id) = fields.get("platform_id") {
            if let Some(id_str) = platform_id.as_str() {
                return Some(id_str.to_string());
            }
        }
    }
    
    // Try content.fields format
    if let Some(content) = event_data.get("content") {
        if let Some(fields) = content.get("fields") {
            if let Some(platform_id) = fields.get("platform_id") {
                if let Some(id_str) = platform_id.as_str() {
                    return Some(id_str.to_string());
                }
            }
        }
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
    tracing::warn!("Failed to extract platform_id from event data: {}", 
        serde_json::to_string_pretty(event_data).unwrap_or_default());
    
    None
}