use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{Utc, NaiveDate};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use mys_data_ingestion_core::Worker;
use mys_types::full_checkpoint_content::CheckpointData;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::db::{Database, DbConnection};
use crate::events::{
    parse_event,
    MODULE_PREFIX_PROFILE, MODULE_PREFIX_PLATFORM, MODULE_PREFIX_CONTENT,
    MODULE_PREFIX_BLOCK_LIST, MODULE_PREFIX_MY_IP, MODULE_PREFIX_FEE_DISTRIBUTION,
    ProfileCreatedEvent, PlatformCreatedEvent, ContentCreatedEvent, ContentInteractionEvent,
    EntityBlockedEvent, IPRegisteredEvent, LicenseGrantedEvent, ProofCreatedEvent,
    FeeModelCreatedEvent, FeesDistributedEvent, ProfileFollowEvent, ProfileJoinedPlatformEvent,
};
use crate::models::profile::{NewProfile, NewFollow, NewProfilePlatformLink};
use crate::models::platform::NewPlatform;
use crate::models::content::{NewContent, NewContentInteraction};
use crate::models::block_list::NewBlock;
use crate::models::intellectual_property::{NewIntellectualProperty, NewIPLicense, NewProofOfCreativity};
use crate::models::fee_distribution::{NewFeeModel, NewFeeDistribution, NewFeeRecipient, NewFeeRecipientPayment};
use crate::models::statistics::{NewDailyStatistics, NewPlatformDailyStatistics};
use crate::models::indexer::NewIndexerProgress;
use crate::schema;

/// Social indexer worker that processes blockchain events
pub struct SocialIndexerWorker {
    /// Database connection pool
    db: Arc<Database>,
    /// Worker ID
    worker_id: String,
}

impl SocialIndexerWorker {
    /// Create a new social indexer worker
    pub fn new(db: Arc<Database>, worker_id: String) -> Self {
        Self { db, worker_id }
    }
    
    /// Get a database connection from the pool
    async fn get_connection(&self) -> Result<DbConnection> {
        self.db.get_connection()
            .await
            .map_err(|e| anyhow!("Failed to get database connection: {}", e))
    }
    
    /// Update worker progress
    async fn update_progress(&self, checkpoint_seq: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let now = Utc::now();
        
        let progress = NewIndexerProgress {
            id: self.worker_id.clone(),
            last_checkpoint_processed: checkpoint_seq as i64,
            last_processed_at: now,
        };
        
        diesel::insert_into(schema::indexer_progress::table)
            .values(&progress)
            .on_conflict(schema::indexer_progress::id)
            .do_update()
            .set(&progress)
            .execute(&mut conn)
            .await?;
            
        Ok(())
    }
    
    /// Process a profile created event
    async fn process_profile_created(&self, event: &ProfileCreatedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_profile = event.into_model()?;
        
        // Insert the profile
        diesel::insert_into(schema::profiles::table)
            .values(&new_profile)
            .on_conflict(schema::profiles::id)
            .do_update()
            .set(&new_profile)
            .execute(&mut conn)
            .await?;
            
        // Update daily statistics
        self.update_daily_stats(|stats| {
            stats.new_profiles_count += 1;
        }).await?;
        
        info!("Processed profile created: {}", event.profile_id);
        Ok(())
    }
    
    /// Process a profile follow event
    async fn process_profile_follow(&self, event: &ProfileFollowEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Create new follow relationship
        let follow = NewFollow {
            follower_id: event.follower_id.clone(),
            following_id: event.following_id.clone(),
            followed_at: Utc::now(), // Use current time if event doesn't provide it
        };
        
        // Insert the follow relationship
        diesel::insert_into(schema::follows::table)
            .values(&follow)
            .on_conflict((schema::follows::follower_id, schema::follows::following_id))
            .do_update()
            .set(&follow)
            .execute(&mut conn)
            .await?;
            
        // Update follower and following counts
        diesel::update(schema::profiles::table.find(&event.follower_id))
            .set(schema::profiles::following_count.eq(schema::profiles::following_count + 1))
            .execute(&mut conn)
            .await?;
            
        diesel::update(schema::profiles::table.find(&event.following_id))
            .set(schema::profiles::followers_count.eq(schema::profiles::followers_count + 1))
            .execute(&mut conn)
            .await?;
            
        info!("Processed profile follow: {} -> {}", event.follower_id, event.following_id);
        Ok(())
    }
    
    /// Process a platform created event
    async fn process_platform_created(&self, event: &PlatformCreatedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_platform = event.into_model()?;
        
        // Insert the platform
        diesel::insert_into(schema::platforms::table)
            .values(&new_platform)
            .on_conflict(schema::platforms::id)
            .do_update()
            .set(&new_platform)
            .execute(&mut conn)
            .await?;
            
        info!("Processed platform created: {}", event.platform_id);
        Ok(())
    }
    
    /// Process a profile joined platform event
    async fn process_profile_joined_platform(&self, event: &ProfileJoinedPlatformEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Create join record
        let joined_at = Utc::now(); // Use current time if event doesn't provide it
        let link = NewProfilePlatformLink {
            profile_id: event.profile_id.clone(),
            platform_id: event.platform_id.clone(),
            joined_at,
            last_active_at: Some(joined_at),
        };
        
        // Insert the platform join
        diesel::insert_into(schema::profile_platform_links::table)
            .values(&link)
            .on_conflict((schema::profile_platform_links::profile_id, schema::profile_platform_links::platform_id))
            .do_update()
            .set(&link)
            .execute(&mut conn)
            .await?;
            
        // Update platform user counts
        diesel::update(schema::platforms::table.find(&event.platform_id))
            .set((
                schema::platforms::total_users_count.eq(schema::platforms::total_users_count + 1),
                schema::platforms::active_users_count.eq(schema::platforms::active_users_count + 1),
                schema::platforms::last_activity_at.eq(joined_at),
            ))
            .execute(&mut conn)
            .await?;
            
        // Update profile platforms joined count
        diesel::update(schema::profiles::table.find(&event.profile_id))
            .set((
                schema::profiles::platforms_joined.eq(schema::profiles::platforms_joined + 1),
                schema::profiles::last_activity_at.eq(joined_at),
            ))
            .execute(&mut conn)
            .await?;
            
        // Update platform daily statistics
        self.update_platform_daily_stats(&event.platform_id, |stats| {
            stats.new_users_count += 1;
            stats.active_users_count += 1;
        }).await?;
        
        info!("Processed profile joined platform: {} -> {}", event.profile_id, event.platform_id);
        Ok(())
    }
    
    /// Process a content created event
    async fn process_content_created(&self, event: &ContentCreatedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_content = event.into_model()?;
        
        // Insert the content
        diesel::insert_into(schema::content::table)
            .values(&new_content)
            .on_conflict(schema::content::id)
            .do_update()
            .set(&new_content)
            .execute(&mut conn)
            .await?;
            
        // Update profile content count
        diesel::update(schema::profiles::table.find(&event.creator_id))
            .set((
                schema::profiles::content_count.eq(schema::profiles::content_count + 1),
                schema::profiles::last_activity_at.eq(new_content.created_at),
            ))
            .execute(&mut conn)
            .await?;
            
        // Update platform content count
        diesel::update(schema::platforms::table.find(&event.platform_id))
            .set((
                schema::platforms::content_count.eq(schema::platforms::content_count + 1),
                schema::platforms::last_activity_at.eq(new_content.created_at),
            ))
            .execute(&mut conn)
            .await?;
            
        // If this is a comment/reply, increment the comment count on the parent
        if let Some(parent_id) = &event.parent_id {
            diesel::update(schema::content::table.find(parent_id))
                .set(schema::content::comment_count.eq(schema::content::comment_count + 1))
                .execute(&mut conn)
                .await?;
        }
            
        // Update daily statistics
        self.update_daily_stats(|stats| {
            stats.new_content_count += 1;
        }).await?;
        
        // Update platform daily statistics
        self.update_platform_daily_stats(&event.platform_id, |stats| {
            stats.content_created_count += 1;
        }).await?;
        
        info!("Processed content created: {}", event.content_id);
        Ok(())
    }
    
    /// Process a content interaction event
    async fn process_content_interaction(&self, event: &ContentInteractionEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_interaction = event.into_model()?;
        
        // Insert the interaction
        diesel::insert_into(schema::content_interactions::table)
            .values(&new_interaction)
            .on_conflict((
                schema::content_interactions::profile_id, 
                schema::content_interactions::content_id,
                schema::content_interactions::interaction_type
            ))
            .do_update()
            .set(&new_interaction)
            .execute(&mut conn)
            .await?;
            
        // Update content metrics based on interaction type
        match event.interaction_type.as_str() {
            "like" => {
                diesel::update(schema::content::table.find(&event.content_id))
                    .set(schema::content::like_count.eq(schema::content::like_count + 1))
                    .execute(&mut conn)
                    .await?;
            },
            "view" => {
                diesel::update(schema::content::table.find(&event.content_id))
                    .set(schema::content::view_count.eq(schema::content::view_count + 1))
                    .execute(&mut conn)
                    .await?;
            },
            "share" => {
                diesel::update(schema::content::table.find(&event.content_id))
                    .set(schema::content::share_count.eq(schema::content::share_count + 1))
                    .execute(&mut conn)
                    .await?;
            },
            _ => {}
        }
            
        // Update user last activity
        diesel::update(schema::profiles::table.find(&event.profile_id))
            .set(schema::profiles::last_activity_at.eq(new_interaction.created_at))
            .execute(&mut conn)
            .await?;
            
        // Get platform ID from content
        let content = schema::content::table
            .find(&event.content_id)
            .select(schema::content::platform_id)
            .first::<String>(&mut conn)
            .await?;
            
        // Update daily statistics
        self.update_daily_stats(|stats| {
            stats.total_interactions_count += 1;
        }).await?;
        
        // Update platform daily statistics
        self.update_platform_daily_stats(&content, |stats| {
            stats.total_interactions_count += 1;
        }).await?;
        
        info!("Processed content interaction: {} -> {}: {}", 
            event.profile_id, event.content_id, event.interaction_type);
        Ok(())
    }
    
    /// Process an entity blocked event
    async fn process_entity_blocked(&self, event: &EntityBlockedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_block = event.into_model()?;
        
        // Insert the block
        diesel::insert_into(schema::blocks::table)
            .values(&new_block)
            .on_conflict((schema::blocks::blocker_id, schema::blocks::blocked_id))
            .do_update()
            .set(&new_block)
            .execute(&mut conn)
            .await?;
            
        info!("Processed entity blocked: {} blocked {}", event.blocker_id, event.blocked_id);
        Ok(())
    }
    
    /// Process an IP registration event
    async fn process_ip_registered(&self, event: &IPRegisteredEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_ip = event.into_model(None, None)?;
        
        // Insert the IP registration
        diesel::insert_into(schema::intellectual_property::table)
            .values(&new_ip)
            .on_conflict(schema::intellectual_property::id)
            .do_update()
            .set(&new_ip)
            .execute(&mut conn)
            .await?;
            
        // Update daily statistics
        self.update_daily_stats(|stats| {
            stats.new_ip_registrations_count += 1;
        }).await?;
        
        // If this IP is for content, mark the content as having IP
        diesel::update(schema::content::table.find(&event.ip_id))
            .set(schema::content::has_ip_registered.eq(true))
            .execute(&mut conn)
            .await
            .ok(); // Ignore errors, content might not exist
            
        info!("Processed IP registered: {}", event.ip_id);
        Ok(())
    }
    
    /// Process a license granted event
    async fn process_license_granted(&self, event: &LicenseGrantedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_license = event.into_model(None)?;
        
        // Insert the license
        diesel::insert_into(schema::ip_licenses::table)
            .values(&new_license)
            .on_conflict(schema::ip_licenses::id)
            .do_update()
            .set(&new_license)
            .execute(&mut conn)
            .await?;
            
        // Update IP metrics
        diesel::update(schema::intellectual_property::table.find(&event.ip_id))
            .set((
                schema::intellectual_property::total_licenses_count.eq(
                    schema::intellectual_property::total_licenses_count + 1
                ),
                schema::intellectual_property::active_licenses_count.eq(
                    schema::intellectual_property::active_licenses_count + 1
                ),
                schema::intellectual_property::total_revenue.eq(
                    schema::intellectual_property::total_revenue + event.payment_amount as i64
                ),
            ))
            .execute(&mut conn)
            .await?;
            
        // Update daily statistics
        self.update_daily_stats(|stats| {
            stats.new_licenses_count += 1;
        }).await?;
        
        info!("Processed license granted: {} for IP {}", event.license_id, event.ip_id);
        Ok(())
    }
    
    /// Process a fee distribution event
    async fn process_fee_distribution(&self, event: &FeesDistributedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Convert event to database model
        let new_distribution = event.into_model()?;
        
        // Insert the fee distribution
        let result = diesel::insert_into(schema::fee_distributions::table)
            .values(&new_distribution)
            .returning(schema::fee_distributions::id)
            .get_result::<i32>(&mut conn)
            .await?;
            
        let distribution_id = result;
            
        // Update daily statistics
        self.update_daily_stats(|stats| {
            stats.total_fees_distributed += event.total_fee_amount as i64;
        }).await?;
        
        info!("Processed fee distribution: {} for model {}", distribution_id, event.fee_model_id);
        Ok(())
    }
    
    /// Update daily statistics
    async fn update_daily_stats<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut NewDailyStatistics),
    {
        let mut conn = self.get_connection().await?;
        let today = Utc::now().date_naive();
        
        // Try to load existing stats for today
        let existing_stats = schema::daily_statistics::table
            .find(today)
            .first::<crate::models::statistics::DailyStatistics>(&mut conn)
            .await
            .ok();
            
        // Create new stats or update existing
        let mut stats = match existing_stats {
            Some(existing) => NewDailyStatistics {
                date: existing.date,
                new_profiles_count: existing.new_profiles_count,
                active_profiles_count: existing.active_profiles_count,
                new_content_count: existing.new_content_count,
                total_interactions_count: existing.total_interactions_count,
                new_ip_registrations_count: existing.new_ip_registrations_count,
                new_licenses_count: existing.new_licenses_count,
                total_fees_distributed: existing.total_fees_distributed,
            },
            None => NewDailyStatistics {
                date: today,
                new_profiles_count: 0,
                active_profiles_count: 0,
                new_content_count: 0,
                total_interactions_count: 0,
                new_ip_registrations_count: 0,
                new_licenses_count: 0,
                total_fees_distributed: 0,
            },
        };
        
        // Apply updates to stats
        updater(&mut stats);
        
        // Insert or update stats
        diesel::insert_into(schema::daily_statistics::table)
            .values(&stats)
            .on_conflict(schema::daily_statistics::date)
            .do_update()
            .set(&stats)
            .execute(&mut conn)
            .await?;
            
        Ok(())
    }
    
    /// Update platform daily statistics
    async fn update_platform_daily_stats<F>(&self, platform_id: &str, updater: F) -> Result<()>
    where
        F: FnOnce(&mut NewPlatformDailyStatistics),
    {
        let mut conn = self.get_connection().await?;
        let today = Utc::now().date_naive();
        
        // Try to load existing stats for today and platform
        let existing_stats = schema::platform_daily_statistics::table
            .find((platform_id, today))
            .first::<crate::models::statistics::PlatformDailyStatistics>(&mut conn)
            .await
            .ok();
            
        // Create new stats or update existing
        let mut stats = match existing_stats {
            Some(existing) => NewPlatformDailyStatistics {
                platform_id: existing.platform_id,
                date: existing.date,
                active_users_count: existing.active_users_count,
                new_users_count: existing.new_users_count,
                content_created_count: existing.content_created_count,
                total_interactions_count: existing.total_interactions_count,
            },
            None => NewPlatformDailyStatistics {
                platform_id: platform_id.to_string(),
                date: today,
                active_users_count: 0,
                new_users_count: 0,
                content_created_count: 0,
                total_interactions_count: 0,
            },
        };
        
        // Apply updates to stats
        updater(&mut stats);
        
        // Insert or update stats
        diesel::insert_into(schema::platform_daily_statistics::table)
            .values(&stats)
            .on_conflict((schema::platform_daily_statistics::platform_id, schema::platform_daily_statistics::date))
            .do_update()
            .set(&stats)
            .execute(&mut conn)
            .await?;
            
        Ok(())
    }
}

#[async_trait]
impl Worker for SocialIndexerWorker {
    type Result = ();

    async fn process_checkpoint(&self, checkpoint: &CheckpointData) -> Result<()> {
        let checkpoint_seq = checkpoint.checkpoint_summary.sequence_number;
        info!("Processing checkpoint: {}", checkpoint_seq);
        
        // Process each transaction in the checkpoint
        for transaction in &checkpoint.transactions {
            // Process each event in the transaction
            for event in &transaction.events {
                let type_str = &event.type_;
                
                // Process events by module
                match type_str {
                    // Profile events
                    t if t.starts_with(MODULE_PREFIX_PROFILE) && t.ends_with("ProfileCreatedEvent") => {
                        if let Ok(event) = parse_event::<ProfileCreatedEvent>(event) {
                            if let Err(e) = self.process_profile_created(&event).await {
                                error!("Failed to process ProfileCreatedEvent: {}", e);
                            }
                        }
                    },
                    t if t.starts_with(MODULE_PREFIX_SOCIAL_GRAPH) && t.ends_with("ProfileFollowEvent") => {
                        if let Ok(event) = parse_event::<ProfileFollowEvent>(event) {
                            if let Err(e) = self.process_profile_follow(&event).await {
                                error!("Failed to process ProfileFollowEvent: {}", e);
                            }
                        }
                    },
                    
                    // Platform events
                    t if t.starts_with(MODULE_PREFIX_PLATFORM) && t.ends_with("PlatformCreatedEvent") => {
                        if let Ok(event) = parse_event::<PlatformCreatedEvent>(event) {
                            if let Err(e) = self.process_platform_created(&event).await {
                                error!("Failed to process PlatformCreatedEvent: {}", e);
                            }
                        }
                    },
                    t if t.starts_with(MODULE_PREFIX_PLATFORM) && t.ends_with("ProfileJoinedPlatformEvent") => {
                        if let Ok(event) = parse_event::<ProfileJoinedPlatformEvent>(event) {
                            if let Err(e) = self.process_profile_joined_platform(&event).await {
                                error!("Failed to process ProfileJoinedPlatformEvent: {}", e);
                            }
                        }
                    },
                    
                    // Content events
                    t if t.starts_with(MODULE_PREFIX_CONTENT) && t.ends_with("ContentCreatedEvent") => {
                        if let Ok(event) = parse_event::<ContentCreatedEvent>(event) {
                            if let Err(e) = self.process_content_created(&event).await {
                                error!("Failed to process ContentCreatedEvent: {}", e);
                            }
                        }
                    },
                    t if t.starts_with(MODULE_PREFIX_CONTENT) && t.ends_with("ContentInteractionEvent") => {
                        if let Ok(event) = parse_event::<ContentInteractionEvent>(event) {
                            if let Err(e) = self.process_content_interaction(&event).await {
                                error!("Failed to process ContentInteractionEvent: {}", e);
                            }
                        }
                    },
                    
                    // Block list events
                    t if t.starts_with(MODULE_PREFIX_BLOCK_LIST) && t.ends_with("EntityBlockedEvent") => {
                        if let Ok(event) = parse_event::<EntityBlockedEvent>(event) {
                            if let Err(e) = self.process_entity_blocked(&event).await {
                                error!("Failed to process EntityBlockedEvent: {}", e);
                            }
                        }
                    },
                    
                    // IP events
                    t if t.starts_with(MODULE_PREFIX_MY_IP) && t.ends_with("IPRegisteredEvent") => {
                        if let Ok(event) = parse_event::<IPRegisteredEvent>(event) {
                            if let Err(e) = self.process_ip_registered(&event).await {
                                error!("Failed to process IPRegisteredEvent: {}", e);
                            }
                        }
                    },
                    t if t.starts_with(MODULE_PREFIX_MY_IP) && t.ends_with("LicenseGrantedEvent") => {
                        if let Ok(event) = parse_event::<LicenseGrantedEvent>(event) {
                            if let Err(e) = self.process_license_granted(&event).await {
                                error!("Failed to process LicenseGrantedEvent: {}", e);
                            }
                        }
                    },
                    
                    // Fee distribution events
                    t if t.starts_with(MODULE_PREFIX_FEE_DISTRIBUTION) && t.ends_with("FeesDistributedEvent") => {
                        if let Ok(event) = parse_event::<FeesDistributedEvent>(event) {
                            if let Err(e) = self.process_fee_distribution(&event).await {
                                error!("Failed to process FeesDistributedEvent: {}", e);
                            }
                        }
                    },
                    
                    // Ignore other events
                    _ => {}
                }
            }
        }
        
        // Update worker progress
        self.update_progress(checkpoint_seq).await?;
        
        info!("Processed checkpoint: {}", checkpoint_seq);
        Ok(())
    }
}