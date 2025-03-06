use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::Array;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::intellectual_property)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IntellectualProperty {
    pub id: String,
    pub creator_id: String,
    pub title: String,
    pub description: Option<String>,
    pub ip_type: i32,
    pub content_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub royalty_basis_points: Option<i32>,
    pub registered_countries: Vec<String>,
    pub ipo_tokenized: bool,
    pub total_licenses_count: i32,
    pub active_licenses_count: i32,
    pub total_revenue: i64,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::intellectual_property)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewIntellectualProperty {
    pub id: String,
    pub creator_id: String,
    pub title: String,
    pub description: Option<String>,
    pub ip_type: u8,
    pub content_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub royalty_basis_points: Option<i32>,
    pub registered_countries: Vec<String>,
    pub ipo_tokenized: bool,
    pub total_licenses_count: i32,
    pub active_licenses_count: i32,
    pub total_revenue: i64,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ip_licenses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IPLicense {
    pub id: String,
    pub ip_id: String,
    pub licensee_id: String,
    pub license_type: i32,
    pub terms: Option<String>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: i32,
    pub payment_amount: i64,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::ip_licenses)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewIPLicense {
    pub id: String,
    pub ip_id: String,
    pub licensee_id: String,
    pub license_type: u8,
    pub terms: Option<String>,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub status: i32,
    pub payment_amount: i64,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::proof_of_creativity)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProofOfCreativity {
    pub id: String,
    pub creator_id: String,
    pub ip_id: Option<String>,
    pub title: String,
    pub proof_type: i32,
    pub verification_state: i32,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::proof_of_creativity)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewProofOfCreativity {
    pub id: String,
    pub creator_id: String,
    pub ip_id: Option<String>,
    pub title: String,
    pub proof_type: u8,
    pub verification_state: i32,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}