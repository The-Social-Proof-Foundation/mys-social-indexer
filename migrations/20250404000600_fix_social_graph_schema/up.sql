-- Update social_graph_relationships table to use blockchain addresses as primary key fields
-- This ensures we don't rely on database IDs, which can be duplicated

ALTER TABLE social_graph_relationships 
DROP COLUMN follower_id,
DROP COLUMN following_id;

-- Update the unique constraint to prevent duplicate relationships using addresses
ALTER TABLE social_graph_relationships
ADD CONSTRAINT social_graph_relationships_unique_relationship 
UNIQUE (follower_address, following_address);

-- Add indexes on addresses for fast lookups
CREATE INDEX idx_social_graph_relationships_follower_address ON social_graph_relationships(follower_address);
CREATE INDEX idx_social_graph_relationships_following_address ON social_graph_relationships(following_address);

-- Similarly update the social_graph_events table to remove db IDs
ALTER TABLE social_graph_events
DROP COLUMN follower_id,
DROP COLUMN following_id;

-- Add raw blockchain IDs to profiles table for better tracking
ALTER TABLE profiles
ADD COLUMN blockchain_tx_id VARCHAR NULL;

-- Delete placeholder profiles (rows 3 and 4) that were automatically created
DELETE FROM profiles WHERE id IN (3, 4);

-- Update counter values for existing profiles
UPDATE profiles 
SET followers_count = (
    SELECT COUNT(*) FROM social_graph_relationships 
    WHERE following_address = profiles.owner_address
),
following_count = (
    SELECT COUNT(*) FROM social_graph_relationships 
    WHERE follower_address = profiles.owner_address
);