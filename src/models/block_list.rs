use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Block {
    pub blocker_id: String,
    pub blocked_id: String,
    pub blocker_type: i32,
    pub reason: Option<String>,
    pub blocked_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewBlock {
    pub blocker_id: String,
    pub blocked_id: String,
    pub blocker_type: u8,
    pub reason: Option<String>,
    pub blocked_at: DateTime<Utc>,
}