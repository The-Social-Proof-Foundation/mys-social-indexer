// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::social_graph::{NewSocialGraphRelationship};
use crate::models::profile::UpdateProfile;

/// Event emitted when a profile follows another profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowEvent {
    /// Address of the follower
    pub follower: String,
    /// Address of the user being followed
    pub following: String,
    /// Optional timestamp - if not provided, current time will be used
    #[serde(default)]
    pub timestamp: Option<u64>,
}

/// Event emitted when a profile unfollows another profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnfollowEvent {
    /// Address of the follower who is unfollowing
    pub follower: String,
    /// Address of the user being unfollowed
    pub unfollowed: String,
    /// Optional timestamp - if not provided, current time will be used
    #[serde(default)]
    pub timestamp: Option<u64>,
}

impl FollowEvent {
    /// Convert the FollowEvent to a NewSocialGraphRelationship database model
    pub fn into_relationship(&self) -> Result<NewSocialGraphRelationship> {
        // Use provided timestamp or current time
        let timestamp = self.timestamp.unwrap_or_else(|| 
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );
        
        let created_at = DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or(Utc::now())
            .naive_utc();
            
        Ok(NewSocialGraphRelationship {
            follower_address: self.follower.clone(),
            following_address: self.following.clone(),
            created_at,
        })
    }
    
    /// Create UpdateProfile struct to increment following count
    pub fn follower_update(&self) -> UpdateProfile {
        UpdateProfile {
            display_name: None,
            bio: None,
            profile_photo: None,
            website: None,
            cover_photo: None,
            sensitive_data_updated_at: None,
            followers_count: None,
            following_count: Some(1), // This will be used in a raw SQL to increment by 1
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
            block_list_address: None,
        }
    }
    
    /// Create UpdateProfile struct to increment followers count
    pub fn following_update(&self) -> UpdateProfile {
        UpdateProfile {
            display_name: None,
            bio: None,
            profile_photo: None,
            website: None,
            cover_photo: None,
            sensitive_data_updated_at: None,
            followers_count: Some(1), // This will be used in a raw SQL to increment by 1
            following_count: None,
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
            block_list_address: None,
        }
    }
}

impl UnfollowEvent {
    /// Create UpdateProfile struct to decrement following count
    pub fn follower_update(&self) -> UpdateProfile {
        UpdateProfile {
            display_name: None,
            bio: None,
            profile_photo: None,
            website: None,
            cover_photo: None,
            sensitive_data_updated_at: None,
            followers_count: None,
            following_count: Some(-1), // This will be used in a raw SQL to decrement by 1
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
            block_list_address: None,
        }
    }
    
    /// Create UpdateProfile struct to decrement followers count
    pub fn following_update(&self) -> UpdateProfile {
        UpdateProfile {
            display_name: None,
            bio: None,
            profile_photo: None,
            website: None,
            cover_photo: None,
            sensitive_data_updated_at: None,
            followers_count: Some(-1), // This will be used in a raw SQL to decrement by 1
            following_count: None,
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
            block_list_address: None,
        }
    }
}