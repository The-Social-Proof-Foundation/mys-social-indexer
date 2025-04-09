-- Remove encrypted data table
DROP TABLE IF EXISTS profile_encrypted_data;

-- Remove private fields table
DROP TABLE IF EXISTS profile_private_fields;

-- Remove columns from profiles table
ALTER TABLE profiles DROP COLUMN IF EXISTS cover_photo;
ALTER TABLE profiles DROP COLUMN IF EXISTS profile_id;
ALTER TABLE profiles DROP COLUMN IF EXISTS has_private_data;
ALTER TABLE profiles DROP COLUMN IF EXISTS private_data_updated_at;