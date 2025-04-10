-- Update the profiles_blocked table by removing the is_blocked and unblocked_at columns
-- This change is to support the new approach where we completely delete records when profiles are unblocked
-- rather than just marking them as unblocked

-- Remove the columns
ALTER TABLE profiles_blocked 
    DROP COLUMN is_blocked,
    DROP COLUMN unblocked_at;

-- Add a comment to the table explaining the new behavior
COMMENT ON TABLE profiles_blocked IS 'Records of profiles blocked by other profiles. Records are deleted when a profile is unblocked.'; 