-- Your SQL goes here
ALTER TABLE profiles
ADD COLUMN block_list_address VARCHAR;

-- Add index for faster lookups
CREATE INDEX idx_profiles_block_list_address ON profiles (block_list_address);

-- Comment for the new column
COMMENT ON COLUMN profiles.block_list_address IS 'The Blockchain address of the profile''s BlockList object';