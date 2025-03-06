# MySocial Network Indexer

A comprehensive indexer for the MySocial blockchain that tracks all social network activities including profiles, platforms, content, interactions, IP registrations, and fee distributions.

## Features

- **Full Social Graph Indexing**: Tracks profiles, follows, blocks, and relationships
- **Platform Analytics**: Monitors platform growth, user engagement, and content trends
- **Content Metrics**: Records content creation, interactions, and popularity
- **IP Registration Tracking**: Indexes intellectual property registrations and licenses
- **Fee Distribution Analysis**: Monitors fee models, distributions, and recipient payments
- **Real-time Statistics**: Aggregates daily metrics for quick analysis
- **GraphQL & REST API**: Provides comprehensive API for accessing indexed data

## Architecture

The indexer consists of the following components:

1. **Blockchain Listener**: Processes MySocial blockchain checkpoints and extracts events
2. **Event Processor**: Parses and processes social network-related events
3. **Database**: Stores indexed data in a PostgreSQL database
4. **API Server**: Exposes data through REST endpoints
5. **Metrics & Monitoring**: Tracks indexer performance and health

## Getting Started

### Prerequisites

- Rust 1.70+
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
- Social Network Indexer
- Grafana dashboard for monitoring

## Configuration

The indexer can be configured via environment variables:

```bash
# Database configuration
DATABASE_URL=postgres://postgres:postgres@localhost:5432/mys_social_indexer
DATABASE_MAX_CONNECTIONS=10
DATABASE_CONNECTION_TIMEOUT=30

# Indexer configuration
CHECKPOINT_URL=https://checkpoints.mainnet.mysocial.io
INITIAL_CHECKPOINT=0
CONCURRENCY=5
PROGRESS_FILE_PATH=/tmp/social_indexer_progress
MONITORING_INTERVAL=30

# API configuration
API_HOST=0.0.0.0
API_PORT=3000
ENABLE_CORS=true

# Metrics configuration
METRICS_ENABLED=true
METRICS_PORT=9000
```

## API Endpoints

### Profiles

- `GET /api/profiles` - List profiles
- `GET /api/profiles/:id` - Get profile details
- `GET /api/profiles/:id/following` - Get profiles followed by user
- `GET /api/profiles/:id/followers` - Get followers of user
- `GET /api/profiles/:id/content` - Get content created by user
- `GET /api/profiles/:id/platforms` - Get platforms joined by user

### Platforms

- `GET /api/platforms` - List platforms
- `GET /api/platforms/:id` - Get platform details
- `GET /api/platforms/:id/users` - Get platform users
- `GET /api/platforms/:id/content` - Get platform content
- `GET /api/platforms/:id/stats` - Get platform statistics

### Content

- `GET /api/content` - List content
- `GET /api/content/:id` - Get content details
- `GET /api/content/:id/interactions` - Get content interactions
- `GET /api/content/trending` - Get trending content

### Intellectual Property

- `GET /api/ip` - List IP assets
- `GET /api/ip/:id` - Get IP details
- `GET /api/ip/:id/licenses` - Get IP licenses

### Fee Distribution

- `GET /api/fees/models` - List fee models
- `GET /api/fees/models/:id` - Get fee model details
- `GET /api/fees/recipients` - List fee recipients
- `GET /api/fees/distributions` - List fee distributions

### Statistics

- `GET /api/stats/daily` - Get daily statistics
- `GET /api/stats/platforms` - Get platform statistics
- `GET /api/stats/overview` - Get overall statistics

## License

Apache License 2.0