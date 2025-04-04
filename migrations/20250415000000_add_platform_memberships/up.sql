-- Create platform_memberships table
CREATE TABLE platform_memberships (
    id SERIAL PRIMARY KEY,
    platform_id VARCHAR NOT NULL,
    profile_id VARCHAR NOT NULL,
    role VARCHAR NOT NULL,
    joined_at TIMESTAMP NOT NULL,
    left_at TIMESTAMP,
    -- Add unique constraint to prevent duplicate active memberships
    UNIQUE (platform_id, profile_id)
);

-- Create index for faster lookups
CREATE INDEX idx_platform_memberships_platform_id ON platform_memberships (platform_id);
CREATE INDEX idx_platform_memberships_profile_id ON platform_memberships (profile_id);
CREATE INDEX idx_platform_memberships_role ON platform_memberships (role);
CREATE INDEX idx_platform_memberships_joined_at ON platform_memberships (joined_at);
CREATE INDEX idx_platform_memberships_left_at ON platform_memberships (left_at); 