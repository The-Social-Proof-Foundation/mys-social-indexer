# MySocial Social Indexer

A simplified indexer for the MySocial blockchain that focuses on tracking user profiles and their updates.

## Features

- **Profile Indexing**: Tracks profile creation and updates
- **Database Storage**: Stores profile data in PostgreSQL
- **REST API**: Provides endpoints for accessing profile data
- **Configurable**: Customizable via environment variables
- **Containerized**: Easy deployment with Docker

## Architecture

The indexer consists of the following components:

1. **Blockchain Listener**: Processes MySocial blockchain checkpoints and extracts events
2. **Event Processor**: Identifies and processes profile-related events
3. **Database**: Stores profile data in PostgreSQL
4. **API Server**: Exposes profile data through REST endpoints

## Getting Started

### Prerequisites

- Rust 1.75+
- PostgreSQL 15+
- Docker and Docker Compose (for containerized deployment)

### Running Locally

1. Clone the repository
2. Install dependencies:
   ```
   cargo build
   ```
3. Set up the database:
   ```
   createdb mys_social_indexer
   ```
4. Run the indexer:
   ```
   cargo run
   ```

### Using Docker

```bash
docker-compose up -d
```

This will start:
- PostgreSQL database
- Social Profile Indexer

## Configuration

The indexer can be configured via environment variables:

```bash
# Database configuration
DATABASE_URL=postgres://postgres:postgres@localhost:5432/myso_social_indexer
DATABASE_MAX_CONNECTIONS=10

# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Indexer configuration
CHECKPOINT_URL=https://checkpoints.testnet.mysocial.network
START_CHECKPOINT=0
INDEXER_CONCURRENCY=5

# Package configuration
PROFILE_PACKAGE_ADDRESS=0x61e95f6a3382232579263b473847d00ac1d56dbe69e22674de7d35b4ce26e588

# Logging
RUST_LOG=info,mys_social_indexer=debug
```

## API Endpoints

### Profiles

- `GET /profiles` - List profiles with pagination (query params: limit, offset)
- `GET /profiles/:address` - Get profile by owner address
- `GET /profiles/username/:username` - Get profile by username

### Health

- `GET /health` - Check the health of the API server

## Database Schema

```sql
-- Profiles Table
CREATE TABLE profiles (
    id SERIAL PRIMARY KEY,
    owner_address VARCHAR(255) NOT NULL,
    username VARCHAR(100) NOT NULL,
    display_name VARCHAR(255),
    bio TEXT,
    avatar_url VARCHAR(255),
    website_url VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Indexer State Table
CREATE TABLE indexer_checkpoint_state (
    id SERIAL PRIMARY KEY,
    last_processed_checkpoint BIGINT NOT NULL,
    last_processed_timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);
```

## License

Apache License 2.0