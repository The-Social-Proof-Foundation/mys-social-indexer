use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use mys_types::base_types::SuiAddress;

use crate::models::profile::NewProfile;

/// Event emitted when a new profile is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCreatedEvent {
    pub profile_id: String,
    pub owner_address: String,
    pub username: String,
    pub display_name: String,
    pub bio: String,
    pub created_at: u64,
}

impl ProfileCreatedEvent {
    /// Convert the event into a NewProfile model
    pub fn into_model(&self) -> Result<NewProfile> {
        let created_at = DateTime::<Utc>::from_timestamp(self.created_at as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        Ok(NewProfile {
            id: self.profile_id.clone(),
            owner_address: self.owner_address.clone(),
            username: Some(self.username.clone()),
            display_name: Some(self.display_name.clone()),
            bio: Some(self.bio.clone()),
            created_at,
            last_activity_at: Some(created_at),
            followers_count: 0,
            following_count: 0,
            content_count: 0,
            platforms_joined: 0,
        })
    }
}

/// Event emitted when a profile is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdatedEvent {
    pub profile_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub updated_at: u64,
}

/// Event emitted when a profile follows another profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFollowEvent {
    pub follower_id: String,
    pub following_id: String,
    pub followed_at: u64,
}

/// Event emitted when a profile unfollows another profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUnfollowEvent {
    pub follower_id: String,
    pub following_id: String,
    pub unfollowed_at: u64,
}