use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::models::intellectual_property::{NewIntellectualProperty, NewIPLicense, NewProofOfCreativity};

/// Event emitted when new IP is registered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPRegisteredEvent {
    pub ip_id: String,
    pub creator: String,
    pub title: String,
    pub ip_type: u8,
    pub created_at: u64,
}

impl IPRegisteredEvent {
    /// Convert the event into a NewIntellectualProperty model
    pub fn into_model(&self, description: Option<String>, royalty_basis_points: Option<i32>) -> Result<NewIntellectualProperty> {
        let created_at = DateTime::<Utc>::from_timestamp(self.created_at as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        Ok(NewIntellectualProperty {
            id: self.ip_id.clone(),
            creator_id: self.creator.clone(),
            title: self.title.clone(),
            description,
            ip_type: self.ip_type,
            content_hash: None,
            created_at,
            royalty_basis_points,
            registered_countries: Vec::new(),
            ipo_tokenized: false,
            total_licenses_count: 0,
            active_licenses_count: 0,
            total_revenue: 0,
        })
    }
}

/// Event emitted when a license is granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseGrantedEvent {
    pub license_id: String,
    pub ip_id: String,
    pub licensee: String,
    pub license_type: u8,
    pub granted_at: u64,
    pub expires_at: u64,
    pub payment_amount: u64,
}

impl LicenseGrantedEvent {
    /// Convert the event into a NewIPLicense model
    pub fn into_model(&self, terms: Option<String>) -> Result<NewIPLicense> {
        let granted_at = DateTime::<Utc>::from_timestamp(self.granted_at as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        let expires_at = if self.expires_at == 0 {
            None
        } else {
            Some(DateTime::<Utc>::from_timestamp(self.expires_at as i64 / 1000, 0)
                .unwrap_or_else(|| Utc::now()))
        };
            
        Ok(NewIPLicense {
            id: self.license_id.clone(),
            ip_id: self.ip_id.clone(),
            licensee_id: self.licensee.clone(),
            license_type: self.license_type,
            terms,
            granted_at,
            expires_at,
            status: 0, // Active status
            payment_amount: self.payment_amount as i64,
        })
    }
}

/// Event emitted when a license status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatusChangedEvent {
    pub license_id: String,
    pub ip_id: String,
    pub licensee: String,
    pub old_status: u8,
    pub new_status: u8,
}

/// Event emitted when a dispute is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeCreatedEvent {
    pub dispute_id: String,
    pub ip_id: String,
    pub challenger: String,
    pub original_creator: String,
    pub created_at: u64,
}

/// Event emitted when a dispute is resolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeResolvedEvent {
    pub dispute_id: String,
    pub ip_id: String,
    pub resolution: u8,
    pub resolver: String,
}

/// Event emitted when proof of creativity is created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofCreatedEvent {
    pub poc_id: String,
    pub creator: String,
    pub title: String,
    pub proof_type: u8,
    pub created_at: u64,
}

impl ProofCreatedEvent {
    /// Convert the event into a NewProofOfCreativity model
    pub fn into_model(&self) -> Result<NewProofOfCreativity> {
        let created_at = DateTime::<Utc>::from_timestamp(self.created_at as i64 / 1000, 0)
            .unwrap_or_else(|| Utc::now());
            
        Ok(NewProofOfCreativity {
            id: self.poc_id.clone(),
            creator_id: self.creator.clone(),
            ip_id: None, // Will be linked later if available
            title: self.title.clone(),
            proof_type: self.proof_type,
            verification_state: 0, // Pending state
            verified_at: None,
            created_at,
        })
    }
}

/// Event emitted when proof is verified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerifiedEvent {
    pub poc_id: String,
    pub creator: String,
    pub provider: String,
    pub verification_state: u8,
    pub verification_time: u64,
}