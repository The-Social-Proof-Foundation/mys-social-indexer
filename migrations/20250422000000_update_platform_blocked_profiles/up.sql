-- Update the platform_blocked_profiles table by removing the is_blocked, unblocked_at, and unblocked_by columns
-- This change is to support the new approach where we completely delete records when profiles are unblocked
-- rather than just marking them as unblocked

-- Remove the columns
ALTER TABLE platform_blocked_profiles 
    DROP COLUMN is_blocked,
    DROP COLUMN unblocked_at,
    DROP COLUMN unblocked_by;

-- Add a comment to the table explaining the new behavior
COMMENT ON TABLE platform_blocked_profiles IS 'Records of profiles blocked by platforms. Records are deleted when a profile is unblocked.'; 