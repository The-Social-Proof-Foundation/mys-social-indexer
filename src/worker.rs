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
    MODULE_PREFIX_SOCIAL_GRAPH,
    ProfileCreatedEvent, ProfileUpdatedEvent, UsernameUpdatedEvent, UsernameRegisteredEvent, 
    PlatformCreatedEvent, ContentCreatedEvent, ContentInteractionEvent,
    EntityBlockedEvent, IPRegisteredEvent, LicenseGrantedEvent, ProofCreatedEvent,
    FeeModelCreatedEvent, FeesDistributedEvent, ProfileFollowEvent, ProfileJoinedPlatformEvent,
    FollowEvent, UnfollowEvent,
    PlatformBlockedProfileEvent, PlatformUnblockedProfileEvent, UserJoinedPlatformEvent, UserLeftPlatformEvent,
};
use crate::models::profile::{NewProfile, NewFollow, NewProfilePlatformLink, UpdateProfile};
use crate::models::username::{NewUsername, UpdateUsername, NewUsernameHistory};
// These model imports will be added when we implement these features
//use crate::models::platform::NewPlatform;
//use crate::models::content::{NewContent, NewContentInteraction};
//use crate::models::block_list::NewBlock;
//use crate::models::intellectual_property::{NewIntellectualProperty, NewIPLicense, NewProofOfCreativity};
//use crate::models::fee_distribution::{NewFeeModel, NewFeeDistribution, NewFeeRecipient, NewFeeRecipientPayment};
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
        
        info!("Processing ProfileCreatedEvent: profile_id={}, username={:?}", 
              event.profile_id, event.username);
        
        // Convert event to database model
        let new_profile = event.into_model()?;
        
        // Insert the profile
        let result = diesel::insert_into(schema::profiles::table)
            .values(&new_profile)
            .on_conflict(schema::profiles::id)
            .do_update()
            .set(&new_profile)
            .returning(schema::profiles::id) // Return the profile ID
            .get_result::<i32>(&mut conn)
            .await?;
            
        let profile_id = result; // This is the newly created profile's ID
        
        // If the profile has a username, add it to the usernames table
        if let Some(username) = &event.username {
            info!("Profile has username: {}, adding to usernames table", username);
            
            // Check if the username already exists in the usernames table
            let username_exists = schema::usernames::table
                .filter(schema::usernames::profile_id.eq(profile_id))
                .filter(schema::usernames::username.eq(username))
                .first::<crate::models::username::Username>(&mut conn)
                .await.is_ok();
                
            if !username_exists {
                // Convert timestamp to NaiveDateTime
                let registered_at = DateTime::from_timestamp(event.created_at as i64, 0)
                    .unwrap_or(Utc::now())
                    .naive_utc();
                
                // Create a new username record
                let new_username = NewUsername {
                    profile_id,
                    username: username.clone(),
                    registered_at,
                    updated_at: registered_at,
                };
                
                // Insert the username
                info!("Inserting username record into usernames table");
                match diesel::insert_into(schema::usernames::table)
                    .values(&new_username)
                    .execute(&mut conn)
                    .await {
                    Ok(rows) => info!("Successfully inserted {} username record(s) for: {}", rows, username),
                    Err(e) => error!("Failed to insert username record: {}", e)
                };
                
                // Verify the insertion worked
                info!("Verifying username insertion");
                match schema::usernames::table
                    .filter(schema::usernames::profile_id.eq(profile_id))
                    .filter(schema::usernames::username.eq(username))
                    .first::<crate::models::username::Username>(&mut conn)
                    .await {
                    Ok(username_rec) => info!("Verified username record exists: id={}, username={}", username_rec.id, username_rec.username),
                    Err(e) => error!("Username record not found after insertion: {}", e)
                }
            } else {
                info!("Username already exists in usernames table for this profile");
            }
        } else {
            info!("Profile doesn't have a username, skipping usernames table insertion");
        }
            
        // Update daily statistics
        self.update_daily_stats(|stats| {
            stats.new_profiles_count += 1;
        }).await?;
        
        info!("Processed profile created: {}", event.profile_id);
        Ok(())
    }
    
    /// Process a profile updated event
    async fn process_profile_updated(&self, event: &ProfileUpdatedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Find the profile by profile_id
        let profile = schema::profiles::table
            .filter(schema::profiles::profile_id.eq(&event.profile_id))
            .first::<crate::models::profile::Profile>(&mut conn)
            .await?;
        
        // Log all fields for debugging
        info!("Processing ProfileUpdatedEvent:");
        info!("  profile_id: {}", event.profile_id);
        info!("  display_name: {:?}", event.display_name);
        info!("  bio: {:?}", event.bio);
        info!("  profile_photo: {:?}", event.profile_photo);
        info!("  cover_photo: {:?}", event.cover_photo);
        info!("  website: {:?}", event.website);
        
        // For existing profile in database:
        info!("Existing profile in database:");
        info!("  id: {}", profile.id);
        info!("  display_name: {:?}", profile.display_name);
        info!("  bio: {:?}", profile.bio);
        info!("  profile_photo: {:?}", profile.profile_photo);
        info!("  cover_photo: {:?}", profile.cover_photo);
        info!("  website: {:?}", profile.website);
        
        // Create an update model - use existing values when event doesn't provide them
        // Use website field from event if provided, otherwise keep existing
        
        let update = UpdateProfile {
            display_name: event.display_name.clone(),
            bio: if event.bio.is_some() { event.bio.clone() } else { profile.bio.clone() },
            profile_photo: if event.profile_photo.is_some() { event.profile_photo.clone() } else { profile.profile_photo.clone() },
            website: event.website.clone(),  // Use new website field from event
            cover_photo: if event.cover_photo.is_some() { event.cover_photo.clone() } else { profile.cover_photo.clone() },
            sensitive_data_updated_at: Some(DateTime::from_timestamp(event.updated_at as i64, 0)
                .unwrap_or(Utc::now())
                .naive_utc()),
            // Include all sensitive fields from the event
            birthdate: event.birthdate.clone(),
            current_location: event.current_location.clone(),
            raised_location: event.raised_location.clone(),
            phone: event.phone.clone(),
            email: event.email.clone(),
            gender: event.gender.clone(),
            political_view: event.political_view.clone(),
            religion: event.religion.clone(),
            education: event.education.clone(),
            primary_language: event.primary_language.clone(),
            relationship_status: event.relationship_status.clone(),
            x_username: event.x_username.clone(),
            mastodon_username: event.mastodon_username.clone(),
            facebook_username: event.facebook_username.clone(),
            reddit_username: event.reddit_username.clone(),
            github_username: event.github_username.clone(),
        };
        
        info!("Updating profile with:");
        info!("  display_name: {:?}", update.display_name);
        info!("  bio: {:?}", update.bio);
        info!("  profile_photo: {:?}", update.profile_photo);
        info!("  website: {:?}", update.website);
        info!("  cover_photo: {:?}", update.cover_photo);
        
        // Update the profile
        diesel::update(schema::profiles::table.find(profile.id))
            .set(&update)
            .execute(&mut conn)
            .await?;
            
        info!("Processed profile updated: {}", event.profile_id);
        Ok(())
    }
    
    /// Process a username updated event
    async fn process_username_updated(&self, event: &UsernameUpdatedEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        // Find the profile by profile_id
        let profile = schema::profiles::table
            .filter(schema::profiles::profile_id.eq(&event.profile_id))
            .first::<crate::models::profile::Profile>(&mut conn)
            .await?;
        
        // Update the profile table's username column (for backward compatibility)
        diesel::update(schema::profiles::table.find(profile.id))
            .set(schema::profiles::username.eq(&event.new_username))
            .execute(&mut conn)
            .await?;
        
        // Check if the username exists in the usernames table
        let username_result = schema::usernames::table
            .filter(schema::usernames::profile_id.eq(profile.id))
            .first::<crate::models::username::Username>(&mut conn)
            .await;
            
        let now = Utc::now().naive_utc();
        
        // If the username record exists, update it
        if let Ok(username) = username_result {
            // Update the username in the usernames table
            diesel::update(schema::usernames::table.find(username.id))
                .set(UpdateUsername {
                    username: Some(event.new_username.clone()),
                    updated_at: Some(now),
                })
                .execute(&mut conn)
                .await?;
        } else {
            // If username doesn't exist, create a new record
            let new_username = NewUsername {
                profile_id: profile.id,
                username: event.new_username.clone(),
                registered_at: now,
                updated_at: now,
            };
            
            diesel::insert_into(schema::usernames::table)
                .values(&new_username)
                .execute(&mut conn)
                .await?;
        }
        
        // Create a history record of the username change
        let history_record = NewUsernameHistory {
            profile_id: profile.id,
            old_username: event.old_username.clone(),
            new_username: event.new_username.clone(),
            changed_at: now,
        };
        
        diesel::insert_into(schema::username_history::table)
            .values(&history_record)
            .execute(&mut conn)
            .await?;
            
        info!("Processed username updated: {} -> {}", event.old_username, event.new_username);
        Ok(())
    }
    
    /// Process a username registered event
    async fn process_username_registered(&self, event: &UsernameRegisteredEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        
        info!("Processing UsernameRegisteredEvent: {:?}", event);
        
        // Find the profile by profile_id
        let profile_result = schema::profiles::table
            .filter(schema::profiles::profile_id.eq(&event.profile_id))
            .first::<crate::models::profile::Profile>(&mut conn)
            .await;
        
        match profile_result {
            Ok(profile) => {
                info!("Found profile with ID: {} for username: {}", profile.id, event.username);
                
                // Update the profile table's username column (for backward compatibility)
                diesel::update(schema::profiles::table.find(profile.id))
                    .set(schema::profiles::username.eq(&event.username))
                    .execute(&mut conn)
                    .await?;
                    
                // Check if the username already exists in the usernames table
                let username_exists = schema::usernames::table
                    .filter(schema::usernames::profile_id.eq(profile.id))
                    .filter(schema::usernames::username.eq(&event.username))
                    .first::<crate::models::username::Username>(&mut conn)
                    .await.is_ok();
                
                // Get timestamp from event or create a default one
                let now = if event.registered_at > 0 {
                    chrono::DateTime::from_timestamp(event.registered_at as i64, 0)
                        .unwrap_or(Utc::now())
                        .naive_utc()
                } else {
                    Utc::now().naive_utc()
                };
                    
                // Only insert if it doesn't exist
                if !username_exists {
                    info!("Username doesn't exist in the usernames table, inserting new record");
                    
                    let new_username = NewUsername {
                        profile_id: profile.id,
                        username: event.username.clone(),
                        registered_at: now,
                        updated_at: now,
                    };
                    
                    // Insert the username
                    let result = diesel::insert_into(schema::usernames::table)
                        .values(&new_username)
                        .execute(&mut conn)
                        .await;
                        
                    match result {
                        Ok(_) => info!("Successfully inserted username record"),
                        Err(e) => error!("Failed to insert username record: {}", e)
                    }
                    
                    // Verify the username was inserted correctly
                    match schema::usernames::table
                        .filter(schema::usernames::profile_id.eq(profile.id))
                        .filter(schema::usernames::username.eq(&event.username))
                        .first::<crate::models::username::Username>(&mut conn)
                        .await {
                        Ok(username) => info!("Verified username record exists: id={}, username={}", username.id, username.username),
                        Err(e) => error!("Failed to verify username record: {}", e)
                    }
                } else {
                    info!("Username already exists in the usernames table, skipping insertion");
                }
            },
            Err(_) => {
                // Profile doesn't exist yet, likely because events are processed out of order
                warn!("Profile not found for profile_id: {}. UsernameRegisteredEvent will be handled when profile is created", event.profile_id);
                
                // Try to find a profile with a matching username
                let profile_by_username = schema::profiles::table
                    .filter(schema::profiles::username.eq(&event.username))
                    .first::<crate::models::profile::Profile>(&mut conn)
                    .await;
                
                if let Ok(profile) = profile_by_username {
                    info!("Found profile with username: {}, using that instead", event.username);
                    
                    // Create a new username record
                    let now = Utc::now().naive_utc();
                    let new_username = NewUsername {
                        profile_id: profile.id,
                        username: event.username.clone(),
                        registered_at: now,
                        updated_at: now,
                    };
                    
                    // Try to insert the username for this profile
                    match diesel::insert_into(schema::usernames::table)
                        .values(&new_username)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)
                        .await {
                        Ok(_) => info!("Created username record for existing profile with matching username"),
                        Err(e) => error!("Failed to create username record: {}", e)
                    }
                } else {
                    warn!("No profile found with username {}. Event will be processed when profile is created", event.username);
                }
            }
        }
        
        info!("Processed username registered: {} for profile {}", event.username, event.profile_id);
        Ok(())
    }
    
    // Private data update functionality has been removed
    // All sensitive fields are now stored directly in the profile
    
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

    /// Process a platform blocked profile event
    async fn process_platform_blocked_profile(&self, event: &PlatformBlockedProfileEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let now = Utc::now().naive_utc();
        
        // Create new blocked profile record
        let new_blocked_profile = NewPlatformBlockedProfile {
            platform_id: event.platform_id.clone(),
            profile_id: event.profile_id.clone(),
            blocked_by: event.blocked_by.clone(),
            created_at: now,
            is_blocked: true,
        };
        
        // Insert the blocked profile record
        diesel::insert_into(schema::platform_blocked_profiles::table)
            .values(&new_blocked_profile)
            .execute(&mut conn)
            .await?;
            
        info!("Processed platform blocked profile: platform={}, profile={}", 
              event.platform_id, event.profile_id);
        Ok(())
    }
    
    /// Process a platform unblocked profile event
    async fn process_platform_unblocked_profile(&self, event: &PlatformUnblockedProfileEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let now = Utc::now().naive_utc();
        
        // Delete the blocked profile record
        diesel::delete(schema::platform_blocked_profiles::table)
            .filter(schema::platform_blocked_profiles::platform_id.eq(&event.platform_id))
            .filter(schema::platform_blocked_profiles::profile_id.eq(&event.profile_id))
            .execute(&mut conn)
            .await?;
            
        info!("Processed platform unblocked profile: platform={}, profile={}", 
              event.platform_id, event.profile_id);
        Ok(())
    }
    
    /// Process a user joined platform event
    async fn process_user_joined_platform(&self, event: &UserJoinedPlatformEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let now = Utc::now().naive_utc();
        
        // Create new platform relationship record
        let new_relationship = NewPlatformRelationship {
            platform_id: event.platform_id.clone(),
            profile_id: event.profile_id.clone(),
            joined_at: now,
            left_at: None,
        };
        
        // Insert the platform relationship record
        diesel::insert_into(schema::platform_relationships::table)
            .values(&new_relationship)
            .execute(&mut conn)
            .await?;
            
        info!("Processed user joined platform: platform={}, profile={}", 
              event.platform_id, event.profile_id);
        Ok(())
    }
    
    /// Process a user left platform event
    async fn process_user_left_platform(&self, event: &UserLeftPlatformEvent) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let now = Utc::now().naive_utc();
        
        // Update the platform relationship record to set left_at
        let update = UpdatePlatformRelationship {
            left_at: Some(now),
        };
        
        diesel::update(schema::platform_relationships::table)
            .filter(schema::platform_relationships::platform_id.eq(&event.platform_id))
            .filter(schema::platform_relationships::profile_id.eq(&event.profile_id))
            .filter(schema::platform_relationships::left_at.is_null())
            .set(&update)
            .execute(&mut conn)
            .await?;
            
        info!("Processed user left platform: platform={}, profile={}", 
              event.platform_id, event.profile_id);
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
                
                // Log all events for debugging
                info!("Processing event of type: {}", type_str);
                
                // Process events by module
                match type_str {
                    // Profile events
                    t if t.starts_with(MODULE_PREFIX_PROFILE) && t.ends_with("ProfileCreatedEvent") => {
                        // Log the raw event for better debugging
                        info!("Raw ProfileCreatedEvent data: {}", serde_json::to_string_pretty(&event).unwrap_or_default());
                        
                        match parse_event::<ProfileCreatedEvent>(event) {
                            Ok(event) => {
                                info!("Successfully parsed ProfileCreatedEvent with fields:");
                                info!("  profile_id: {}", event.profile_id);
                                info!("  owner_address: {}", event.owner_address);
                                info!("  username: {:?}", event.username);
                                info!("  display_name: {}", event.display_name);
                                info!("  bio: {:?}", event.bio);
                                info!("  profile_photo: {:?}", event.profile_photo);
                                info!("  cover_photo: {:?}", event.cover_photo);
                                
                                if let Err(e) = self.process_profile_created(&event).await {
                                    error!("Failed to process ProfileCreatedEvent: {}", e);
                                }
                            },
                            Err(e) => {
                                error!("Failed to parse ProfileCreatedEvent: {}", e);
                                // Log full event for debugging
                                error!("Event data: {}", serde_json::to_string_pretty(event).unwrap_or_default());
                            }
                        }
                    },
                    t if t.starts_with(MODULE_PREFIX_PROFILE) && t.ends_with("ProfileUpdatedEvent") => {
                        // Log the raw event for better debugging
                        info!("Raw ProfileUpdatedEvent data: {}", serde_json::to_string_pretty(&event).unwrap_or_default());
                        
                        match parse_event::<ProfileUpdatedEvent>(event) {
                            Ok(event) => {
                                info!("Successfully parsed ProfileUpdatedEvent with fields:");
                                info!("  profile_id: {}", event.profile_id);
                                info!("  owner_address: {}", event.owner_address);
                                info!("  username: {:?}", event.username);
                                info!("  display_name: {:?}", event.display_name);
                                info!("  bio: {:?}", event.bio);
                                info!("  profile_photo: {:?}", event.profile_photo);
                                info!("  cover_photo: {:?}", event.cover_photo);
                                
                                if let Err(e) = self.process_profile_updated(&event).await {
                                    error!("Failed to process ProfileUpdatedEvent: {}", e);
                                }
                            },
                            Err(e) => {
                                error!("Failed to parse ProfileUpdatedEvent: {}", e);
                                // Log full event for debugging
                                error!("Event data: {}", serde_json::to_string_pretty(event).unwrap_or_default());
                            }
                        }
                    },
                    t if t.starts_with(MODULE_PREFIX_PROFILE) && t.ends_with("UsernameUpdatedEvent") => {
                        if let Ok(event) = parse_event::<UsernameUpdatedEvent>(event) {
                            if let Err(e) = self.process_username_updated(&event).await {
                                error!("Failed to process UsernameUpdatedEvent: {}", e);
                            }
                        }
                    },
                    t if t.starts_with(MODULE_PREFIX_PROFILE) && t.ends_with("UsernameRegisteredEvent") => {
                        info!("Found a UsernameRegisteredEvent: {}", serde_json::to_string_pretty(event).unwrap_or_default());
                        match parse_event::<UsernameRegisteredEvent>(event) {
                            Ok(event) => {
                                info!("Successfully parsed UsernameRegisteredEvent: profile_id={}, username={}", 
                                       event.profile_id, event.username);
                                
                                if let Err(e) = self.process_username_registered(&event).await {
                                    error!("Failed to process UsernameRegisteredEvent: {}", e);
                                }
                            },
                            Err(e) => {
                                error!("Failed to parse UsernameRegisteredEvent: {}", e);
                                // Dump the full event for debugging
                                error!("Raw event data: {}", serde_json::to_string_pretty(event).unwrap_or_default());
                            }
                        }
                    },
                    // Private data update functionality has been removed
                    // All sensitive fields are now stored directly in the profile
                    t if t.starts_with(MODULE_PREFIX_SOCIAL_GRAPH) && t.ends_with("ProfileFollowEvent") => {
                        if let Ok(event) = parse_event::<ProfileFollowEvent>(event) {
                            if let Err(e) = self.process_profile_follow(&event).await {
                                error!("Failed to process ProfileFollowEvent: {}", e);
                            }
                        }
                    },
                    
                    // Social Graph events from social_graph module
                    t if t.starts_with(MODULE_PREFIX_SOCIAL_GRAPH) && t.ends_with("FollowEvent") => {
                        info!("Processing social graph FollowEvent");
                        if let Ok(event) = parse_event::<FollowEvent>(event) {
                            // Create a database connection
                            let mut conn = match self.get_connection().await {
                                Ok(conn) => conn,
                                Err(e) => {
                                    error!("Failed to get database connection: {}", e);
                                    continue;
                                }
                            };
                            
                            // Get profile IDs from addresses
                            let follower_profile = match schema::profiles::table
                                .filter(schema::profiles::owner_address.eq(&event.follower))
                                .select((schema::profiles::id, schema::profiles::owner_address))
                                .first::<(i32, String)>(&mut conn)
                                .await {
                                Ok(profile) => profile,
                                Err(e) => {
                                    error!("Failed to find follower profile for address {}: {}", event.follower, e);
                                    continue;
                                }
                            };
                                
                            let following_profile = match schema::profiles::table
                                .filter(schema::profiles::owner_address.eq(&event.following))
                                .select((schema::profiles::id, schema::profiles::owner_address))
                                .first::<(i32, String)>(&mut conn)
                                .await {
                                Ok(profile) => profile,
                                Err(e) => {
                                    error!("Failed to find following profile for address {}: {}", event.following, e);
                                    continue;
                                }
                            };
                            
                            // Create relationship
                            let relationship = match event.into_relationship(follower_profile.0, following_profile.0) {
                                Ok(rel) => rel,
                                Err(e) => {
                                    error!("Failed to create relationship: {}", e);
                                    continue;
                                }
                            };
                            
                            // Check if relationship already exists
                            let existing = match schema::social_graph_relationships::table
                                .filter(schema::social_graph_relationships::follower_id.eq(follower_profile.0))
                                .filter(schema::social_graph_relationships::following_id.eq(following_profile.0))
                                .count()
                                .get_result::<i64>(&mut conn)
                                .await {
                                Ok(count) => count > 0,
                                Err(e) => {
                                    error!("Failed to check existing relationship: {}", e);
                                    continue;
                                }
                            };
                                
                            if existing {
                                info!("Follow relationship already exists between {} and {}", 
                                    event.follower, event.following);
                                continue;
                            }
                                
                            // Start a transaction for atomicity
                            let result = conn.build_transaction()
                                .run(|mut conn| Box::pin(async move {
                                    // Insert relationship
                                    diesel::insert_into(schema::social_graph_relationships::table)
                                        .values(&relationship)
                                        .execute(&mut conn)
                                        .await?;
                                        
                                    // Update follower's following count (increment)
                                    diesel::sql_query(format!(
                                        "UPDATE profiles SET following_count = following_count + 1 WHERE id = {}", 
                                        follower_profile.0
                                    ))
                                    .execute(&mut conn)
                                    .await?;
                                    
                                    // Update followed's followers count (increment)
                                    diesel::sql_query(format!(
                                        "UPDATE profiles SET followers_count = followers_count + 1 WHERE id = {}", 
                                        following_profile.0
                                    ))
                                    .execute(&mut conn)
                                    .await?;
                                    
                                    Result::<_, diesel::result::Error>::Ok(())
                                }))
                                .await;
                                
                            if let Err(e) = result {
                                error!("Failed to process follow event transaction: {}", e);
                            } else {
                                info!("Processed follow event: {} is now following {}", 
                                    event.follower, event.following);
                            }
                        }
                    },
                    
                    t if t.starts_with(MODULE_PREFIX_SOCIAL_GRAPH) && t.ends_with("UnfollowEvent") => {
                        info!("Processing social graph UnfollowEvent");
                        if let Ok(event) = parse_event::<UnfollowEvent>(event) {
                            // Create a database connection
                            let mut conn = match self.get_connection().await {
                                Ok(conn) => conn,
                                Err(e) => {
                                    error!("Failed to get database connection: {}", e);
                                    continue;
                                }
                            };
                            
                            // Get profile IDs from addresses
                            let follower_profile = match schema::profiles::table
                                .filter(schema::profiles::owner_address.eq(&event.follower))
                                .select((schema::profiles::id, schema::profiles::owner_address))
                                .first::<(i32, String)>(&mut conn)
                                .await {
                                Ok(profile) => profile,
                                Err(e) => {
                                    error!("Failed to find follower profile for address {}: {}", event.follower, e);
                                    continue;
                                }
                            };
                                
                            let unfollowed_profile = match schema::profiles::table
                                .filter(schema::profiles::owner_address.eq(&event.unfollowed))
                                .select((schema::profiles::id, schema::profiles::owner_address))
                                .first::<(i32, String)>(&mut conn)
                                .await {
                                Ok(profile) => profile,
                                Err(e) => {
                                    error!("Failed to find unfollowed profile for address {}: {}", event.unfollowed, e);
                                    continue;
                                }
                            };
                            
                            // Check if relationship exists
                            let relationship = match schema::social_graph_relationships::table
                                .filter(schema::social_graph_relationships::follower_id.eq(follower_profile.0))
                                .filter(schema::social_graph_relationships::following_id.eq(unfollowed_profile.0))
                                .select(schema::social_graph_relationships::id)
                                .first::<i32>(&mut conn)
                                .await {
                                Ok(id) => id,
                                Err(diesel::result::Error::NotFound) => {
                                    info!("Follow relationship does not exist between {} and {}", 
                                        event.follower, event.unfollowed);
                                    continue;
                                },
                                Err(e) => {
                                    error!("Failed to check existing relationship: {}", e);
                                    continue;
                                }
                            };
                                
                            // Start a transaction for atomicity
                            let result = conn.build_transaction()
                                .run(|mut conn| Box::pin(async move {
                                    // Delete the relationship
                                    diesel::delete(schema::social_graph_relationships::table
                                        .filter(schema::social_graph_relationships::id.eq(relationship)))
                                        .execute(&mut conn)
                                        .await?;
                                        
                                    // Update follower's following count (decrement)
                                    diesel::sql_query(format!(
                                        "UPDATE profiles SET following_count = GREATEST(0, following_count - 1) WHERE id = {}", 
                                        follower_profile.0
                                    ))
                                    .execute(&mut conn)
                                    .await?;
                                    
                                    // Update unfollowed's followers count (decrement)
                                    diesel::sql_query(format!(
                                        "UPDATE profiles SET followers_count = GREATEST(0, followers_count - 1) WHERE id = {}", 
                                        unfollowed_profile.0
                                    ))
                                    .execute(&mut conn)
                                    .await?;
                                    
                                    Result::<_, diesel::result::Error>::Ok(())
                                }))
                                .await;
                                
                            if let Err(e) = result {
                                error!("Failed to process unfollow event transaction: {}", e);
                            } else {
                                info!("Processed unfollow event: {} unfollowed {}", 
                                    event.follower, event.unfollowed);
                            }
                        }
                    },
                    
                    // Platform events
                    t if t.starts_with(MODULE_PREFIX_PLATFORM) => {
                        match type_str {
                            t if t.ends_with("PlatformBlockedProfileEvent") => {
                                match parse_event::<PlatformBlockedProfileEvent>(event) {
                                    Ok(event) => self.process_platform_blocked_profile(&event).await?,
                                    Err(e) => error!("Failed to parse PlatformBlockedProfileEvent: {}", e),
                                }
                            }
                            t if t.ends_with("PlatformUnblockedProfileEvent") => {
                                match parse_event::<PlatformUnblockedProfileEvent>(event) {
                                    Ok(event) => self.process_platform_unblocked_profile(&event).await?,
                                    Err(e) => error!("Failed to parse PlatformUnblockedProfileEvent: {}", e),
                                }
                            }
                            t if t.ends_with("UserJoinedPlatformEvent") => {
                                match parse_event::<UserJoinedPlatformEvent>(event) {
                                    Ok(event) => self.process_user_joined_platform(&event).await?,
                                    Err(e) => error!("Failed to parse UserJoinedPlatformEvent: {}", e),
                                }
                            }
                            t if t.ends_with("UserLeftPlatformEvent") => {
                                match parse_event::<UserLeftPlatformEvent>(event) {
                                    Ok(event) => self.process_user_left_platform(&event).await?,
                                    Err(e) => error!("Failed to parse UserLeftPlatformEvent: {}", e),
                                }
                            }
                            _ => {
                                debug!("Unhandled platform event type: {}", type_str);
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