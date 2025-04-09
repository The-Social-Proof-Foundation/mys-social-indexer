// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::profiles_blocked;

/// Profile block model - represents a profile blocking another profile
#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = profiles_blocked)]
pub struct ProfileBlock {
    pub id: i32,
    pub blocker_profile_id: String,
    pub blocked_profile_id: String,
    pub created_at: NaiveDateTime,
    pub is_blocked: bool,
    pub unblocked_at: Option<NaiveDateTime>,
}

/// DTO for inserting a new profile block
#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = profiles_blocked)]
pub struct NewProfileBlock {
    pub blocker_profile_id: String,
    pub blocked_profile_id: String,
    pub created_at: NaiveDateTime,
    pub is_blocked: bool,
}

/// DTO for updating a profile block
#[derive(Debug, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = profiles_blocked)]
pub struct UpdateProfileBlock {
    pub is_blocked: Option<bool>,
    pub unblocked_at: Option<NaiveDateTime>,
}

/// Events from block_list.move - renamed to match Move contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBlockEvent {
    pub blocker: String,
    pub blocked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUnblockEvent {
    pub blocker: String,
    pub unblocked: String,
}

// No legacy event types needed - using only UserBlockEvent and UserUnblockEvent