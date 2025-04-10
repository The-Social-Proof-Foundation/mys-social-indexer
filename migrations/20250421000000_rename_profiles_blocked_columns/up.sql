-- Rename columns in the profiles_blocked table
ALTER TABLE profiles_blocked 
  RENAME COLUMN blocker_profile_id TO blocker_wallet_address;

ALTER TABLE profiles_blocked 
  RENAME COLUMN blocked_profile_id TO blocked_address;

-- Update indices to reflect the new column names
DROP INDEX IF EXISTS idx_profiles_blocked_blocker_profile_id;
DROP INDEX IF EXISTS idx_profiles_blocked_blocked_profile_id;

CREATE INDEX IF NOT EXISTS idx_profiles_blocked_blocker_wallet_address ON profiles_blocked(blocker_wallet_address);
CREATE INDEX IF NOT EXISTS idx_profiles_blocked_blocked_address ON profiles_blocked(blocked_address); 