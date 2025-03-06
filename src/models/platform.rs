use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::platforms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub creator_address: String,
    pub created_at: DateTime<Utc>,
    pub active_users_count: i32,
    pub total_users_count: i32,
    pub content_count: i32,
    pub last_activity_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::platforms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPlatform {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub creator_address: String,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub active_users_count: i32,
    pub total_users_count: i32,
    pub content_count: i32,
}