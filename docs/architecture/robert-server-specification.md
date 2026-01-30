# Robert Server Specification

## Overview

The Robert Server is a Rust-based web service built with the Warp framework that provides a REST API for interacting with Robert's core AI capabilities. It instantiates and manages `robert-core` instances, which handle memory management, prompt engineering, RAG loops, and all main functionality of the application.

## Architecture

```
┌─────────────────┐         HTTPS/REST           ┌──────────────────┐
│   Robert App    │ ◄──────────────────────────► │  Robert Server   │
│   (Desktop)     │    (SSL Encrypted)           │   (Rust/Warp)    │
└─────────────────┘                              └──────────────────┘
                                                           │
                                                           │ Instantiates
                                                           ▼
                                                  ┌──────────────────┐
                                                  │   robert-core    │
                                                  │  (AI/RAG Engine) │
                                                  └──────────────────┘
                                                           │
                                                           │ Uses
                                                           ▼
                                                  ┌──────────────────┐
                                                  │  robert-graph    │
                                                  │ (DB Management)  │
                                                  └──────────────────┘

     OR (Headless/CLI Mode)

┌─────────────────┐                              ┌──────────────────┐
│   robert-cli    │ ────────────────────────────► │  Robert Server   │
│   (CLI Tool)    │    Starts server process     │   (Rust/Warp)    │
└─────────────────┘                              └──────────────────┘
                                                           │
                                                           │ Instantiates
                                                           ▼
                                                  ┌──────────────────┐
                                                  │   robert-core    │
                                                  │  (AI/RAG Engine) │
                                                  └──────────────────┘
```

## Component Responsibilities

### robert-server
- Provides REST API endpoints for client applications
- Manages authentication and authorization
- Instantiates and lifecycle-manages robert-core instances
- Handles concurrent sessions from multiple clients
- Provides server-side encryption and security
- Streams responses back to clients (SSE/WebSocket)

### robert-core
- Core AI/RAG engine with all main application logic
- Memory management and context retention
- Prompt engineering and optimization
- RAG (Retrieval-Augmented Generation) loop implementation
- Integration with Claude CLI and other AI services
- Business logic and workflow orchestration

### robert-graph
- Database management (SurrealDB)
- Graph-based data storage and retrieval
- Database encryption and security
- Data synchronization (local <-> cloud)
- Query optimization and indexing

## Deployment Modes

### Mode 1: Local Desktop (Default)

The desktop app spawns a local robert-server instance automatically:

```
robert-app (Tauri)
    └─► Spawns local robert-server (127.0.0.1:8443)
            └─► Creates robert-core instance
                    └─► Uses local robert-graph database (~/.robert/data)
```

**Use Cases:**
- Individual users
- Complete privacy (data never leaves machine)
- No internet required after setup
- Fast local processing

**Desktop App Configuration:**
```json
{
  "server": {
    "mode": "local",
    "auto_start": true
  }
}
```

### Mode 2: Remote Server (Teams/Headless)

robert-cli starts a server that multiple clients can connect to:

```
# Start server (typically on a remote machine or cloud)
robert-cli server start --host 0.0.0.0 --port 8443

# Then clients connect:
robert-app (Desktop) ──► robert-server (https://team.example.com:8443)
                                 └─► Shared robert-core instance
                                         └─► Shared robert-graph database

# Or use via CLI:
robert-cli query "..." --server https://team.example.com:8443
```

**Use Cases:**
- **Team Deployments**: Shared knowledge base across organization
- **Cloud Hosting**: Deploy on AWS/GCP/Azure for remote access
- **CI/CD Integrations**: Automated workflows and testing
- **API Access**: Programmatic access from other services
- **Headless Mode**: Server-only deployments without UI

**Server Configuration (config.toml):**
```toml
[server]
host = "0.0.0.0"  # Listen on all interfaces
port = 8443
enable_tls = true
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

[auth]
require_auth = true
tokens = ["team-token-1", "team-token-2"]

[graph]
database_path = "/var/lib/robert/data"
enable_encryption = true
sync_enabled = false  # Or enable for multi-region sync
```

**Desktop App Configuration (connecting to remote):**
```json
{
  "server": {
    "mode": "remote",
    "url": "https://robert.example.com:8443",
    "api_token": "team-token-1"
  }
}
```

**CLI Configuration (connecting to remote):**
```bash
# Set via environment or config
export ROBERT_SERVER_URL=https://robert.example.com:8443
export ROBERT_API_TOKEN=team-token-1

robert-cli query "analyze this codebase"
robert-cli ingest /path/to/docs
```

## Key Requirements

### 1. Network Protocol

- **Transport**: HTTPS with WebSocket upgrade for streaming
- **SSL/TLS**: Mandatory TLS 1.3 encryption (optional for localhost)
- **Authentication**: Bearer token or API key based authentication
- **Port**: Configurable (default: 8443)

### 2. Request Flow

1. Client establishes SSL connection to robert-server
2. Client sends request (query, ingestion, etc.)
3. Server validates authentication and request format
4. Server routes request to appropriate robert-core handler
5. robert-core processes request using robert-graph for data access
6. Server streams responses back to client
7. Connection remains open for streaming or closes after response

### 3. API Endpoints

#### `POST /api/v1/inference`

Execute an AI inference request with RAG context.

**Request Headers:**
```
Authorization: Bearer <token>
Content-Type: application/json
Accept: text/event-stream
```

**Request Body:**
```json
{
  "session_id": "uuid",
  "query": "string",
  "context": {
    "screenshots": [],
    "documents": [],
    "memory_filter": {}
  },
  "options": {
    "timeout_seconds": 300,
    "max_tokens": 100000,
    "stream": true,
    "use_rag": true
  }
}
```

**Response (Server-Sent Events):**
```
event: thinking
data: {"message": "Retrieving relevant context..."}

event: rag_results
data: {"chunks": [...], "relevance_scores": [...]}

event: content
data: {"type": "text", "text": "Based on the context..."}

event: complete
data: {"session_id": "uuid", "status": "success"}
```

#### `POST /api/v1/ingest`

Ingest documents or data into the knowledge graph.

**Request Body:**
```json
{
  "source": "file|url|text",
  "content": "...",
  "metadata": {
    "title": "string",
    "tags": ["tag1", "tag2"],
    "category": "string"
  },
  "options": {
    "chunking_strategy": "semantic|fixed",
    "embedding_model": "default|custom"
  }
}
```

**Response:**
```json
{
  "ingestion_id": "uuid",
  "status": "processing|completed|failed",
  "chunks_created": 42,
  "graph_nodes_added": 15
}
```

#### `GET /api/v1/health`

Health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "components": {
    "robert_core": "healthy",
    "robert_graph": "healthy",
    "database": "connected"
  },
  "uptime_seconds": 12345
}
```

#### `GET /api/v1/sessions/:session_id`

Query status of an active or completed session.

**Response:**
```json
{
  "session_id": "uuid",
  "status": "running|completed|failed",
  "started_at": "2025-10-17T10:30:00Z",
  "completed_at": "2025-10-17T10:35:00Z",
  "memory_created": true,
  "error": null
}
```

#### `DELETE /api/v1/sessions/:session_id`

Cancel a running session.

**Response:**
```json
{
  "session_id": "uuid",
  "status": "cancelled"
}
```

#### `GET /api/v1/memory/search`

Search the knowledge graph.

**Query Parameters:**
```
?q=search+query&limit=10&category=docs
```

**Response:**
```json
{
  "results": [
    {
      "id": "uuid",
      "content": "...",
      "relevance_score": 0.95,
      "metadata": {...}
    }
  ],
  "total_results": 42
}
```

### 4. Security Considerations

#### Authentication

- API key/bearer token validation on every request
- Rate limiting per token (configurable)
- IP whitelisting (optional for remote mode)
- Token rotation support
- Team-based access control (for shared deployments)

#### Input Validation

- Maximum request size limits (default: 50MB)
- Content sanitization
- JSON schema validation
- SQL injection prevention (via SurrealDB)

#### Process Isolation

- Each robert-core instance runs in isolated context
- Resource limits (CPU, memory, file descriptors)
- Timeout enforcement (default: 5 minutes)
- Automatic cleanup of stale sessions

#### Data Privacy

- End-to-end encryption for remote connections
- Local database encryption via robert-graph
- No persistent logging of sensitive content
- Configurable data retention policies
- GDPR compliance for team deployments

### 5. Performance Requirements

- **Concurrent Sessions**: Support 10-50 concurrent executions
- **Latency**: <50ms API overhead
- **Throughput**: Handle 10MB/s per connection
- **Memory**: <200MB base + <500MB per active session
- **Database**: Fast vector similarity search (<100ms)

### 6. Configuration

```toml
[server]
mode = "local"  # local | remote | headless
host = "127.0.0.1"
port = 8443
enable_tls = true
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

[auth]
tokens = ["token1", "token2"]
rate_limit_per_minute = 60
require_auth = true

[core]
max_concurrent_sessions = 20
default_timeout_seconds = 300
enable_rag = true
embedding_model = "fastembed"

[graph]
database_path = "~/.robert/data"
enable_encryption = true
sync_enabled = false
sync_url = ""  # For team deployments

[limits]
max_request_size_mb = 50
max_memory_chunks = 10000
max_prompt_length = 100000

[logging]
level = "info"
format = "json"
sanitize_sensitive_data = true
```

### 7. Error Handling

#### Error Types

- `AUTH_FAILED`: Invalid or missing authentication
- `RATE_LIMITED`: Too many requests
- `INVALID_REQUEST`: Malformed request body
- `CORE_UNAVAILABLE`: robert-core initialization failed
- `DATABASE_ERROR`: robert-graph connection issue
- `EXECUTION_ERROR`: Processing failed
- `TIMEOUT`: Request exceeded timeout limit
- `INTERNAL_ERROR`: Server-side error

#### Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "session_id": "uuid",
    "timestamp": "2025-10-17T10:30:00Z",
    "retry_after_seconds": 60
  }
}
```

### 8. Local Development & Testing

#### Development Server

```bash
# Start local development server
cargo run --bin robert-server -- --dev

# Or with explicit configuration
cargo run --bin robert-server -- \
  --config config.dev.toml \
  --dev-mode
```

**Development Mode Features:**
- HTTP support (no TLS required for localhost)
- Simplified authentication (optional)
- Verbose logging enabled
- CORS enabled for all origins
- Hot-reload configuration

#### Configuration: `config.dev.toml`

```toml
[server]
mode = "local"
host = "127.0.0.1"
port = 8443
enable_tls = false  # Use HTTP for localhost

[auth]
dev_token = "dev-token-12345"
require_auth = false  # Optional: disable for local testing

[core]
default_timeout_seconds = 300

[graph]
database_path = "./dev-data"
enable_encryption = false  # Disable for easier debugging

[logging]
level = "debug"
pretty_print = true
```

#### Integration with Tauri App

```json
// package.json
{
  "scripts": {
    "dev": "tauri dev",
    "dev:local": "concurrently \"cargo run --bin robert-server -- --dev\" \"tauri dev\"",
    "test:e2e": "cargo test --workspace"
  }
}
```

### 9. Deployment

#### Docker Support

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin robert-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/robert-server /usr/local/bin/
EXPOSE 8443
CMD ["robert-server"]
```

#### Kubernetes Deployment (Teams)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: robert-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: robert-server
  template:
    metadata:
      labels:
        app: robert-server
    spec:
      containers:
      - name: robert-server
        image: robert-server:latest
        ports:
        - containerPort: 8443
        env:
        - name: SERVER_MODE
          value: "remote"
        volumeMounts:
        - name: config
          mountPath: /etc/robert
```

### 10. Team/Enterprise Features

For organizations deploying shared robert-server instances:

- **Multi-tenancy**: Isolated workspaces per team
- **Shared Knowledge Base**: Centralized memory with access control
- **Usage Analytics**: Track queries, costs, performance
- **Audit Logging**: Complete audit trail of all operations
- **SSO Integration**: SAML/OAuth2 authentication
- **High Availability**: Load balanced, redundant instances

## Implementation Dependencies

```toml
[dependencies]
# robert dependencies
robert-core = { path = "../robert-core" }
robert-graph = { path = "../robert-graph" }
robert-types = { workspace = true }

# Web framework
warp = { workspace = true }
tokio = { workspace = true }
tokio-stream = { version = "0.1", features = ["net"] }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Authentication & Security
argon2 = { workspace = true }
aes-gcm = { workspace = true }

# Utilities
uuid = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
```

## Development Roadmap

### Phase 1: MVP (Current)
- Basic REST API
- Local server mode
- Single session management
- robert-core integration
- Development tooling

### Phase 2: Streaming & RAG
- WebSocket/SSE streaming
- Full RAG pipeline integration
- Memory persistence via robert-graph
- Multi-session support

### Phase 3: Remote Mode
- Remote server support
- Team authentication
- Database synchronization
- Production hardening

### Phase 4: Enterprise
- Multi-tenancy
- SSO integration
- Advanced monitoring
- HA deployment patterns

## References

- Warp documentation: https://docs.rs/warp
- Tokio async runtime: https://tokio.rs
- SurrealDB: https://surrealdb.com
- Server-Sent Events spec: https://html.spec.whatwg.org/multipage/server-sent-events.html
