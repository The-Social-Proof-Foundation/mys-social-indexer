-- Since usernames are now handled directly within profiles,
-- we no longer need separate tables for usernames and username_history

-- Make sure to drop foreign key constraints first
DO $$ 
BEGIN
    -- Check if the constraint exists before dropping
    IF EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'usernames_profile_id_fkey'
    ) THEN
        ALTER TABLE usernames DROP CONSTRAINT usernames_profile_id_fkey;
    END IF;
    
    IF EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'username_history_profile_id_fkey'
    ) THEN
        ALTER TABLE username_history DROP CONSTRAINT username_history_profile_id_fkey;
    END IF;
END $$;

-- Drop the tables
DROP TABLE IF EXISTS username_history;
DROP TABLE IF EXISTS usernames;

-- Add index on username column in profiles
-- This ensures we can still efficiently lookup profiles by username
CREATE INDEX IF NOT EXISTS idx_profiles_username ON profiles(username);