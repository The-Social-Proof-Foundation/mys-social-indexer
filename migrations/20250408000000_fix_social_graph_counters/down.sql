-- No specific rollback needed, as data recalculation is idempotent
-- We only drop any indexes we added
DROP INDEX IF EXISTS idx_social_graph_relationships_pair;