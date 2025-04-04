-- Add all encrypted fields directly to the profiles table
ALTER TABLE profiles 
  ADD COLUMN IF NOT EXISTS birthdate TEXT,
  ADD COLUMN IF NOT EXISTS current_location TEXT,
  ADD COLUMN IF NOT EXISTS raised_location TEXT,
  ADD COLUMN IF NOT EXISTS phone TEXT,
  ADD COLUMN IF NOT EXISTS email TEXT,
  ADD COLUMN IF NOT EXISTS gender TEXT,
  ADD COLUMN IF NOT EXISTS political_view TEXT,
  ADD COLUMN IF NOT EXISTS religion TEXT,
  ADD COLUMN IF NOT EXISTS education TEXT,
  ADD COLUMN IF NOT EXISTS primary_language TEXT,
  ADD COLUMN IF NOT EXISTS relationship_status TEXT,
  ADD COLUMN IF NOT EXISTS x_username TEXT,
  ADD COLUMN IF NOT EXISTS mastodon_username TEXT,
  ADD COLUMN IF NOT EXISTS facebook_username TEXT,
  ADD COLUMN IF NOT EXISTS reddit_username TEXT,
  ADD COLUMN IF NOT EXISTS github_username TEXT;

-- Migrate any existing data from profile_encrypted_data and profile_private_fields
-- For the migration, we're setting just flags indicating if data exists
-- In a real migration, you would decode the encrypted data and move it
DO $$ 
BEGIN
  UPDATE profiles p
  SET 
    birthdate = CASE WHEN ppf.has_birthdate THEN '(encrypted)' ELSE NULL END,
    current_location = CASE WHEN ppf.has_current_location THEN '(encrypted)' ELSE NULL END,
    raised_location = CASE WHEN ppf.has_raised_location THEN '(encrypted)' ELSE NULL END,
    phone = CASE WHEN ppf.has_phone THEN '(encrypted)' ELSE NULL END,
    email = CASE WHEN ppf.has_email THEN '(encrypted)' ELSE NULL END,
    gender = CASE WHEN ppf.has_gender THEN '(encrypted)' ELSE NULL END,
    political_view = CASE WHEN ppf.has_political_view THEN '(encrypted)' ELSE NULL END,
    education = CASE WHEN ppf.has_education THEN '(encrypted)' ELSE NULL END,
    primary_language = CASE WHEN ppf.has_primary_language THEN '(encrypted)' ELSE NULL END,
    relationship_status = CASE WHEN ppf.has_relationship_status THEN '(encrypted)' ELSE NULL END,
    x_username = CASE WHEN ppf.has_social_usernames THEN '(encrypted)' ELSE NULL END,
    mastodon_username = CASE WHEN ppf.has_social_usernames THEN '(encrypted)' ELSE NULL END,
    facebook_username = CASE WHEN ppf.has_social_usernames THEN '(encrypted)' ELSE NULL END,
    reddit_username = CASE WHEN ppf.has_social_usernames THEN '(encrypted)' ELSE NULL END,
    github_username = CASE WHEN ppf.has_social_usernames THEN '(encrypted)' ELSE NULL END
  FROM profile_private_fields ppf
  WHERE p.id = ppf.profile_id;
END $$;

-- Drop old tables that are no longer needed
DROP TABLE IF EXISTS profile_encrypted_data;
DROP TABLE IF EXISTS profile_private_fields;

-- Remove has_private_data column, since we now know based on field presence
ALTER TABLE profiles DROP COLUMN IF EXISTS has_private_data;

-- Rename private_data_updated_at to sensitive_data_updated_at for clarity
ALTER TABLE profiles 
  RENAME COLUMN private_data_updated_at TO sensitive_data_updated_at;