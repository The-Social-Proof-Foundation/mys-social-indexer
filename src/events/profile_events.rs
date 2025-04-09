// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Deserializer};
use std::str::FromStr;

use crate::models::profile::NewProfile;

/// Helper function to deserialize strings as numbers
fn deserialize_number_from_string<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr + Deserialize<'de>,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    // This will handle both string and numeric inputs
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber<T> {
        String(String),
        Number(T),
    }

    match StringOrNumber::<T>::deserialize(deserializer) {
        Ok(StringOrNumber::String(s)) => {
            T::from_str(&s).map_err(serde::de::Error::custom)
        }
        Ok(StringOrNumber::Number(n)) => Ok(n),
        Err(e) => Err(e),
    }
}

/// Helper function to deserialize optional strings as optional numbers
fn deserialize_optional_number_from_string<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr + Deserialize<'de>,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    // This will handle both string and numeric inputs, and None values
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumberOrNone<T> {
        String(String),
        Number(T),
        None,
    }

    match StringOrNumberOrNone::<T>::deserialize(deserializer) {
        Ok(StringOrNumberOrNone::String(s)) => {
            if s.is_empty() {
                Ok(None)
            } else {
                match T::from_str(&s) {
                    Ok(val) => Ok(Some(val)),
                    Err(e) => Err(serde::de::Error::custom(e)),
                }
            }
        }
        Ok(StringOrNumberOrNone::Number(n)) => Ok(Some(n)),
        Ok(StringOrNumberOrNone::None) => Ok(None),
        Err(_) => Ok(None), // Treat errors as None
    }
}

/// Helper function for default timestamp
fn default_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Event emitted when a profile is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCreatedEvent {
    /// ID of the profile - can come from multiple sources
    #[serde(rename = "profile_id", alias = "id", default)]
    pub profile_id: String,
    
    /// Owner's address - can be 'owner' or 'owner_address' in the event
    #[serde(rename = "owner_address", alias = "owner", default)]
    pub owner_address: String,
    
    /// Username - may not be present in the event
    #[serde(default)]
    pub username: Option<String>,
    
    /// Display name
    #[serde(rename = "display_name", default)]
    pub display_name: String,
    
    /// Profile photo URL - can come from multiple fields
    #[serde(rename = "profile_photo", alias = "profile_picture", alias = "avatar_url", default)]
    pub profile_photo: Option<String>,
    
    /// Cover photo URL
    #[serde(rename = "cover_photo", alias = "cover_url", default)]
    pub cover_photo: Option<String>,
    
    /// Bio - may be a string directly in the event
    #[serde(default)]
    pub bio: Option<String>,
    
    /// Timestamp of profile creation
    #[serde(rename = "created_at", default = "default_timestamp", deserialize_with = "deserialize_number_from_string")]
    pub created_at: u64,
}

impl ProfileCreatedEvent {
    /// Convert the event to a database model
    pub fn into_model(&self) -> Result<NewProfile> {
        // Convert timestamp to datetime format for database
        let created_at = DateTime::from_timestamp(self.created_at as i64, 0)
            .unwrap_or(Utc::now())
            .naive_utc();
        
        // Use username if available, otherwise generate a placeholder
        let username = match &self.username {
            Some(name) => name.clone(),
            None => format!("user_{}", self.owner_address.chars().take(8).collect::<String>())
        };
        
        // Log all fields for debugging
        tracing::info!("Converting ProfileCreatedEvent to database model:");
        tracing::info!("  profile_id: {}", self.profile_id);
        tracing::info!("  username: {}", username);
        tracing::info!("  display_name: {}", self.display_name);
        tracing::info!("  bio: {:?}", self.bio);
        tracing::info!("  profile_photo: {:?}", self.profile_photo);
        tracing::info!("  cover_photo: {:?}", self.cover_photo);
        
        // Always use the profile photo if it exists
        let profile_photo = self.profile_photo.clone();
        
        // Always use the cover photo if it exists
        let cover_photo = self.cover_photo.clone();
        
        Ok(NewProfile {
            owner_address: self.owner_address.clone(),
            username,
            display_name: Some(self.display_name.clone()),
            bio: self.bio.clone(),
            profile_photo,
            website: None,     // Not provided in profile creation event
            created_at,
            updated_at: created_at,
            cover_photo,
            profile_id: Some(self.profile_id.clone()),
            sensitive_data_updated_at: None, // Will be set when sensitive data is added
            // Initialize follower/following counts to 0
            followers_count: 0,
            following_count: 0,
            // Initialize all sensitive fields as None
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
            // BlockList object address - will be set when the BlockListCreatedEvent is received
            block_list_address: None,
        })
    }
}

/// Updated ProfileUpdatedEvent with all profile fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdatedEvent {
    /// ID of the profile
    #[serde(rename = "profile_id", alias = "id", default)]
    pub profile_id: String,
    
    /// Display name
    #[serde(rename = "display_name", default)]
    pub display_name: Option<String>,
    
    /// Username
    #[serde(default)]
    pub username: Option<String>,
    
    /// Owner's address
    #[serde(rename = "owner_address", alias = "owner", default)]
    pub owner_address: String,
    
    /// Profile photo URL
    #[serde(rename = "profile_photo", alias = "profile_picture", alias = "avatar_url", default)]
    pub profile_photo: Option<String>,
    
    /// Cover photo URL
    #[serde(rename = "cover_photo", alias = "cover_url", default)]
    pub cover_photo: Option<String>,
    
    /// Bio
    #[serde(rename = "bio", alias = "description", default)]
    pub bio: Option<String>,
    
    /// Update timestamp
    #[serde(rename = "updated_at", default = "default_timestamp", deserialize_with = "deserialize_number_from_string")]
    pub updated_at: u64,
    
    // All sensitive fields that are client-side encrypted
    #[serde(default)]
    pub birthdate: Option<String>,
    
    #[serde(default)]
    pub current_location: Option<String>,
    
    #[serde(default)]
    pub raised_location: Option<String>,
    
    #[serde(default)]
    pub phone: Option<String>,
    
    #[serde(default)]
    pub email: Option<String>,
    
    #[serde(default)]
    pub gender: Option<String>,
    
    #[serde(default)]
    pub political_view: Option<String>,
    
    #[serde(default)]
    pub religion: Option<String>,
    
    #[serde(default)]
    pub education: Option<String>,
    
    #[serde(default)]
    pub primary_language: Option<String>,
    
    #[serde(default)]
    pub relationship_status: Option<String>,
    
    #[serde(default)]
    pub x_username: Option<String>,
    
    #[serde(default)]
    pub mastodon_username: Option<String>,
    
    #[serde(default)]
    pub facebook_username: Option<String>,
    
    #[serde(default)]
    pub reddit_username: Option<String>,
    
    #[serde(default)]
    pub github_username: Option<String>,
}

/// Event emitted when a username is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameUpdatedEvent {
    /// ID of the profile
    #[serde(rename = "profile_id", default)]
    pub profile_id: String,
    /// Old username
    #[serde(rename = "old_username", default)]
    pub old_username: String,
    /// New username
    #[serde(rename = "new_username", default)]
    pub new_username: String,
    /// Owner's address
    #[serde(rename = "owner_address", alias = "owner", default)]
    pub owner_address: String,
}

/// Event emitted when a username is registered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsernameRegisteredEvent {
    /// ID of the profile
    #[serde(rename = "profile_id", default)]
    pub profile_id: String,
    /// Username
    #[serde(default)]
    pub username: String,
    /// Owner's address
    #[serde(rename = "owner_address", alias = "owner", default)]
    pub owner_address: String,
    /// Expiration timestamp
    #[serde(rename = "expires_at", default, deserialize_with = "deserialize_number_from_string")]
    pub expires_at: u64,
    /// Registration timestamp
    #[serde(rename = "registered_at", default, deserialize_with = "deserialize_number_from_string")]
    pub registered_at: u64,
}

/// Event emitted when a profile follows another profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFollowEvent {
    /// ID of the follower profile
    pub follower_id: String,
    /// ID of the profile being followed
    pub following_id: String,
    /// Timestamp of the follow action
    #[serde(default, deserialize_with = "deserialize_optional_number_from_string")]
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
    #[serde(default, deserialize_with = "deserialize_optional_number_from_string")]
    pub joined_at: Option<u64>,
}