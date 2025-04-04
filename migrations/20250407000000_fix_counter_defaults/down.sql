-- Remove the triggers
DROP TRIGGER IF EXISTS update_follow_counts_insert ON social_graph_relationships;
DROP TRIGGER IF EXISTS update_follow_counts_delete ON social_graph_relationships;

-- Remove the trigger function
DROP FUNCTION IF EXISTS verify_follow_counts();