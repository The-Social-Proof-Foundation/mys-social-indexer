// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::profile::NewProfile;

/// Event emitted when a profile is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCreatedEvent {
    /// ID of the profile
    #[serde(rename = "profile_id", default)]
    pub profile_id: String,
    /// Owner's address
    #[serde(rename = "owner_address", alias = "owner", default)]
    pub owner_address: String,
    /// Username
    #[serde(default)]
    pub username: String,
    /// Display name
    #[serde(rename = "display_name", default)]
    pub display_name: Option<String>,
    /// Bio
    #[serde(default)]
    pub bio: Option<String>,
    /// Profile picture URL
    #[serde(rename = "profile_picture", alias = "avatar_url", default)]
    pub profile_picture: Option<String>,
    /// Cover photo URL
    #[serde(rename = "cover_photo", alias = "cover_url", default)]
    pub cover_photo: Option<String>,
    /// Timestamp of profile creation
    #[serde(rename = "created_at", default = "default_timestamp")]
    pub created_at: u64,
}

fn default_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl ProfileCreatedEvent {
    /// Convert the event to a database model
    pub fn into_model(&self) -> Result<NewProfile> {
        // Convert timestamp to NaiveDateTime
        let created_at = DateTime::from_timestamp(self.created_at as i64, 0)
            .unwrap_or(Utc::now())
            .naive_utc();
        
        Ok(NewProfile {
            owner_address: self.owner_address.clone(),
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            bio: self.bio.clone(),
            avatar_url: self.profile_picture.clone(),
            website_url: None, // Not provided in event
            created_at,
            updated_at: created_at,
        })
    }
}

/// Event emitted when a profile follows another profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFollowEvent {
    /// ID of the follower profile
    pub follower_id: String,
    /// ID of the profile being followed
    pub following_id: String,
    /// Timestamp of the follow action
    pub followed_at: Option<u64>,
}

/// Event emitted when a profile joins a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileJoinedPlatformEvent {
    /// ID of the profile
    pub profile_id: String,
    /// ID of the platform
    pub platform_id: String,
    /// Timestamp of the join action
    pub joined_at: Option<u64>,
}