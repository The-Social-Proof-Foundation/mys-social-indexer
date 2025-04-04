-- This migration ensures the followers_count and following_count columns
-- in the profiles table are correctly updated from relationships

-- First ensure columns have proper defaults
ALTER TABLE profiles 
ALTER COLUMN followers_count SET DEFAULT 0,
ALTER COLUMN followers_count SET NOT NULL;

ALTER TABLE profiles 
ALTER COLUMN following_count SET DEFAULT 0,
ALTER COLUMN following_count SET NOT NULL;

-- Ensure profile_id is indexed for faster lookups
CREATE INDEX IF NOT EXISTS idx_profiles_profile_id ON profiles(profile_id);

-- Update the counts based on actual relationships - matching on profile_id, not owner_address
UPDATE profiles p
SET followers_count = (
    SELECT COUNT(*) FROM social_graph_relationships 
    WHERE following_address = p.profile_id
)
WHERE p.profile_id IS NOT NULL;

UPDATE profiles p
SET following_count = (
    SELECT COUNT(*) FROM social_graph_relationships 
    WHERE follower_address = p.profile_id
)
WHERE p.profile_id IS NOT NULL;

-- Create an index on relationships to improve performance of count lookups
DROP INDEX IF EXISTS idx_social_graph_relationships_pair;
CREATE INDEX idx_social_graph_relationships_pair 
ON social_graph_relationships(follower_address, following_address);

-- Add details to the unique constraint for clarity
COMMENT ON CONSTRAINT social_graph_relationships_unique_relationship 
ON social_graph_relationships 
IS 'Ensures follower can only follow an account once';