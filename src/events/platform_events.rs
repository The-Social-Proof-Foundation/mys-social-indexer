use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::models::platform::NewPlatform;

/// Event emitted when a new platform is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCreatedEvent {
    pub platform_id: String,
    pub name: String,
    pub description: String,
    pub creator_address: String,
    pub created_at: u64,
}

impl PlatformCreatedEvent {
    /// Convert the event into a NewPlatform model
    pub fn into_model(&self) -> Result<NewPlatform> {
        let created_at = DateTime::<Utc>::from_timestamp(self.created_at as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        Ok(NewPlatform {
            id: self.platform_id.clone(),
            name: self.name.clone(),
            description: Some(self.description.clone()),
            creator_address: self.creator_address.clone(),
            created_at,
            last_activity_at: Some(created_at),
            active_users_count: 0,
            total_users_count: 0,
            content_count: 0,
        })
    }
}

/// Event emitted when platform settings are updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfigUpdatedEvent {
    pub platform_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub updated_at: u64,
}

/// Event emitted when a profile joins a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileJoinedPlatformEvent {
    pub profile_id: String,
    pub platform_id: String,
    pub joined_at: u64,
}

/// Event emitted when a profile leaves a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileLeftPlatformEvent {
    pub profile_id: String,
    pub platform_id: String,
    pub left_at: u64,
}