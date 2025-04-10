// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

// Import diesel table macros
use diesel::table;
use diesel::allow_tables_to_appear_in_same_query;

// Define profile table with all fields including encrypted ones directly in the table
table! {
    profiles (id) {
        id -> Integer,
        owner_address -> Varchar,
        username -> Varchar,
        display_name -> Nullable<Varchar>,
        bio -> Nullable<Text>,
        profile_photo -> Nullable<Varchar>,
        website -> Nullable<Text>,           // Website field from contract
        created_at -> Timestamp,
        updated_at -> Timestamp,
        cover_photo -> Nullable<Varchar>,
        profile_id -> Nullable<Varchar>,
        sensitive_data_updated_at -> Nullable<Timestamp>,
        // Followers count - updated when follow/unfollow occurs
        followers_count -> Integer,
        // Following count - updated when follow/unfollow occurs
        following_count -> Integer,
        // Sensitive fields (client-side encrypted)
        birthdate -> Nullable<Text>,
        current_location -> Nullable<Text>,
        raised_location -> Nullable<Text>,
        phone -> Nullable<Text>,
        email -> Nullable<Text>,
        gender -> Nullable<Text>,
        political_view -> Nullable<Text>,
        religion -> Nullable<Text>,
        education -> Nullable<Text>,
        primary_language -> Nullable<Text>,
        relationship_status -> Nullable<Text>,
        x_username -> Nullable<Text>,
        mastodon_username -> Nullable<Text>,
        facebook_username -> Nullable<Text>,
        reddit_username -> Nullable<Text>,
        github_username -> Nullable<Text>,
        // Block list address
        block_list_address -> Nullable<Varchar>,
    }
}

// Define social graph relationships table
// This is a highly optimized junction table for handling follows/followers
// Now uses blockchain addresses directly to avoid database ID references
table! {
    social_graph_relationships (id) {
        id -> Integer,
        // Blockchain address for the follower
        follower_address -> Varchar,
        // Blockchain address for the followed user
        following_address -> Varchar,
        // When the relationship was created
        created_at -> Timestamp,
    }
}

// Define social graph events table for tracking all follow/unfollow actions
table! {
    social_graph_events (id) {
        id -> Integer,
        event_type -> Varchar,
        follower_address -> Varchar,
        following_address -> Varchar,
        created_at -> Timestamp,
        event_id -> Nullable<Varchar>,  // Changed from blockchain_tx_hash to event_id
        raw_event_data -> Nullable<Jsonb>,
    }
}

// Define indexer progress table
table! {
    indexer_progress (id) {
        id -> Varchar,
        last_checkpoint_processed -> Bigint,
        last_processed_at -> Timestamp,
    }
}

// Define platforms table
table! {
    platforms (id) {
        id -> Integer,
        platform_id -> Varchar,
        name -> Varchar,
        tagline -> Varchar,
        description -> Nullable<Text>,
        logo -> Nullable<Varchar>,
        developer_address -> Varchar,
        terms_of_service -> Nullable<Text>,
        privacy_policy -> Nullable<Text>,
        #[sql_name = "platforms"]
        platform_names -> Nullable<Jsonb>,
        links -> Nullable<Jsonb>,
        status -> SmallInt,
        release_date -> Nullable<Varchar>,
        shutdown_date -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        is_approved -> Bool,
        approval_changed_at -> Nullable<Timestamp>,
        approved_by -> Nullable<Varchar>,
    }
}

// Define platform_moderators table
table! {
    platform_moderators (id) {
        id -> Integer,
        platform_id -> Varchar,
        moderator_address -> Varchar,
        added_by -> Varchar,
        created_at -> Timestamp,
    }
}

// Define platform_blocked_profiles table
table! {
    platform_blocked_profiles (id) {
        id -> Integer,
        platform_id -> Varchar,
        profile_id -> Varchar,
        blocked_by -> Varchar,
        created_at -> Timestamp,
    }
}

// Define platform_events table
table! {
    platform_events (id) {
        id -> Integer,
        event_type -> Varchar,
        platform_id -> Varchar,
        event_data -> Jsonb,
        event_id -> Nullable<Varchar>,
        created_at -> Timestamp,
    }
}

// Define platform_memberships table
table! {
    platform_memberships (id) {
        id -> Integer,
        platform_id -> Varchar,
        profile_id -> Varchar,
        joined_at -> Timestamp,
    }
}

// Note: platform_relationships table has been removed in favor of platform_memberships

// Profile blocking table
table! {
    profiles_blocked (id) {
        id -> Integer,
        blocker_wallet_address -> Varchar,
        blocked_address -> Varchar,
        created_at -> Timestamp,
    }
}

// Profile events table
table! {
    profile_events (id) {
        id -> Integer,
        event_type -> Varchar,
        profile_id -> Varchar,
        event_data -> Jsonb,
        event_id -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

// Allow joining the tables if needed
allow_tables_to_appear_in_same_query!(
    profiles,
    social_graph_relationships,
    social_graph_events,
    indexer_progress,
    platforms,
    platform_moderators,
    platform_blocked_profiles,
    platform_events,
    platform_memberships,
    profiles_blocked,
    profile_events,
);