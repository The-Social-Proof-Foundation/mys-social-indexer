-- Revert column name changes in the profiles_blocked table
ALTER TABLE profiles_blocked 
  RENAME COLUMN blocker_wallet_address TO blocker_profile_id;

ALTER TABLE profiles_blocked 
  RENAME COLUMN blocked_address TO blocked_profile_id;

-- Revert indices to match original column names
DROP INDEX IF EXISTS idx_profiles_blocked_blocker_wallet_address;
DROP INDEX IF EXISTS idx_profiles_blocked_blocked_address;

CREATE INDEX IF NOT EXISTS idx_profiles_blocked_blocker_profile_id ON profiles_blocked(blocker_profile_id);
CREATE INDEX IF NOT EXISTS idx_profiles_blocked_blocked_profile_id ON profiles_blocked(blocked_profile_id); 