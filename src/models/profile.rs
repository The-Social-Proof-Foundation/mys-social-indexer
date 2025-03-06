use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Profile {
    pub id: String,
    pub owner_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub followers_count: i32,
    pub following_count: i32,
    pub content_count: i32,
    pub platforms_joined: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::profiles)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewProfile {
    pub id: String,
    pub owner_address: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub followers_count: i32,
    pub following_count: i32,
    pub content_count: i32,
    pub platforms_joined: i32,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::follows)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Follow {
    pub follower_id: String,
    pub following_id: String,
    pub followed_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::follows)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFollow {
    pub follower_id: String,
    pub following_id: String,
    pub followed_at: DateTime<Utc>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::profile_platform_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProfilePlatformLink {
    pub profile_id: String,
    pub platform_id: String,
    pub joined_at: DateTime<Utc>,
    pub last_active_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::profile_platform_links)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewProfilePlatformLink {
    pub profile_id: String,
    pub platform_id: String,
    pub joined_at: DateTime<Utc>,
    pub last_active_at: Option<DateTime<Utc>>,
}