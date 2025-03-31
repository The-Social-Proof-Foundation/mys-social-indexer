// Copyright (c) MySocial Team
// SPDX-License-Identifier: Apache-2.0

// Import diesel table macros
use diesel::table;
use diesel::allow_tables_to_appear_in_same_query;

// Define profile table
table! {
    profiles (id) {
        id -> Integer,
        owner_address -> Varchar,
        username -> Varchar,
        display_name -> Nullable<Varchar>,
        bio -> Nullable<Text>,
        avatar_url -> Nullable<Varchar>,
        website_url -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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

// Allow joining the tables if needed
allow_tables_to_appear_in_same_query!(
    profiles,
    indexer_progress,
);