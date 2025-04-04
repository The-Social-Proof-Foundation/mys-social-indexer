-- Remove columns related to platform approval
ALTER TABLE platforms DROP COLUMN is_approved;
ALTER TABLE platforms DROP COLUMN approval_changed_at;
ALTER TABLE platforms DROP COLUMN approved_by;