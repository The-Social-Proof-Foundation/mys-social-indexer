-- Drop platforms_blocked table
DROP TABLE IF EXISTS platforms_blocked;

-- Drop platform_relationships table
DROP TABLE IF EXISTS platform_relationships;

-- Create profile_events table
CREATE TABLE profile_events (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR NOT NULL,
    profile_id VARCHAR NOT NULL,
    event_data JSONB NOT NULL,
    event_id VARCHAR,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Add indexes for profile_events table
CREATE INDEX idx_profile_events_profile_id ON profile_events (profile_id);
CREATE INDEX idx_profile_events_event_type ON profile_events (event_type);
CREATE INDEX idx_profile_events_event_id ON profile_events (event_id);
CREATE INDEX idx_profile_events_created_at ON profile_events (created_at);