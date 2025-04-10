-- Update the platform_memberships table by removing the role and left_at columns
-- This change is to support the new approach where we completely delete records when users leave platforms
-- rather than just marking them with a left_at timestamp

-- Remove the columns
ALTER TABLE platform_memberships 
    DROP COLUMN role,
    DROP COLUMN left_at;

-- Add a comment to the table explaining the new behavior
COMMENT ON TABLE platform_memberships IS 'Records of profiles joined to platforms. Records are deleted when a user leaves a platform.'; 