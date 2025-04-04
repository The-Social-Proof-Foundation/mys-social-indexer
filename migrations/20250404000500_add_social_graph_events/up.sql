-- Create table to log social graph events
CREATE TABLE social_graph_events (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR NOT NULL, -- 'follow' or 'unfollow'
    follower_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    following_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    follower_address VARCHAR NOT NULL,
    following_address VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    blockchain_tx_hash VARCHAR, -- Optional blockchain transaction hash
    raw_event_data JSONB -- Raw event data from blockchain
);

-- Add indexes for efficient querying
CREATE INDEX idx_social_graph_events_event_type ON social_graph_events(event_type);
CREATE INDEX idx_social_graph_events_follower_id ON social_graph_events(follower_id);
CREATE INDEX idx_social_graph_events_following_id ON social_graph_events(following_id);
CREATE INDEX idx_social_graph_events_created_at ON social_graph_events(created_at);

-- Add comment to describe the purpose of the table
COMMENT ON TABLE social_graph_events IS 'Records all follow/unfollow events for audit and history';