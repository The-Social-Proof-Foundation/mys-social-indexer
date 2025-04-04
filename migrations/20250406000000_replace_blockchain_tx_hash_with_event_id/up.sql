-- Remove blockchain_tx_hash and add event_id to the social_graph_events table
ALTER TABLE social_graph_events 
DROP COLUMN IF EXISTS blockchain_tx_hash,
ADD COLUMN event_id VARCHAR;

-- Create an index on event_id for faster lookups
CREATE INDEX idx_social_graph_events_event_id ON social_graph_events(event_id);

-- Add a comment to describe the event_id field
COMMENT ON COLUMN social_graph_events.event_id IS 'Blockchain event ID in the format <digest>:<event_seq>';

-- Update any existing rows to have a placeholder event_id if needed
-- Update with placeholder since we can't go back in time to get the real event IDs
UPDATE social_graph_events
SET event_id = 'historical_event_' || id::text
WHERE event_id IS NULL;