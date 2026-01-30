# Facet Server

Remote execution server for the Facet desktop application. Receives SSL-encrypted requests and forwards them to headless claude-cli processes.

## Features

- **REST API** with Server-Sent Events (SSE) streaming
- **Authentication** via bearer tokens with rate limiting
- **Session Management** for tracking concurrent executions
- **Mock Mode** for testing without claude-cli installed
- **Development Mode** with HTTP support for local testing
- **Production Ready** with TLS support and comprehensive error handling

## Quick Start

### Development Mode

```bash
# Run with default dev configuration
cargo run --bin facet-server -- --dev

# Run with mock executor (no claude-cli required)
cargo run --bin facet-server -- --dev --mock

# Server listens on http://127.0.0.1:8443
```

### Custom Configuration

```bash
# Use custom config file
cargo run --bin facet-server -- --config /path/to/config.toml
```

## API Endpoints

### Health Check

```bash
GET /api/v1/health

# Response
{
  "status": "healthy",
  "version": "1.0.0",
  "claude_cli_available": true,
  "uptime_seconds": 12345
}
```

### Execute Request

```bash
POST /api/v1/execute
Authorization: Bearer <token>
Content-Type: application/json
Accept: text/event-stream

# Request body - see specification for full schema
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "context": {
    "screenshots": [...],
    "dom_state": {...},
    "user_intent": "Click the login button"
  },
  "prompt": "Help me log in to this website",
  "options": {
    "timeout_seconds": 300,
    "max_tokens": 100000,
    "stream": true
  }
}

# Response (Server-Sent Events)
event: content
data: {"type":"content","text":"Analyzing screenshot..."}

event: tool_use
data: {"type":"tool_use","tool":"cdp_command","params":{...}}

event: complete
data: {"type":"complete","session_id":"...","status":"success"}
```

### Get Session Status

```bash
GET /api/v1/sessions/:session_id
Authorization: Bearer <token>

# Response
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "running",
  "started_at": "2025-10-17T10:30:00Z",
  "completed_at": null,
  "error": null
}
```

### Cancel Session

```bash
DELETE /api/v1/sessions/:session_id
Authorization: Bearer <token>

# Response
{
  "session_id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "cancelled"
}
```

## Configuration

See `config.dev.toml` for an example configuration file.

```toml
[server]
host = "127.0.0.1"
port = 8443
dev_mode = true
enable_tls = false

[auth]
dev_token = "dev-token-12345"
require_auth = false
rate_limit_per_minute = 100

[claude]
binary_path = "claude"
mock_mode = false
default_timeout_seconds = 300
max_concurrent_sessions = 20

[limits]
max_request_size_mb = 50
max_screenshot_count = 10
max_prompt_length = 50000

[logging]
level = "debug"
pretty_print = true
sanitize_sensitive_data = true
```

## Testing

```bash
# Run all library tests
cargo test -p facet-server --lib

# Run with verbose output
cargo test -p facet-server --lib -- --nocapture

# Run specific test
cargo test -p facet-server --lib test_mock_executor_success
```

## Project Structure

```
facet-server/
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
│   │   ├── execute.rs       # Execute endpoint
│   │   └── sessions.rs      # Session endpoints
│   └── claude/
│       ├── mod.rs
│       ├── executor.rs      # Real Claude CLI executor
│       └── mock.rs          # Mock executor for testing
├── tests/
│   └── integration_tests.rs
├── benches/
│   └── streaming_benchmark.rs
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

### Test Coverage

- **106 unit tests** covering all core functionality
- **Mock executor** for rapid testing without dependencies
- **Integration tests** for end-to-end flows
- **Benchmark tests** for performance validation

## Architecture

```
┌─────────────────┐         HTTPS/SSE          ┌──────────────────┐
│   Facet App    │ ◄─────────────────────────► │  Facet Server   │
│   (Desktop)     │                              │   (Rust/Warp)    │
└─────────────────┘                              └──────────────────┘
                                                          │
                                                          │ Spawns
                                                          ▼
                                                 ┌──────────────────┐
                                                 │   claude-cli     │
                                                 │   (Headless)     │
                                                 └──────────────────┘
```

## Security

- **Bearer Token Authentication**: All endpoints (except health) require valid tokens
- **Rate Limiting**: Per-token request limits prevent abuse
- **Input Validation**: Comprehensive validation of all request fields
- **TLS Support**: Production deployments use TLS 1.3 encryption
- **Resource Limits**: Configurable limits on request size, screenshot count, etc.

## Performance

- **Async/Non-blocking**: Tokio-based async runtime for high concurrency
- **Streaming**: Server-Sent Events for efficient real-time updates
- **Low Overhead**: <100ms forwarding latency
- **Scalable**: Handles 10-50 concurrent sessions per instance

## License

MIT

## See Also

- [Full Specification](/home/jeef/facet/docs/ROBERT_SERVER_SPECIFICATION.md)
- [Facet App](/home/jeef/facet/crates/facet-app)
- [Facet CLI](/home/jeef/facet/crates/facet-cli)
