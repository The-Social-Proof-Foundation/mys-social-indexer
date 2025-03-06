use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::models::content::{NewContent, NewContentInteraction};

/// Event emitted when content is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCreatedEvent {
    pub content_id: String,
    pub creator_id: String,
    pub platform_id: String,
    pub content_type: String,
    pub parent_id: Option<String>,
    pub created_at: u64,
}

impl ContentCreatedEvent {
    /// Convert the event into a NewContent model
    pub fn into_model(&self) -> Result<NewContent> {
        let created_at = DateTime::<Utc>::from_timestamp(self.created_at as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        Ok(NewContent {
            id: self.content_id.clone(),
            creator_id: self.creator_id.clone(),
            platform_id: self.platform_id.clone(),
            content_type: self.content_type.clone(),
            parent_id: self.parent_id.clone(),
            created_at,
            has_ip_registered: false,
            view_count: 0,
            like_count: 0,
            comment_count: 0,
            share_count: 0,
        })
    }
}

/// Event emitted when content is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentUpdatedEvent {
    pub content_id: String,
    pub updated_at: u64,
}

/// Event emitted when a user interacts with content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentInteractionEvent {
    pub profile_id: String,
    pub content_id: String,
    pub interaction_type: String,
    pub created_at: u64,
}

impl ContentInteractionEvent {
    /// Convert the event into a NewContentInteraction model
    pub fn into_model(&self) -> Result<NewContentInteraction> {
        let created_at = DateTime::<Utc>::from_timestamp(self.created_at as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        Ok(NewContentInteraction {
            profile_id: self.profile_id.clone(),
            content_id: self.content_id.clone(),
            interaction_type: self.interaction_type.clone(),
            created_at,
        })
    }
}

/// Event emitted when content is removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRemovedEvent {
    pub content_id: String,
    pub removed_at: u64,
}