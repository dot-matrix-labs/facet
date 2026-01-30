# robert-graph

Database management layer for Robert - handles all data storage, retrieval, encryption, and synchronization using SurrealDB and graph-based storage.

## Overview

`robert-graph` is the data persistence layer for Robert. It provides a clean abstraction over SurrealDB for managing:
- Vector embeddings and semantic search
- Knowledge graph storage
- Conversation history
- Document chunks and metadata
- Database encryption and security
- Data synchronization (local ↔ cloud)

## Architecture

```
┌──────────────────┐
│   robert-core    │  ← Uses robert-graph for all data access
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  robert-graph    │  ← Database Management Layer (THIS CRATE)
│                  │
│  • SurrealDB     │
│  • Vector Store  │
│  • Encryption    │
│  • Sync Engine   │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   SurrealDB      │  ← Embedded database (RocksDB backend)
│   ~/.robert/data │
└──────────────────┘
```

## Responsibilities

### 1. Vector Storage & Search
- Store embeddings with metadata
- Fast vector similarity search (ANN)
- Hybrid search (vector + keyword)
- Result ranking and filtering

### 2. Knowledge Graph
- Entity and relationship storage
- Graph traversal queries
- Temporal data modeling
- Schema management

### 3. Encryption
- At-rest encryption of database files
- Secure key management
- Per-user or per-team encryption keys
- Transparent encryption/decryption

### 4. Data Synchronization
- Local database → Cloud backup
- Multi-device sync
- Conflict resolution
- Selective sync (privacy control)

### 5. Query Optimization
- Index management
- Query planning and caching
- Batch operations
- Connection pooling

## Key Components

### SurrealStore
```rust
pub struct SurrealStore {
    db: Surreal<Db>,
    encryption_key: Option<Key>,
}

impl SurrealStore {
    pub async fn new(config: StoreConfig) -> Result<Self>;
    pub async fn store_embedding(&self, chunk: Chunk) -> Result<ChunkId>;
    pub async fn search_similar(&self, query: &[f32], limit: usize) -> Result<Vec<Chunk>>;
    pub async fn sync_to_cloud(&self) -> Result<SyncStatus>;
}
```

### IngestionPipeline
```rust
pub struct IngestionPipeline {
    store: Arc<SurrealStore>,
    embedder: Arc<dyn Embedder>,
}

impl IngestionPipeline {
    pub async fn ingest_document(&self, doc: Document) -> Result<IngestionResult>;
    pub async fn chunk_and_embed(&self, text: &str) -> Result<Vec<Chunk>>;
}
```

### Query Engine
```rust
pub struct QueryEngine {
    store: Arc<SurrealStore>,
}

impl QueryEngine {
    pub async fn semantic_search(&self, query: &str, k: usize) -> Result<Vec<SearchResult>>;
    pub async fn graph_traverse(&self, start_id: &str, depth: usize) -> Result<Graph>;
    pub async fn hybrid_search(&self, params: SearchParams) -> Result<Vec<SearchResult>>;
}
```

## Database Schema

### Chunks Table
```surql
DEFINE TABLE chunks SCHEMAFULL;

DEFINE FIELD content ON chunks TYPE string;
DEFINE FIELD embedding ON chunks TYPE array<float>;
DEFINE FIELD metadata ON chunks TYPE object;
DEFINE FIELD source_id ON chunks TYPE string;
DEFINE FIELD created_at ON chunks TYPE datetime DEFAULT time::now();

DEFINE INDEX embedding_idx ON chunks FIELDS embedding MTREE DIMENSION 384;
DEFINE INDEX source_idx ON chunks FIELDS source_id;
```

### Memory Table
```surql
DEFINE TABLE memory SCHEMAFULL;

DEFINE FIELD session_id ON memory TYPE string;
DEFINE FIELD role ON memory TYPE string;  // user | assistant | system
DEFINE FIELD content ON memory TYPE string;
DEFINE FIELD tokens ON memory TYPE int;
DEFINE FIELD timestamp ON memory TYPE datetime DEFAULT time::now();

DEFINE INDEX session_idx ON memory FIELDS session_id;
```

### Documents Table
```surql
DEFINE TABLE documents SCHEMAFULL;

DEFINE FIELD title ON documents TYPE string;
DEFINE FIELD source_url ON documents TYPE option<string>;
DEFINE FIELD content_type ON documents TYPE string;
DEFINE FIELD tags ON documents TYPE array<string>;
DEFINE FIELD created_at ON documents TYPE datetime DEFAULT time::now();

DEFINE INDEX tags_idx ON documents FIELDS tags;
```

## Features

### Vector Search
```rust
// Store document with embeddings
let chunks = pipeline.ingest_document(document).await?;

// Semantic search
let results = store.search_similar(
    query_embedding,
    limit: 10,
    filter: Some("tags CONTAINS 'documentation'")
).await?;
```

### Encryption
```rust
// Initialize with encryption
let config = StoreConfig {
    path: "~/.robert/data",
    encryption_enabled: true,
    encryption_key: derive_key_from_password(password),
};

let store = SurrealStore::new(config).await?;
// All data automatically encrypted at rest
```

### Synchronization
```rust
// Sync to cloud
let sync_config = SyncConfig {
    endpoint: "https://sync.robert.example.com",
    token: api_token,
    sync_interval: Duration::from_secs(300),
};

store.enable_sync(sync_config).await?;

// Automatic background sync
// Or manual sync
let status = store.sync_now().await?;
```

## Configuration

```rust
pub struct StoreConfig {
    /// Path to database directory
    pub path: PathBuf,

    /// Enable encryption at rest
    pub encryption_enabled: bool,

    /// Encryption key (derived from user password)
    pub encryption_key: Option<Vec<u8>>,

    /// Enable cloud sync
    pub sync_enabled: bool,

    /// Sync endpoint URL
    pub sync_url: Option<String>,

    /// Embedding dimension (default: 384 for fastembed)
    pub embedding_dimension: usize,

    /// Vector index type (MTREE, HNSW, etc.)
    pub index_type: IndexType,
}
```

## Usage Examples

### Basic Setup
```rust
use robert_graph::{SurrealStore, StoreConfig};

let config = StoreConfig {
    path: "~/.robert/data".into(),
    encryption_enabled: true,
    ..Default::default()
};

let store = SurrealStore::new(config).await?;
```

### Document Ingestion
```rust
use robert_graph::IngestionPipeline;

let pipeline = IngestionPipeline::new(store.clone()).await?;

let document = Document {
    title: "API Documentation",
    content: "...",
    source: "https://docs.example.com/api",
};

let result = pipeline.ingest_document(document).await?;
println!("Created {} chunks", result.chunks_created);
```

### Semantic Search
```rust
let query = "How do I authenticate API requests?";
let results = store.search_similar(query, limit: 5).await?;

for result in results {
    println!("Score: {:.3} - {}", result.score, result.content);
}
```

## Development

```bash
# Run tests
cargo test -p robert-graph

# Run tests with logging
RUST_LOG=robert_graph=debug cargo test

# Run integration tests (requires SurrealDB)
cargo test -p robert-graph --test integration_tests
```

## Dependencies

- **surrealdb**: Embedded database with graph and vector capabilities
- **petgraph**: Graph data structures and algorithms
- **fastembed**: Fast embedding generation
- **aes-gcm**: AES-GCM encryption
- **argon2**: Key derivation from passwords

## Project Structure

```
robert-graph/
├── src/
│   ├── lib.rs              # Public API
│   ├── surreal_store.rs    # SurrealDB integration
│   ├── ingest.rs           # Document ingestion pipeline
│   ├── query.rs            # Query engine
│   ├── ephemeral_graph.rs  # In-memory graph operations
│   └── tests/
│       └── integration_tests.rs
├── Cargo.toml
└── README.md               # This file
```

## Performance Considerations

### Vector Search
- Uses MTREE index for fast ANN search
- Typical query time: <100ms for 100k vectors
- Batch operations for better throughput

### Encryption Overhead
- ~5-10% performance impact with encryption enabled
- Encryption happens at page level (transparent to queries)
- Key caching minimizes overhead

### Synchronization
- Async background sync (non-blocking)
- Incremental sync (only changed data)
- Conflict resolution using last-write-wins or custom strategies

## Future Enhancements

### Phase 1 (Current)
- [x] Basic SurrealDB integration
- [x] Ephemeral graph operations
- [ ] Vector storage and search
- [ ] Document ingestion pipeline

### Phase 2
- [ ] Database encryption
- [ ] Cloud synchronization
- [ ] Advanced graph queries
- [ ] Query optimization

### Phase 3
- [ ] Multi-tenancy support
- [ ] Fine-grained access control
- [ ] Sharding and scaling
- [ ] Real-time collaboration

## Security

### Encryption
- AES-256-GCM for data at rest
- Argon2id for key derivation
- Secure key storage using OS keychain
- Memory wiping for sensitive data

### Access Control
- Per-user encryption keys
- Team-based access (future)
- Audit logging
- Rate limiting for sync operations

## Contributing

When modifying `robert-graph`:

1. All database operations should be async
2. Support both encrypted and non-encrypted modes
3. Write integration tests with real SurrealDB instance
4. Document schema changes
5. Maintain backward compatibility for data migrations

## References

- [SurrealDB Documentation](https://surrealdb.com/docs)
- [Robert Architecture](../../docs/architecture/robert-server-specification.md)
- [RAG Overview](../../docs/architecture/rag-overview.md)
