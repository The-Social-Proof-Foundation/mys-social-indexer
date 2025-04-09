-- Create the profiles_blocked table
CREATE TABLE IF NOT EXISTS profiles_blocked (
    id SERIAL PRIMARY KEY,
    blocker_profile_id VARCHAR NOT NULL,
    blocked_profile_id VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    is_blocked BOOLEAN NOT NULL DEFAULT TRUE,
    unblocked_at TIMESTAMP,
    UNIQUE(blocker_profile_id, blocked_profile_id)
);

-- Create indices for faster lookups
CREATE INDEX IF NOT EXISTS idx_profiles_blocked_blocker_profile_id ON profiles_blocked(blocker_profile_id);
CREATE INDEX IF NOT EXISTS idx_profiles_blocked_blocked_profile_id ON profiles_blocked(blocked_profile_id);
CREATE INDEX IF NOT EXISTS idx_profiles_blocked_is_blocked ON profiles_blocked(is_blocked); 