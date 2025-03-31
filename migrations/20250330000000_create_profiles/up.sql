-- Create profiles table
CREATE TABLE profiles (
    id SERIAL PRIMARY KEY,
    owner_address VARCHAR(255) NOT NULL,
    username VARCHAR(100) NOT NULL,
    display_name VARCHAR(255),
    bio TEXT,
    avatar_url VARCHAR(255),
    website_url VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create unique indexes
CREATE UNIQUE INDEX idx_profiles_owner_address ON profiles(owner_address);
CREATE UNIQUE INDEX idx_profiles_username ON profiles(username);

-- Create indexer checkpoint state table
CREATE TABLE indexer_checkpoint_state (
    id SERIAL PRIMARY KEY,
    last_processed_checkpoint BIGINT NOT NULL,
    last_processed_timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Insert initial indexer checkpoint state
INSERT INTO indexer_checkpoint_state (id, last_processed_checkpoint, last_processed_timestamp)
VALUES (1, 0, NOW());