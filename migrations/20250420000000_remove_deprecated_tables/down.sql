-- Drop profile_events table
DROP TABLE IF EXISTS profile_events;

-- Re-create platform_relationships table
CREATE TABLE platform_relationships (
    id SERIAL PRIMARY KEY,
    platform_id VARCHAR NOT NULL,
    profile_id VARCHAR NOT NULL,
    joined_at TIMESTAMP NOT NULL,
    left_at TIMESTAMP
);

-- Re-create indexes for platform_relationships
CREATE INDEX idx_platform_relationships_platform_id ON platform_relationships (platform_id);
CREATE INDEX idx_platform_relationships_profile_id ON platform_relationships (profile_id);
CREATE INDEX idx_platform_relationships_joined_at ON platform_relationships (joined_at);
CREATE INDEX idx_platform_relationships_left_at ON platform_relationships (left_at);

-- Re-create platforms_blocked table
CREATE TABLE platforms_blocked (
    id SERIAL PRIMARY KEY,
    profile_id VARCHAR NOT NULL,
    platform_id VARCHAR NOT NULL,
    blocked_by VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    is_blocked BOOLEAN NOT NULL,
    unblocked_at TIMESTAMP,
    unblocked_by VARCHAR
);

-- Re-create indexes for platforms_blocked
CREATE INDEX idx_platforms_blocked_profile_id ON platforms_blocked (profile_id);
CREATE INDEX idx_platforms_blocked_platform_id ON platforms_blocked (platform_id);
CREATE INDEX idx_platforms_blocked_blocked_by ON platforms_blocked (blocked_by);
CREATE INDEX idx_platforms_blocked_is_blocked ON platforms_blocked (is_blocked);