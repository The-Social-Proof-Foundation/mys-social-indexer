-- Create initial schema for MySocial Network Indexer

-- Core tables

CREATE TABLE platforms (
    id TEXT PRIMARY KEY,            -- Platform ID from blockchain
    name TEXT NOT NULL,             -- Platform name
    description TEXT,               -- Platform description
    creator_address TEXT NOT NULL,  -- Creator address
    created_at TIMESTAMP NOT NULL,  -- Creation timestamp
    active_users_count INT DEFAULT 0,
    total_users_count INT DEFAULT 0,
    content_count INT DEFAULT 0,
    last_activity_at TIMESTAMP
);

CREATE TABLE profiles (
    id TEXT PRIMARY KEY,            -- Profile ID from blockchain
    owner_address TEXT NOT NULL,    -- Owner address 
    username TEXT,                  -- Profile username
    display_name TEXT,              -- Display name
    bio TEXT,                       -- Profile bio
    created_at TIMESTAMP NOT NULL,  -- Creation timestamp
    last_activity_at TIMESTAMP,     -- Last activity time
    followers_count INT DEFAULT 0,  -- Follower count
    following_count INT DEFAULT 0,  -- Following count
    content_count INT DEFAULT 0,    -- Total content created
    platforms_joined INT DEFAULT 0  -- Number of platforms joined
);

CREATE TABLE profile_platform_links (
    profile_id TEXT REFERENCES profiles(id),
    platform_id TEXT REFERENCES platforms(id),
    joined_at TIMESTAMP NOT NULL,   -- When profile joined platform
    last_active_at TIMESTAMP,       -- Last activity on platform
    PRIMARY KEY (profile_id, platform_id)
);

CREATE TABLE content (
    id TEXT PRIMARY KEY,            -- Content ID from blockchain
    creator_id TEXT REFERENCES profiles(id),
    platform_id TEXT REFERENCES platforms(id),
    content_type TEXT NOT NULL,     -- Post, comment, article, etc.
    parent_id TEXT,                 -- Parent content ID (for replies/comments)
    created_at TIMESTAMP NOT NULL,  -- Creation timestamp
    has_ip_registered BOOLEAN DEFAULT FALSE,
    view_count INT DEFAULT 0,
    like_count INT DEFAULT 0,
    comment_count INT DEFAULT 0,
    share_count INT DEFAULT 0
);

-- IP registration and licensing tables

CREATE TABLE intellectual_property (
    id TEXT PRIMARY KEY,            -- IP ID from blockchain
    creator_id TEXT REFERENCES profiles(id),
    title TEXT NOT NULL,
    description TEXT,
    ip_type INT NOT NULL,           -- Content, image, audio, etc.
    content_hash TEXT,              -- Hash of original content
    created_at TIMESTAMP NOT NULL,
    royalty_basis_points INT,       -- Royalty percentage in basis points
    registered_countries TEXT[],    -- Array of country codes
    ipo_tokenized BOOLEAN DEFAULT FALSE,
    total_licenses_count INT DEFAULT 0,
    active_licenses_count INT DEFAULT 0,
    total_revenue BIGINT DEFAULT 0  -- Total revenue in smallest units
);

CREATE TABLE ip_licenses (
    id TEXT PRIMARY KEY,            -- License ID from blockchain
    ip_id TEXT REFERENCES intellectual_property(id),
    licensee_id TEXT REFERENCES profiles(id),
    license_type INT NOT NULL,      -- Exclusive, non-exclusive, etc.
    terms TEXT,                     -- License terms
    granted_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP,           -- NULL for perpetual
    status INT NOT NULL,            -- Active, expired, terminated, disputed
    payment_amount BIGINT NOT NULL  -- Payment in smallest units
);

CREATE TABLE proof_of_creativity (
    id TEXT PRIMARY KEY,            -- PoC ID from blockchain
    creator_id TEXT REFERENCES profiles(id),
    ip_id TEXT REFERENCES intellectual_property(id),
    title TEXT NOT NULL,
    proof_type INT NOT NULL,        -- Timestamped, witnessed, etc.
    verification_state INT NOT NULL,-- Pending, approved, rejected
    verified_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL
);

-- Social graph tracking

CREATE TABLE follows (
    follower_id TEXT REFERENCES profiles(id),
    following_id TEXT REFERENCES profiles(id),
    followed_at TIMESTAMP NOT NULL,
    PRIMARY KEY (follower_id, following_id)
);

CREATE TABLE blocks (
    blocker_id TEXT NOT NULL,       -- Profile or platform ID
    blocked_id TEXT NOT NULL,       -- Profile ID
    blocker_type INT NOT NULL,      -- 0=Profile, 1=Platform
    reason TEXT,
    blocked_at TIMESTAMP NOT NULL,
    PRIMARY KEY (blocker_id, blocked_id)
);

-- Interaction tables

CREATE TABLE content_interactions (
    profile_id TEXT REFERENCES profiles(id),
    content_id TEXT REFERENCES content(id),
    interaction_type TEXT NOT NULL, -- like, comment, share, etc.
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (profile_id, content_id, interaction_type)
);

-- Fee distribution tracking

CREATE TABLE fee_models (
    id TEXT PRIMARY KEY,            -- Fee model ID from blockchain
    name TEXT NOT NULL,
    description TEXT,
    model_type INT NOT NULL,        -- Percentage, fixed, tiered
    fee_amount BIGINT,              -- Base fee amount
    total_split_basis_points INT,   -- Total split percentage
    owner_address TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE fee_recipients (
    id TEXT PRIMARY KEY,            -- Recipient ID
    recipient_address TEXT NOT NULL,
    recipient_name TEXT,
    total_collected BIGINT DEFAULT 0-- Total fees collected
);

CREATE TABLE fee_distributions (
    id SERIAL PRIMARY KEY,
    fee_model_id TEXT REFERENCES fee_models(id),
    transaction_amount BIGINT NOT NULL,
    total_fee_amount BIGINT NOT NULL,
    token_type TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE fee_recipient_payments (
    distribution_id INT REFERENCES fee_distributions(id),
    recipient_id TEXT REFERENCES fee_recipients(id),
    amount BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    PRIMARY KEY (distribution_id, recipient_id)
);

-- Daily statistics aggregation (for quick queries)

CREATE TABLE daily_statistics (
    date DATE PRIMARY KEY,
    new_profiles_count INT DEFAULT 0,
    active_profiles_count INT DEFAULT 0,
    new_content_count INT DEFAULT 0,
    total_interactions_count INT DEFAULT 0,
    new_ip_registrations_count INT DEFAULT 0,
    new_licenses_count INT DEFAULT 0,
    total_fees_distributed BIGINT DEFAULT 0
);

CREATE TABLE platform_daily_statistics (
    platform_id TEXT REFERENCES platforms(id),
    date DATE,
    active_users_count INT DEFAULT 0,
    new_users_count INT DEFAULT 0,
    content_created_count INT DEFAULT 0,
    total_interactions_count INT DEFAULT 0,
    PRIMARY KEY (platform_id, date)
);

-- Indexer tracking

CREATE TABLE indexer_progress (
    id TEXT PRIMARY KEY,            -- Worker ID
    last_checkpoint_processed BIGINT,-- Last processed checkpoint number
    last_processed_at TIMESTAMP     -- When the last checkpoint was processed
);

-- Create indexes for better query performance

-- Platform indexes
CREATE INDEX idx_platforms_creator ON platforms(creator_address);

-- Profile indexes
CREATE INDEX idx_profiles_owner ON profiles(owner_address);
CREATE INDEX idx_profiles_username ON profiles(username);
CREATE INDEX idx_profiles_created_at ON profiles(created_at);
CREATE INDEX idx_profiles_last_activity ON profiles(last_activity_at);

-- Content indexes
CREATE INDEX idx_content_creator ON content(creator_id);
CREATE INDEX idx_content_platform ON content(platform_id);
CREATE INDEX idx_content_created_at ON content(created_at);
CREATE INDEX idx_content_parent ON content(parent_id);
CREATE INDEX idx_content_type ON content(content_type);

-- IP indexes
CREATE INDEX idx_ip_creator ON intellectual_property(creator_id);
CREATE INDEX idx_ip_created_at ON intellectual_property(created_at);
CREATE INDEX idx_ip_type ON intellectual_property(ip_type);

-- License indexes
CREATE INDEX idx_licenses_ip ON ip_licenses(ip_id);
CREATE INDEX idx_licenses_licensee ON ip_licenses(licensee_id);
CREATE INDEX idx_licenses_status ON ip_licenses(status);

-- Social graph indexes
CREATE INDEX idx_follows_follower ON follows(follower_id);
CREATE INDEX idx_follows_following ON follows(following_id);
CREATE INDEX idx_blocks_blocker ON blocks(blocker_id);
CREATE INDEX idx_blocks_blocked ON blocks(blocked_id);

-- Fee distribution indexes
CREATE INDEX idx_fee_distributions_model ON fee_distributions(fee_model_id);
CREATE INDEX idx_fee_distributions_created ON fee_distributions(created_at);
CREATE INDEX idx_fee_payments_recipient ON fee_recipient_payments(recipient_id);

-- Statistics indexes
CREATE INDEX idx_platform_stats_date ON platform_daily_statistics(date);