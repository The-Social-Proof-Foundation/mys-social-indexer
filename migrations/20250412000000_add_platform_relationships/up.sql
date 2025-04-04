-- Create platform_relationships table
CREATE TABLE platform_relationships (
    id SERIAL PRIMARY KEY,
    platform_id VARCHAR NOT NULL,
    profile_id VARCHAR NOT NULL,
    joined_at TIMESTAMP NOT NULL,
    left_at TIMESTAMP,
    -- Add unique constraint to prevent duplicate active relationships
    UNIQUE (platform_id, profile_id)
);

-- Create index for faster lookups
CREATE INDEX idx_platform_relationships_platform_id ON platform_relationships (platform_id);
CREATE INDEX idx_platform_relationships_profile_id ON platform_relationships (profile_id);
CREATE INDEX idx_platform_relationships_joined_at ON platform_relationships (joined_at);
CREATE INDEX idx_platform_relationships_left_at ON platform_relationships (left_at);