-- Add follower/following count columns to profiles table
ALTER TABLE profiles 
ADD COLUMN followers_count INTEGER NOT NULL DEFAULT 0,
ADD COLUMN following_count INTEGER NOT NULL DEFAULT 0;

-- Create social graph relationships table
CREATE TABLE social_graph_relationships (
    id SERIAL PRIMARY KEY,
    follower_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    following_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    follower_address VARCHAR NOT NULL,
    following_address VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    
    -- Add unique constraint to prevent duplicate relationships
    UNIQUE(follower_id, following_id)
);

-- Add indexes for faster lookups
CREATE INDEX idx_social_graph_follower_id ON social_graph_relationships(follower_id);
CREATE INDEX idx_social_graph_following_id ON social_graph_relationships(following_id);
CREATE INDEX idx_social_graph_follower_address ON social_graph_relationships(follower_address);
CREATE INDEX idx_social_graph_following_address ON social_graph_relationships(following_address);
CREATE INDEX idx_social_graph_created_at ON social_graph_relationships(created_at);

-- Add indexes to profile table for follower/following counts
CREATE INDEX idx_profiles_followers_count ON profiles(followers_count);
CREATE INDEX idx_profiles_following_count ON profiles(following_count);

-- Add comment to describe the purpose of the table
COMMENT ON TABLE social_graph_relationships IS 'Tracks follow relationships between profiles';