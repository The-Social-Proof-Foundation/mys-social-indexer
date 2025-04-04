-- This is a destructive change - we can't perfectly restore the previous state
-- But we can add back the columns with null values

-- Add back follower_id and following_id columns to social_graph_relationships
ALTER TABLE social_graph_relationships 
ADD COLUMN follower_id INTEGER NULL,
ADD COLUMN following_id INTEGER NULL;

-- Add back follower_id and following_id columns to social_graph_events
ALTER TABLE social_graph_events
ADD COLUMN follower_id INTEGER NULL,
ADD COLUMN following_id INTEGER NULL;

-- Drop the blockchain_tx_id column from profiles
ALTER TABLE profiles
DROP COLUMN IF EXISTS blockchain_tx_id;