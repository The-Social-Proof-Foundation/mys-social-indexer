-- Revert the changes by adding back the role and left_at columns

-- Add back the removed columns
ALTER TABLE platform_memberships 
    ADD COLUMN role VARCHAR NOT NULL DEFAULT 'member',
    ADD COLUMN left_at TIMESTAMP NULL;

-- Remove the comment
COMMENT ON TABLE platform_memberships IS NULL; 