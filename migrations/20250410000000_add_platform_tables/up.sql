-- Create platforms table
CREATE TABLE platforms (
    id SERIAL PRIMARY KEY,
    platform_id VARCHAR NOT NULL UNIQUE,
    name VARCHAR NOT NULL,
    tagline VARCHAR NOT NULL,
    description TEXT,
    logo VARCHAR,
    developer_address VARCHAR NOT NULL,
    terms_of_service TEXT,
    privacy_policy TEXT,
    platforms JSONB,                  -- Array of platform names (Twitter, Instagram, etc.)
    links JSONB,                      -- Array of platform URLs
    status SMALLINT NOT NULL,         -- Platform status (0=dev, 1=alpha, 2=beta, 3=live, etc.)
    release_date VARCHAR,
    shutdown_date VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create index on platform_id
CREATE INDEX idx_platforms_platform_id ON platforms(platform_id);

-- Create index on name for quick lookups
CREATE INDEX idx_platforms_name ON platforms(name);

-- Create moderators table
CREATE TABLE platform_moderators (
    id SERIAL PRIMARY KEY,
    platform_id VARCHAR NOT NULL,
    moderator_address VARCHAR NOT NULL,
    added_by VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(platform_id, moderator_address)
);

-- Create index on platform_id for moderators
CREATE INDEX idx_platform_moderators_platform_id ON platform_moderators(platform_id);

-- Create blocked profiles table
CREATE TABLE platform_blocked_profiles (
    id SERIAL PRIMARY KEY,
    platform_id VARCHAR NOT NULL,
    profile_id VARCHAR NOT NULL,
    blocked_by VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    is_blocked BOOLEAN NOT NULL DEFAULT TRUE,
    unblocked_at TIMESTAMP,
    unblocked_by VARCHAR,
    UNIQUE(platform_id, profile_id)
);

-- Create index on platform_id for blocked profiles
CREATE INDEX idx_platform_blocked_profiles_platform_id ON platform_blocked_profiles(platform_id);

-- Create index on profile_id for blocked profiles to quickly check if a profile is blocked
CREATE INDEX idx_platform_blocked_profiles_profile_id ON platform_blocked_profiles(profile_id);

-- Create a table for platform events
CREATE TABLE platform_events (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR NOT NULL,
    platform_id VARCHAR NOT NULL,
    event_data JSONB NOT NULL,
    event_id VARCHAR,  
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create index on platform_id for events
CREATE INDEX idx_platform_events_platform_id ON platform_events(platform_id);