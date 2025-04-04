-- Add back blockchain_tx_id to profiles table
ALTER TABLE profiles
ADD COLUMN blockchain_tx_id VARCHAR NULL;

-- Remove trigger that prevents deleting social_graph_events
DROP TRIGGER IF EXISTS no_delete_social_graph_events ON social_graph_events;
DROP FUNCTION IF EXISTS prevent_social_graph_events_deletion();