-- Remove blockchain_tx_id from profiles table if it exists
ALTER TABLE profiles 
DROP COLUMN IF EXISTS blockchain_tx_id;

-- Ensure social_graph_relationships have proper constraints and integrity
-- Make sure we can't delete events (enforce only inserting, never deleting)
CREATE OR REPLACE FUNCTION prevent_social_graph_events_deletion()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Deleting social_graph_events records is not allowed';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER no_delete_social_graph_events
BEFORE DELETE ON social_graph_events
FOR EACH ROW EXECUTE FUNCTION prevent_social_graph_events_deletion();

-- Reset existing follower and following counts to ensure they're accurate
UPDATE profiles 
SET followers_count = (
    SELECT COUNT(*) FROM social_graph_relationships 
    WHERE following_address = profiles.owner_address
),
following_count = (
    SELECT COUNT(*) FROM social_graph_relationships 
    WHERE follower_address = profiles.owner_address
);

-- Note: We're keeping blockchain_tx_hash in social_graph_events table
-- as it's useful for blockchain transaction reference, but we're ensuring
-- it's properly handled.