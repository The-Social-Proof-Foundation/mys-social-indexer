use chrono::{NaiveDate, DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::daily_statistics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DailyStatistics {
    pub date: NaiveDate,
    pub new_profiles_count: i32,
    pub active_profiles_count: i32,
    pub new_content_count: i32,
    pub total_interactions_count: i32,
    pub new_ip_registrations_count: i32,
    pub new_licenses_count: i32,
    pub total_fees_distributed: i64,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::daily_statistics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDailyStatistics {
    pub date: NaiveDate,
    pub new_profiles_count: i32,
    pub active_profiles_count: i32,
    pub new_content_count: i32,
    pub total_interactions_count: i32,
    pub new_ip_registrations_count: i32,
    pub new_licenses_count: i32,
    pub total_fees_distributed: i64,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::platform_daily_statistics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlatformDailyStatistics {
    pub platform_id: String,
    pub date: NaiveDate,
    pub active_users_count: i32,
    pub new_users_count: i32,
    pub content_created_count: i32,
    pub total_interactions_count: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::platform_daily_statistics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewPlatformDailyStatistics {
    pub platform_id: String,
    pub date: NaiveDate,
    pub active_users_count: i32,
    pub new_users_count: i32,
    pub content_created_count: i32,
    pub total_interactions_count: i32,
}