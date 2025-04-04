-- Recreate the original tables
CREATE TABLE IF NOT EXISTS profile_private_fields (
    profile_id INTEGER PRIMARY KEY REFERENCES profiles(id) ON DELETE CASCADE,
    has_birthdate BOOLEAN NOT NULL DEFAULT FALSE,
    has_current_location BOOLEAN NOT NULL DEFAULT FALSE,
    has_raised_location BOOLEAN NOT NULL DEFAULT FALSE,
    has_phone BOOLEAN NOT NULL DEFAULT FALSE,
    has_email BOOLEAN NOT NULL DEFAULT FALSE,
    has_gender BOOLEAN NOT NULL DEFAULT FALSE,
    has_political_view BOOLEAN NOT NULL DEFAULT FALSE,
    has_education BOOLEAN NOT NULL DEFAULT FALSE,
    has_primary_language BOOLEAN NOT NULL DEFAULT FALSE,
    has_relationship_status BOOLEAN NOT NULL DEFAULT FALSE,
    has_social_usernames BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS profile_encrypted_data (
    profile_id INTEGER PRIMARY KEY REFERENCES profiles(id) ON DELETE CASCADE,
    encrypted_data BYTEA NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Add back the has_private_data column
ALTER TABLE profiles 
  ADD COLUMN IF NOT EXISTS has_private_data BOOLEAN NOT NULL DEFAULT FALSE;

-- Rename sensitive_data_updated_at back to private_data_updated_at
ALTER TABLE profiles 
  RENAME COLUMN sensitive_data_updated_at TO private_data_updated_at;

-- Initialize has_private_data based on presence of any field
UPDATE profiles 
SET has_private_data = 
  CASE WHEN 
    birthdate IS NOT NULL OR 
    current_location IS NOT NULL OR 
    raised_location IS NOT NULL OR 
    phone IS NOT NULL OR 
    email IS NOT NULL OR 
    gender IS NOT NULL OR 
    political_view IS NOT NULL OR 
    religion IS NOT NULL OR 
    education IS NOT NULL OR 
    primary_language IS NOT NULL OR 
    relationship_status IS NOT NULL OR 
    x_username IS NOT NULL OR 
    mastodon_username IS NOT NULL OR 
    facebook_username IS NOT NULL OR 
    reddit_username IS NOT NULL OR 
    github_username IS NOT NULL
  THEN TRUE ELSE FALSE END;

-- Drop the new columns - they're now stored in the separate tables
ALTER TABLE profiles 
  DROP COLUMN IF EXISTS birthdate,
  DROP COLUMN IF EXISTS current_location,
  DROP COLUMN IF EXISTS raised_location, 
  DROP COLUMN IF EXISTS phone,
  DROP COLUMN IF EXISTS email,
  DROP COLUMN IF EXISTS gender,
  DROP COLUMN IF EXISTS political_view,
  DROP COLUMN IF EXISTS religion,
  DROP COLUMN IF EXISTS education,
  DROP COLUMN IF EXISTS primary_language, 
  DROP COLUMN IF EXISTS relationship_status,
  DROP COLUMN IF EXISTS x_username,
  DROP COLUMN IF EXISTS mastodon_username,
  DROP COLUMN IF EXISTS facebook_username,
  DROP COLUMN IF EXISTS reddit_username,
  DROP COLUMN IF EXISTS github_username;