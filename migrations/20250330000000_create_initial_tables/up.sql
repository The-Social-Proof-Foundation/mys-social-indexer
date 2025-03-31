-- Create profiles table
CREATE TABLE IF NOT EXISTS profiles (
    id SERIAL PRIMARY KEY,
    owner_address VARCHAR NOT NULL,
    username VARCHAR NOT NULL UNIQUE,
    display_name VARCHAR,
    bio TEXT,
    avatar_url VARCHAR,
    website_url VARCHAR,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create an index on owner_address for faster lookups
CREATE INDEX IF NOT EXISTS idx_profiles_owner_address ON profiles(owner_address);

-- Create indexer progress table to track processing
CREATE TABLE IF NOT EXISTS indexer_progress (
    id VARCHAR PRIMARY KEY,
    last_checkpoint_processed BIGINT NOT NULL DEFAULT 0,
    last_processed_at TIMESTAMP NOT NULL DEFAULT NOW()
);