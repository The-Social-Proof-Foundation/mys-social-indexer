-- Drop indexes first
DROP INDEX IF EXISTS idx_platform_memberships_platform_id;
DROP INDEX IF EXISTS idx_platform_memberships_profile_id;
DROP INDEX IF EXISTS idx_platform_memberships_role;
DROP INDEX IF EXISTS idx_platform_memberships_joined_at;
DROP INDEX IF EXISTS idx_platform_memberships_left_at;

-- Drop the table
DROP TABLE IF EXISTS platform_memberships; 