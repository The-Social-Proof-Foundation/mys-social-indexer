-- Recreate the username and username_history tables

-- Recreate usernames table
CREATE TABLE IF NOT EXISTS usernames (
    id SERIAL PRIMARY KEY,
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    username VARCHAR(100) NOT NULL,
    registered_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Add unique index to enforce uniqueness of usernames
CREATE UNIQUE INDEX IF NOT EXISTS idx_usernames_username ON usernames(username);

-- Recreate username_history table
CREATE TABLE IF NOT EXISTS username_history (
    id SERIAL PRIMARY KEY,
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    old_username VARCHAR(100) NOT NULL,
    new_username VARCHAR(100) NOT NULL,
    changed_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Populate the usernames table with existing usernames from profiles
INSERT INTO usernames (profile_id, username, registered_at, updated_at)
SELECT id, username, created_at, updated_at
FROM profiles;