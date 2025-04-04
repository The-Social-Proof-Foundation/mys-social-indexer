-- Add blockchain_username_id column back to usernames table
ALTER TABLE usernames ADD COLUMN IF NOT EXISTS blockchain_username_id VARCHAR;