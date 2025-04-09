-- Create the platforms_blocked table
CREATE TABLE IF NOT EXISTS platforms_blocked (
    id SERIAL PRIMARY KEY,
    profile_id VARCHAR NOT NULL,
    platform_id VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    is_blocked BOOLEAN NOT NULL DEFAULT TRUE,
    unblocked_at TIMESTAMP,
    UNIQUE(profile_id, platform_id)
);

-- Create indices for faster lookups
CREATE INDEX IF NOT EXISTS idx_platforms_blocked_profile_id ON platforms_blocked(profile_id);
CREATE INDEX IF NOT EXISTS idx_platforms_blocked_platform_id ON platforms_blocked(platform_id);
CREATE INDEX IF NOT EXISTS idx_platforms_blocked_is_blocked ON platforms_blocked(is_blocked); 