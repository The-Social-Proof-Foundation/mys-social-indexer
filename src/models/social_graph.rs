// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::{social_graph_relationships, social_graph_events};

/// Model for a social graph relationship (follow)
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = social_graph_relationships)]
pub struct SocialGraphRelationship {
    pub id: i32,
    pub follower_address: String,
    pub following_address: String,
    pub created_at: NaiveDateTime,
}

/// DTO for creating a new social graph relationship
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = social_graph_relationships)]
pub struct NewSocialGraphRelationship {
    pub follower_address: String,
    pub following_address: String,
    pub created_at: NaiveDateTime,
}

/// Model for social graph events
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = social_graph_events)]
pub struct SocialGraphEvent {
    pub id: i32,
    pub event_type: String,
    pub follower_address: String,
    pub following_address: String,
    pub created_at: NaiveDateTime,
    pub event_id: Option<String>,  // Changed from blockchain_tx_hash to event_id
    pub raw_event_data: Option<serde_json::Value>,
}

/// DTO for creating a new social graph event
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = social_graph_events)]
pub struct NewSocialGraphEvent {
    pub event_type: String,
    pub follower_address: String,
    pub following_address: String,
    pub created_at: NaiveDateTime,
    pub event_id: Option<String>,  // Changed from blockchain_tx_hash to event_id
    pub raw_event_data: Option<serde_json::Value>,
}

/// DTO for querying followers or following with profile details
#[derive(Debug, Serialize, Deserialize)]
pub struct FollowDetail {
    // Profile ID in the database
    pub id: i32,
    // Profile ID in the blockchain
    pub profile_id: Option<String>,
    // Owner address
    pub owner_address: String,
    // Username
    pub username: String,
    // Display name
    pub display_name: Option<String>,
    // Profile photo
    pub profile_photo: Option<String>,
    // Bio
    pub bio: Option<String>,
    // Website
    pub website: Option<String>,
    // When the relationship was created
    pub followed_at: NaiveDateTime,
}

/// Query parameters for paginating followers/following lists
#[derive(Debug, Deserialize)]
pub struct FollowsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub page: Option<i64>,
}