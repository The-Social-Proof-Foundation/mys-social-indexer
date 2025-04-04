-- Create usernames table
CREATE TABLE IF NOT EXISTS usernames (
    id SERIAL PRIMARY KEY,
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    username VARCHAR NOT NULL UNIQUE,
    registered_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create index on username for fast lookups
CREATE INDEX IF NOT EXISTS usernames_username_idx ON usernames(username);

-- Create index on profile_id for fast profile lookups
CREATE INDEX IF NOT EXISTS usernames_profile_id_idx ON usernames(profile_id);

-- Add history table to track username changes
CREATE TABLE IF NOT EXISTS username_history (
    id SERIAL PRIMARY KEY,
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    old_username VARCHAR NOT NULL,
    new_username VARCHAR NOT NULL,
    changed_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create index on profile_id for username history
CREATE INDEX IF NOT EXISTS username_history_profile_id_idx ON username_history(profile_id);