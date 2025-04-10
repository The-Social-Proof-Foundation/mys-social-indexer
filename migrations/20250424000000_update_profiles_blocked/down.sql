-- Revert the changes by adding back the is_blocked and unblocked_at columns

-- Add back the removed columns
ALTER TABLE profiles_blocked 
    ADD COLUMN is_blocked BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN unblocked_at TIMESTAMP NULL;

-- Remove the comment
COMMENT ON TABLE profiles_blocked IS NULL; 