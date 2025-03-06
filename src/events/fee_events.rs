use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::models::fee_distribution::{NewFeeModel, NewFeeDistribution, NewFeeRecipient, NewFeeRecipientPayment};

/// Event emitted when a new fee model is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeModelCreatedEvent {
    pub fee_model_id: String,
    pub name: String,
    pub model_type: u8,
    pub fee_amount: u64,
    pub total_split_bps: u64,
    pub owner: String,
}

impl FeeModelCreatedEvent {
    /// Convert the event into a NewFeeModel model
    pub fn into_model(&self, description: Option<String>) -> Result<NewFeeModel> {
        let created_at = Utc::now(); // Create with current timestamp as event doesn't include it
            
        Ok(NewFeeModel {
            id: self.fee_model_id.clone(),
            name: self.name.clone(),
            description,
            model_type: self.model_type,
            fee_amount: self.fee_amount as i64,
            total_split_basis_points: self.total_split_bps as i32,
            owner_address: self.owner.clone(),
            created_at,
        })
    }
}

/// Event emitted when fees are distributed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeesDistributedEvent {
    pub fee_model_id: String,
    pub model_name: String,
    pub transaction_amount: u64,
    pub total_fee_amount: u64,
    pub token_type: String,
    pub timestamp: u64,
}

impl FeesDistributedEvent {
    /// Convert the event into a NewFeeDistribution model
    pub fn into_model(&self) -> Result<NewFeeDistribution> {
        let created_at = DateTime::<Utc>::from_timestamp(self.timestamp as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        Ok(NewFeeDistribution {
            fee_model_id: self.fee_model_id.clone(),
            transaction_amount: self.transaction_amount as i64,
            total_fee_amount: self.total_fee_amount as i64,
            token_type: self.token_type.clone(),
            created_at,
        })
    }
}

/// Fee split information from the fee distribution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeSplit {
    pub recipient: String,
    pub recipient_name: String,
    pub share_bps: u64,
}

/// Event emitted when a fee model is updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeModelUpdatedEvent {
    pub fee_model_id: String,
    pub name: String,
    pub fee_amount: u64,
    pub total_split_bps: u64,
    pub timestamp: u64,
}

/// Event emitted when a recipient withdraws fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeWithdrawalEvent {
    pub recipient: String,
    pub token_type: String,
    pub amount: u64,
    pub timestamp: u64,
}