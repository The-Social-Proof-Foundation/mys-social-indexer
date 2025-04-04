-- Remove blockchain_username_id column from usernames table
ALTER TABLE usernames DROP COLUMN IF EXISTS blockchain_username_id;