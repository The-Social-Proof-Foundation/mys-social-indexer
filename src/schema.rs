// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (blocker_id, blocked_id) {
        blocker_id -> Text,
        blocked_id -> Text,
        blocker_type -> Int4,
        reason -> Nullable<Text>,
        blocked_at -> Timestamp,
    }
}

diesel::table! {
    content (id) {
        id -> Text,
        creator_id -> Text,
        platform_id -> Text,
        content_type -> Text,
        parent_id -> Nullable<Text>,
        created_at -> Timestamp,
        has_ip_registered -> Bool,
        view_count -> Int4,
        like_count -> Int4,
        comment_count -> Int4,
        share_count -> Int4,
    }
}

diesel::table! {
    content_interactions (profile_id, content_id, interaction_type) {
        profile_id -> Text,
        content_id -> Text,
        interaction_type -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    daily_statistics (date) {
        date -> Date,
        new_profiles_count -> Int4,
        active_profiles_count -> Int4,
        new_content_count -> Int4,
        total_interactions_count -> Int4,
        new_ip_registrations_count -> Int4,
        new_licenses_count -> Int4,
        total_fees_distributed -> Int8,
    }
}

diesel::table! {
    fee_distributions (id) {
        id -> Int4,
        fee_model_id -> Text,
        transaction_amount -> Int8,
        total_fee_amount -> Int8,
        token_type -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fee_models (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        model_type -> Int4,
        fee_amount -> Int8,
        total_split_basis_points -> Int4,
        owner_address -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fee_recipient_payments (distribution_id, recipient_id) {
        distribution_id -> Int4,
        recipient_id -> Text,
        amount -> Int8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    fee_recipients (id) {
        id -> Text,
        recipient_address -> Text,
        recipient_name -> Nullable<Text>,
        total_collected -> Int8,
    }
}

diesel::table! {
    follows (follower_id, following_id) {
        follower_id -> Text,
        following_id -> Text,
        followed_at -> Timestamp,
    }
}

diesel::table! {
    indexer_progress (id) {
        id -> Text,
        last_checkpoint_processed -> Int8,
        last_processed_at -> Timestamp,
    }
}

diesel::table! {
    intellectual_property (id) {
        id -> Text,
        creator_id -> Text,
        title -> Text,
        description -> Nullable<Text>,
        ip_type -> Int4,
        content_hash -> Nullable<Text>,
        created_at -> Timestamp,
        royalty_basis_points -> Nullable<Int4>,
        registered_countries -> Array<Text>,
        ipo_tokenized -> Bool,
        total_licenses_count -> Int4,
        active_licenses_count -> Int4,
        total_revenue -> Int8,
    }
}

diesel::table! {
    ip_licenses (id) {
        id -> Text,
        ip_id -> Text,
        licensee_id -> Text,
        license_type -> Int4,
        terms -> Nullable<Text>,
        granted_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
        status -> Int4,
        payment_amount -> Int8,
    }
}

diesel::table! {
    platform_daily_statistics (platform_id, date) {
        platform_id -> Text,
        date -> Date,
        active_users_count -> Int4,
        new_users_count -> Int4,
        content_created_count -> Int4,
        total_interactions_count -> Int4,
    }
}

diesel::table! {
    platforms (id) {
        id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        creator_address -> Text,
        created_at -> Timestamp,
        active_users_count -> Int4,
        total_users_count -> Int4,
        content_count -> Int4,
        last_activity_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    profile_platform_links (profile_id, platform_id) {
        profile_id -> Text,
        platform_id -> Text,
        joined_at -> Timestamp,
        last_active_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    profiles (id) {
        id -> Text,
        owner_address -> Text,
        username -> Nullable<Text>,
        display_name -> Nullable<Text>,
        bio -> Nullable<Text>,
        created_at -> Timestamp,
        last_activity_at -> Nullable<Timestamp>,
        followers_count -> Int4,
        following_count -> Int4,
        content_count -> Int4,
        platforms_joined -> Int4,
    }
}

diesel::table! {
    proof_of_creativity (id) {
        id -> Text,
        creator_id -> Text,
        ip_id -> Nullable<Text>,
        title -> Text,
        proof_type -> Int4,
        verification_state -> Int4,
        verified_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(content -> platforms (platform_id));
diesel::joinable!(content -> profiles (creator_id));
diesel::joinable!(content_interactions -> content (content_id));
diesel::joinable!(content_interactions -> profiles (profile_id));
diesel::joinable!(fee_distributions -> fee_models (fee_model_id));
diesel::joinable!(fee_recipient_payments -> fee_distributions (distribution_id));
diesel::joinable!(fee_recipient_payments -> fee_recipients (recipient_id));
diesel::joinable!(intellectual_property -> profiles (creator_id));
diesel::joinable!(ip_licenses -> intellectual_property (ip_id));
diesel::joinable!(ip_licenses -> profiles (licensee_id));
diesel::joinable!(platform_daily_statistics -> platforms (platform_id));
diesel::joinable!(profile_platform_links -> platforms (platform_id));
diesel::joinable!(profile_platform_links -> profiles (profile_id));
diesel::joinable!(proof_of_creativity -> intellectual_property (ip_id));
diesel::joinable!(proof_of_creativity -> profiles (creator_id));

diesel::allow_tables_to_appear_in_same_query!(
    blocks,
    content,
    content_interactions,
    daily_statistics,
    fee_distributions,
    fee_models,
    fee_recipient_payments,
    fee_recipients,
    follows,
    indexer_progress,
    intellectual_property,
    ip_licenses,
    platform_daily_statistics,
    platforms,
    profile_platform_links,
    profiles,
    proof_of_creativity,
);