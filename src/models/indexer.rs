// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use crate::schema::indexer_progress;

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = indexer_progress)]
pub struct IndexerProgress {
    pub id: String,
    pub last_checkpoint_processed: i64,
    pub last_processed_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[diesel(table_name = indexer_progress)]
pub struct NewIndexerProgress {
    pub id: String,
    pub last_checkpoint_processed: i64,
    pub last_processed_at: NaiveDateTime,
}