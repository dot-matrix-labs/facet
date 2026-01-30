# robert-server

REST API server that provides the interface for interacting with Robert's AI capabilities. Instantiates and manages `robert-core` instances, which handle memory management, RAG, and all AI functionality.

## Overview

`robert-server` is the server component that:
- Provides REST API endpoints for client applications (desktop app, CLI, API consumers)
- Manages authentication and authorization
- Instantiates and lifecycle-manages `robert-core` instances
- Handles concurrent sessions from multiple clients
- Provides server-side encryption and security
- Streams responses back to clients

## Architecture

```
┌─────────────────┐         HTTPS/REST          ┌──────────────────┐
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
```

## Deployment Modes

### Mode 1: Local (Default)
Desktop app automatically spawns a local server:
```bash
# Started by robert-app automatically on launch
robert-server --mode local --host 127.0.0.1 --port 8443
```
- User doesn't need to manually start the server
- Server lifecycle managed by the desktop app
- Local-only access (127.0.0.1)

### Mode 2: Remote Server (Teams/Headless/Cloud)
Started via robert-cli for shared access or headless deployments:
```bash
# Start server for team/remote/headless use
robert-cli server start --host 0.0.0.0 --port 8443

# Or with explicit config
robert-cli server start --config /etc/robert/config.toml
```
- Multiple clients can connect (desktop apps, CLI clients, APIs)
- Suitable for team deployments, cloud hosting, CI/CD
- Managed via robert-cli (start, stop, status commands)

## Quick Start

### Development Mode

```bash
# Run with default dev configuration
cargo run --bin robert-server -- --dev

# Server listens on http://127.0.0.1:8443
```

### Custom Configuration

```bash
# Use custom config file
cargo run --bin robert-server -- --config config.dev.toml
```

## API Endpoints

### Health Check

```bash
GET /api/v1/health

# Response
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

### Inference Request

```bash
POST /api/v1/inference
Authorization: Bearer <token>
Content-Type: application/json
Accept: text/event-stream

# Request body
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "query": "Analyze this codebase structure",
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

# Response (Server-Sent Events)
event: thinking
data: {"message":"Retrieving relevant context..."}

event: rag_results
data: {"chunks":[...],"relevance_scores":[...]}

event: content
data: {"type":"text","text":"Based on the codebase..."}

event: complete
data: {"session_id":"...","status":"success"}
```

### Document Ingestion

```bash
POST /api/v1/ingest
Authorization: Bearer <token>
Content-Type: application/json

# Request body
{
  "source": "file",
  "content": "...",
  "metadata": {
    "title": "API Documentation",
    "tags": ["docs", "api"],
    "category": "documentation"
  },
  "options": {
    "chunking_strategy": "semantic",
    "embedding_model": "default"
  }
}

# Response
{
  "ingestion_id": "uuid",
  "status": "completed",
  "chunks_created": 42,
  "graph_nodes_added": 15
}
```

### Memory Search

```bash
GET /api/v1/memory/search?q=authentication&limit=10&category=docs
Authorization: Bearer <token>

# Response
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

### Session Management

```bash
# Get session status
GET /api/v1/sessions/:session_id
Authorization: Bearer <token>

# Cancel session
DELETE /api/v1/sessions/:session_id
Authorization: Bearer <token>
```

## Configuration

See `config.dev.toml` for an example configuration file.

```toml
[server]
mode = "local"  # local | remote | headless
host = "127.0.0.1"
port = 8443
enable_tls = false  # Use HTTP for localhost
tls_cert = "/path/to/cert.pem"
tls_key = "/path/to/key.pem"

[auth]
dev_token = "dev-token-12345"
require_auth = false  # Optional for local dev
rate_limit_per_minute = 60

[core]
max_concurrent_sessions = 20
default_timeout_seconds = 300
enable_rag = true
embedding_model = "fastembed"

[graph]
database_path = "./dev-data"
enable_encryption = false  # Disable for easier debugging
sync_enabled = false

[limits]
max_request_size_mb = 50
max_memory_chunks = 10000
max_prompt_length = 100000

[logging]
level = "debug"
pretty_print = true
sanitize_sensitive_data = true
```

## Testing

```bash
# Run all library tests
cargo test -p robert-server --lib

# Run with verbose output
cargo test -p robert-server --lib -- --nocapture

# Run integration tests
cargo test -p robert-server --test integration_tests
```

## Project Structure

```
robert-server/
├── src/
│   ├── main.rs              # Server entry point
│   ├── lib.rs               # Library exports
│   ├── config.rs            # Configuration loading
│   ├── error.rs             # Error types
│   ├── models.rs            # Request/response types
│   ├── session.rs           # Session management
│   ├── auth.rs              # Authentication middleware
│   ├── api/
│   │   ├── mod.rs
│   │   ├── health.rs        # Health endpoint
│   │   ├── execute.rs       # Inference endpoint
│   │   ├── inference.rs     # AI inference handlers
│   │   └── sessions.rs      # Session endpoints
│   └── claude/
│       ├── mod.rs
│       ├── executor.rs      # Claude CLI integration
│       └── mock.rs          # Mock executor for testing
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
├── config.dev.toml
└── README.md
```

## Development

### Code Quality

The codebase follows strict Rust best practices:

- **TDD**: All features developed test-first
- **Documentation**: Comprehensive doc comments on all public items
- **Error Handling**: Proper error types with context
- **Async/Await**: Non-blocking I/O throughout
- **Type Safety**: Leverages Rust's type system for correctness

### Dependencies

```toml
[dependencies]
# Robert dependencies
robert-core = { path = "../robert-core" }
robert-graph = { path = "../robert-graph" }
robert-types = { workspace = true }

# Web framework
warp = { workspace = true }
tokio = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Authentication & Security
argon2 = { workspace = true }
aes-gcm = { workspace = true }

# Utilities
uuid = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
```

## Security

- **Bearer Token Authentication**: All endpoints (except health) require valid tokens
- **Rate Limiting**: Per-token request limits prevent abuse
- **Input Validation**: Comprehensive validation of all request fields
- **TLS Support**: Production deployments use TLS 1.3 encryption
- **Resource Limits**: Configurable limits on request size, memory usage, etc.
- **Audit Logging**: Complete audit trail of all operations

## Performance

- **Async/Non-blocking**: Tokio-based async runtime for high concurrency
- **Streaming**: Server-Sent Events for efficient real-time updates
- **Low Overhead**: <50ms API overhead
- **Scalable**: Handles 10-50 concurrent sessions per instance
- **Connection Pooling**: Efficient database connection management

## Deployment

### Docker

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

### Kubernetes (Teams)

See [deployment documentation](../../docs/architecture/robert-server-specification.md#kubernetes-deployment-teams) for Kubernetes manifests.

## See Also

- [Full Specification](../../docs/architecture/robert-server-specification.md)
- [Robert Core](../robert-core/README.md)
- [Robert Graph](../robert-graph/README.md)
- [Architecture Decisions](../../docs/architecture/architecture-decisions.md)
