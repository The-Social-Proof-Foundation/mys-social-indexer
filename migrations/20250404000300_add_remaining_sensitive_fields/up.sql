-- Make sure profiles table has all the fields from profile.move contract
DO $$ 
BEGIN
    -- Add religion field if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='religion') THEN
        ALTER TABLE profiles ADD COLUMN religion TEXT;
    END IF;
    
    -- Add all other fields if they don't exist
    -- Basic profile fields should already exist
    
    -- Sensitive fields - ensure all are present
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='birthdate') THEN
        ALTER TABLE profiles ADD COLUMN birthdate TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='current_location') THEN
        ALTER TABLE profiles ADD COLUMN current_location TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='raised_location') THEN
        ALTER TABLE profiles ADD COLUMN raised_location TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='phone') THEN
        ALTER TABLE profiles ADD COLUMN phone TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='email') THEN
        ALTER TABLE profiles ADD COLUMN email TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='gender') THEN
        ALTER TABLE profiles ADD COLUMN gender TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='political_view') THEN
        ALTER TABLE profiles ADD COLUMN political_view TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='education') THEN
        ALTER TABLE profiles ADD COLUMN education TEXT;
    END IF;
    
    -- Handle website field - make sure we have just one website field
    -- If website_url exists but website doesn't, rename website_url to website
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name='profiles' AND column_name='website_url') 
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name='profiles' AND column_name='website') THEN
        -- Rename website_url to website
        ALTER TABLE profiles RENAME COLUMN website_url TO website;
    END IF;
    
    -- If both website_url and website exist, copy data from website_url to website and drop website_url
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='website_url')
       AND EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='website') THEN
        -- Copy data from website_url to website
        UPDATE profiles SET website = website_url WHERE website IS NULL AND website_url IS NOT NULL;
        -- Drop website_url column
        ALTER TABLE profiles DROP COLUMN website_url;
    END IF;
    
    -- If neither exists, add website field
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='website')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='website_url') THEN
        ALTER TABLE profiles ADD COLUMN website TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='primary_language') THEN
        ALTER TABLE profiles ADD COLUMN primary_language TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='relationship_status') THEN
        ALTER TABLE profiles ADD COLUMN relationship_status TEXT;
    END IF;
    
    -- Social platform usernames
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='x_username') THEN
        ALTER TABLE profiles ADD COLUMN x_username TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='mastodon_username') THEN
        ALTER TABLE profiles ADD COLUMN mastodon_username TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='facebook_username') THEN
        ALTER TABLE profiles ADD COLUMN facebook_username TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='reddit_username') THEN
        ALTER TABLE profiles ADD COLUMN reddit_username TEXT;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                  WHERE table_name='profiles' AND column_name='github_username') THEN
        ALTER TABLE profiles ADD COLUMN github_username TEXT;
    END IF;
END $$;