-- Add is_approved column to platforms table
ALTER TABLE platforms
ADD COLUMN is_approved BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE platforms
ADD COLUMN approval_changed_at TIMESTAMP NULL;

ALTER TABLE platforms
ADD COLUMN approved_by VARCHAR NULL;