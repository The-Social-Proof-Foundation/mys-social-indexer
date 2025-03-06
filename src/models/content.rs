use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::content)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Content {
    pub id: String,
    pub creator_id: String,
    pub platform_id: String,
    pub content_type: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub has_ip_registered: bool,
    pub view_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub share_count: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::content)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewContent {
    pub id: String,
    pub creator_id: String,
    pub platform_id: String,
    pub content_type: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub has_ip_registered: bool,
    pub view_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub share_count: i32,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::content_interactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContentInteraction {
    pub profile_id: String,
    pub content_id: String,
    pub interaction_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::content_interactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewContentInteraction {
    pub profile_id: String,
    pub content_id: String,
    pub interaction_type: String,
    pub created_at: DateTime<Utc>,
}