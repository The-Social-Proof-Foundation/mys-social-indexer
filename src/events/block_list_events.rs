use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::models::block_list::NewBlock;

/// Event emitted when an entity is blocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBlockedEvent {
    pub blocker_id: String,
    pub blocker_type: u8,
    pub blocked_id: String,
    pub reason: String,
    pub timestamp: u64,
}

impl EntityBlockedEvent {
    /// Convert the event into a NewBlock model
    pub fn into_model(&self) -> Result<NewBlock> {
        let blocked_at = DateTime::<Utc>::from_timestamp(self.timestamp as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        Ok(NewBlock {
            blocker_id: self.blocker_id.clone(),
            blocked_id: self.blocked_id.clone(),
            blocker_type: self.blocker_type,
            reason: Some(self.reason.clone()),
            blocked_at,
        })
    }
}

/// Event emitted when an entity is unblocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityUnblockedEvent {
    pub blocker_id: String,
    pub blocker_type: u8,
    pub unblocked_id: String,
    pub timestamp: u64,
}