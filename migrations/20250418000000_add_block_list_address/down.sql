-- This script reverses "up.sql"
DROP INDEX IF EXISTS idx_profiles_block_list_address;
ALTER TABLE profiles DROP COLUMN IF EXISTS block_list_address;