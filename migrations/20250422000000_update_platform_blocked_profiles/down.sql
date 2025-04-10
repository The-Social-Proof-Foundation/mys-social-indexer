-- Revert the changes by adding back the is_blocked, unblocked_at, and unblocked_by columns

-- Add back the removed columns
ALTER TABLE platform_blocked_profiles 
    ADD COLUMN is_blocked BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN unblocked_at TIMESTAMP NULL,
    ADD COLUMN unblocked_by VARCHAR NULL;

-- Remove the comment
COMMENT ON TABLE platform_blocked_profiles IS NULL; 