-- Revert the change: remove event_id and add back blockchain_tx_hash
ALTER TABLE social_graph_events 
DROP COLUMN IF EXISTS event_id,
ADD COLUMN blockchain_tx_hash VARCHAR;

-- Recreate the index if it existed before
CREATE INDEX idx_social_graph_events_blockchain_tx_hash ON social_graph_events(blockchain_tx_hash);