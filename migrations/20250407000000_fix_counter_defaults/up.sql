-- Ensure the followers and following counts have proper defaults and constraints
-- First set default values for any NULLs that might exist
UPDATE profiles
SET followers_count = 0
WHERE followers_count IS NULL;

UPDATE profiles
SET following_count = 0
WHERE following_count IS NULL;

-- Change the column constraint to ensure it's always NOT NULL with default 0
ALTER TABLE profiles 
ALTER COLUMN followers_count SET DEFAULT 0,
ALTER COLUMN followers_count SET NOT NULL;

ALTER TABLE profiles 
ALTER COLUMN following_count SET DEFAULT 0,
ALTER COLUMN following_count SET NOT NULL;

-- Validate the follower/following counts against actual relationships
UPDATE profiles p
SET followers_count = COALESCE(
    (SELECT COUNT(*) FROM social_graph_relationships 
     WHERE following_address = p.owner_address), 0);

UPDATE profiles p
SET following_count = COALESCE(
    (SELECT COUNT(*) FROM social_graph_relationships 
     WHERE follower_address = p.owner_address), 0);

-- Create a function to verify the counts are updated correctly
CREATE OR REPLACE FUNCTION verify_follow_counts() 
RETURNS TRIGGER AS $$
BEGIN
    -- Log whenever an update happens
    RAISE NOTICE 'Follow relationship changed: %', TG_OP;
    
    -- Different actions based on operation type
    IF (TG_OP = 'INSERT') THEN
        UPDATE profiles
        SET following_count = following_count + 1
        WHERE owner_address = NEW.follower_address;
        
        UPDATE profiles
        SET followers_count = followers_count + 1
        WHERE owner_address = NEW.following_address;
        
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE profiles
        SET following_count = GREATEST(0, following_count - 1)
        WHERE owner_address = OLD.follower_address;
        
        UPDATE profiles
        SET followers_count = GREATEST(0, followers_count - 1)
        WHERE owner_address = OLD.following_address;
        
        RETURN OLD;
    END IF;
    
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create triggers to automatically update the counts
DROP TRIGGER IF EXISTS update_follow_counts_insert ON social_graph_relationships;
CREATE TRIGGER update_follow_counts_insert
AFTER INSERT ON social_graph_relationships
FOR EACH ROW EXECUTE FUNCTION verify_follow_counts();

DROP TRIGGER IF EXISTS update_follow_counts_delete ON social_graph_relationships;
CREATE TRIGGER update_follow_counts_delete
AFTER DELETE ON social_graph_relationships  
FOR EACH ROW EXECUTE FUNCTION verify_follow_counts();