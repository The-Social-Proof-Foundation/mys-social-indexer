-- Drop social graph table and indexes
DROP TABLE IF EXISTS social_graph_relationships;

-- Remove follower/following count columns from profiles table
ALTER TABLE profiles 
DROP COLUMN IF EXISTS followers_count,
DROP COLUMN IF EXISTS following_count;