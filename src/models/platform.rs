// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::{platforms, platform_moderators, platform_events, platform_memberships, platform_blocked_profiles};

/// Platform status constants
pub const PLATFORM_STATUS_DEVELOPMENT: i16 = 0;
pub const PLATFORM_STATUS_ALPHA: i16 = 1;
pub const PLATFORM_STATUS_BETA: i16 = 2;
pub const PLATFORM_STATUS_LIVE: i16 = 3;
pub const PLATFORM_STATUS_MAINTENANCE: i16 = 4;
pub const PLATFORM_STATUS_SUNSET: i16 = 5;
pub const PLATFORM_STATUS_SHUTDOWN: i16 = 6;

/// Platform model
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = platforms)]
pub struct Platform {
    pub id: i32,
    pub platform_id: String,
    pub name: String,
    pub tagline: String,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub developer_address: String,
    pub terms_of_service: Option<String>,
    pub privacy_policy: Option<String>,
    pub platform_names: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
    pub status: i16,
    pub release_date: Option<String>,
    pub shutdown_date: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_approved: bool,
    pub approval_changed_at: Option<NaiveDateTime>,
    pub approved_by: Option<String>,
}

/// DTO for inserting a new platform
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = platforms)]
pub struct NewPlatform {
    pub platform_id: String,
    pub name: String,
    pub tagline: String,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub developer_address: String,
    pub terms_of_service: Option<String>,
    pub privacy_policy: Option<String>,
    pub platform_names: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
    pub status: i16,
    pub release_date: Option<String>,
    pub shutdown_date: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_approved: bool,
    pub approval_changed_at: Option<NaiveDateTime>,
    pub approved_by: Option<String>,
}

/// DTO for updating a platform
#[derive(Debug, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = platforms)]
pub struct UpdatePlatform {
    pub name: Option<String>,
    pub tagline: Option<String>,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub terms_of_service: Option<String>,
    pub privacy_policy: Option<String>,
    pub platform_names: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
    pub status: Option<i16>,
    pub release_date: Option<String>,
    pub shutdown_date: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
    pub is_approved: Option<bool>,
    pub approval_changed_at: Option<NaiveDateTime>,
    pub approved_by: Option<String>,
}

/// Platform moderator model
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = platform_moderators)]
pub struct PlatformModerator {
    pub id: i32,
    pub platform_id: String,
    pub moderator_address: String,
    pub added_by: String,
    pub created_at: NaiveDateTime,
}

/// DTO for inserting a new platform moderator
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = platform_moderators)]
pub struct NewPlatformModerator {
    pub platform_id: String,
    pub moderator_address: String,
    pub added_by: String,
    pub created_at: NaiveDateTime,
}

/// Platform blocked profile model
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = platform_blocked_profiles)]
pub struct PlatformBlockedProfile {
    pub id: i32,
    pub platform_id: String,
    pub profile_id: String,
    pub blocked_by: String,
    pub created_at: NaiveDateTime,
}

/// DTO for inserting a new platform blocked profile
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = platform_blocked_profiles)]
pub struct NewPlatformBlockedProfile {
    pub platform_id: String,
    pub profile_id: String,
    pub blocked_by: String,
    pub created_at: NaiveDateTime,
}

/// Platform event model
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = platform_events)]
pub struct PlatformEvent {
    pub id: i32,
    pub event_type: String,
    pub platform_id: String,
    pub event_data: serde_json::Value,
    pub event_id: Option<String>,
    pub created_at: NaiveDateTime,
}

/// DTO for inserting a new platform event
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = platform_events)]
pub struct NewPlatformEvent {
    pub event_type: String,
    pub platform_id: String,
    pub event_data: serde_json::Value,
    pub event_id: Option<String>,
    pub created_at: NaiveDateTime,
}

/// Platform with related data for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformWithDetails {
    // Platform details
    pub id: i32,
    pub platform_id: String,
    pub name: String,
    pub tagline: String,
    pub description: Option<String>,
    pub logo: Option<String>,
    pub developer_address: String,
    pub terms_of_service: Option<String>,
    pub privacy_policy: Option<String>,
    pub platform_names: Option<Vec<String>>,
    pub links: Option<Vec<String>>,
    pub status: i16,
    pub status_text: String,
    pub release_date: Option<String>,
    pub shutdown_date: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub is_approved: bool,
    pub approval_changed_at: Option<NaiveDateTime>,
    pub approved_by: Option<String>,
    // Related data
    pub moderator_count: i64,
    pub blocked_profiles_count: i64,
}

impl PlatformWithDetails {
    // Helper to convert platform status code to text
    pub fn status_to_text(status: i16) -> String {
        match status {
            PLATFORM_STATUS_DEVELOPMENT => "Development".to_string(),
            PLATFORM_STATUS_ALPHA => "Alpha".to_string(),
            PLATFORM_STATUS_BETA => "Beta".to_string(),
            PLATFORM_STATUS_LIVE => "Live".to_string(),
            PLATFORM_STATUS_MAINTENANCE => "Maintenance".to_string(),
            PLATFORM_STATUS_SUNSET => "Sunset".to_string(),
            PLATFORM_STATUS_SHUTDOWN => "Shutdown".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

/// Events from platform.move
#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformCreatedEvent {
    pub platform_id: String,
    pub name: String,
    pub tagline: String,
    pub description: Option<String>, // Added this field
    pub developer: String,
    pub logo: Option<String>, // Added this field
    pub terms_of_service: String,
    pub privacy_policy: String,
    pub platforms: Vec<String>,
    pub links: Vec<String>,
    pub status: PlatformStatus,
    pub release_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformApprovalChangedEvent {
    pub platform_id: String,
    #[serde(alias = "approved")]
    pub is_approved: bool,
    #[serde(alias = "changed_by")]
    pub approved_by: String,
    #[serde(default, deserialize_with = "deserialize_timestamp_optional")]
    pub changed_at: u64,
}

// Standard deserializer for timestamps that accepts both string and number formats
// Falls back to current time if parsing fails
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct TimestampVisitor;

    impl<'de> serde::de::Visitor<'de> for TimestampVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number or string representing a timestamp")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match value.parse::<u64>() {
                Ok(ts) => Ok(ts),
                Err(e) => {
                    // Log the error but don't fail - use current time instead
                    tracing::warn!("Failed to parse timestamp string '{}': {}. Using current time instead.", value, e);
                    let current_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    Ok(current_time)
                }
            }
        }
        
        // Handle null or missing values
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.visit_unit()
        }
        
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            Ok(current_time)
        }
    }

    deserializer.deserialize_any(TimestampVisitor)
}

// Version that handles missing fields or null values
fn deserialize_timestamp_optional<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Try to deserialize, falling back to current time instead of 0
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
        
    Option::deserialize(deserializer)
        .map(|opt_val: Option<serde_json::Value>| {
            match opt_val {
                Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(current_time),
                Some(serde_json::Value::String(s)) => s.parse::<u64>().unwrap_or(current_time),
                _ => current_time
            }
        })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformUpdatedEvent {
    pub platform_id: String,
    pub name: String,
    pub tagline: String,
    pub description: String,
    pub terms_of_service: String,
    pub privacy_policy: String,
    pub platforms: Vec<String>,
    pub links: Vec<String>,
    pub status: PlatformStatus,
    pub release_date: String,
    pub shutdown_date: Option<String>,
    pub updated_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformStatus {
    pub status: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModeratorAddedEvent {
    pub platform_id: String,
    pub moderator_address: String,
    pub added_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModeratorRemovedEvent {
    pub platform_id: String,
    pub moderator_address: String,
    pub removed_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformBlockedProfileEvent {
    pub platform_id: String,
    pub profile_id: String,
    pub blocked_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformUnblockedProfileEvent {
    pub platform_id: String,
    pub profile_id: String,
    pub unblocked_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserJoinedPlatformEvent {
    pub profile_id: String,
    pub platform_id: String,
    pub user: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserLeftPlatformEvent {
    pub profile_id: String,
    pub platform_id: String,
    pub user: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub timestamp: u64,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = platform_memberships)]
pub struct NewPlatformMembership {
    pub platform_id: String,
    pub profile_id: String,
    pub joined_at: NaiveDateTime,
}

// Note: PlatformRelationship, NewPlatformRelationship, and UpdatePlatformRelationship 
// have been removed in favor of using platform_memberships table