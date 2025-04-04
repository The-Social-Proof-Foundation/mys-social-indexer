-- This migration adds fields to profiles table
-- If we need to revert, we can drop the columns
-- Note: Some columns already exist, so we check first

DO $$ 
BEGIN
    -- Only drop columns that might have been added in the up migration
    -- Basic columns that were already there before the migration should not be dropped

    -- Drop religion field if it exists
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='religion') THEN
        ALTER TABLE profiles DROP COLUMN religion;
    END IF;
    
    -- We don't have website_url anymore, since we renamed it to website
    -- So no need to handle that special case
    
    -- Drop sensitive fields
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='birthdate') THEN
        ALTER TABLE profiles DROP COLUMN birthdate;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='current_location') THEN
        ALTER TABLE profiles DROP COLUMN current_location;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='raised_location') THEN
        ALTER TABLE profiles DROP COLUMN raised_location;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='phone') THEN
        ALTER TABLE profiles DROP COLUMN phone;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='email') THEN
        ALTER TABLE profiles DROP COLUMN email;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='gender') THEN
        ALTER TABLE profiles DROP COLUMN gender;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='political_view') THEN
        ALTER TABLE profiles DROP COLUMN political_view;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='education') THEN
        ALTER TABLE profiles DROP COLUMN education;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='primary_language') THEN
        ALTER TABLE profiles DROP COLUMN primary_language;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='relationship_status') THEN
        ALTER TABLE profiles DROP COLUMN relationship_status;
    END IF;
    
    -- Drop social username fields
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='x_username') THEN
        ALTER TABLE profiles DROP COLUMN x_username;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='mastodon_username') THEN
        ALTER TABLE profiles DROP COLUMN mastodon_username;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='facebook_username') THEN
        ALTER TABLE profiles DROP COLUMN facebook_username;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='reddit_username') THEN
        ALTER TABLE profiles DROP COLUMN reddit_username;
    END IF;
    
    IF EXISTS (SELECT 1 FROM information_schema.columns 
              WHERE table_name='profiles' AND column_name='github_username') THEN
        ALTER TABLE profiles DROP COLUMN github_username;
    END IF;
END $$;