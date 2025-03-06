use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::fee_models)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FeeModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub model_type: i32,
    pub fee_amount: i64,
    pub total_split_basis_points: i32,
    pub owner_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::fee_models)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFeeModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub model_type: u8,
    pub fee_amount: i64,
    pub total_split_basis_points: i32,
    pub owner_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::fee_recipients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FeeRecipient {
    pub id: String,
    pub recipient_address: String,
    pub recipient_name: Option<String>,
    pub total_collected: i64,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::fee_recipients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFeeRecipient {
    pub id: String,
    pub recipient_address: String,
    pub recipient_name: Option<String>,
    pub total_collected: i64,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::fee_distributions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FeeDistribution {
    pub id: i32,
    pub fee_model_id: String,
    pub transaction_amount: i64,
    pub total_fee_amount: i64,
    pub token_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::fee_distributions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFeeDistribution {
    pub fee_model_id: String,
    pub transaction_amount: i64,
    pub total_fee_amount: i64,
    pub token_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::fee_recipient_payments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FeeRecipientPayment {
    pub distribution_id: i32,
    pub recipient_id: String,
    pub amount: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::fee_recipient_payments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFeeRecipientPayment {
    pub distribution_id: i32,
    pub recipient_id: String,
    pub amount: i64,
    pub created_at: DateTime<Utc>,
}