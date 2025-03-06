use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::indexer_progress)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IndexerProgress {
    pub id: String,
    pub last_checkpoint_processed: i64,
    pub last_processed_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::indexer_progress)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewIndexerProgress {
    pub id: String,
    pub last_checkpoint_processed: i64,
    pub last_processed_at: DateTime<Utc>,
}