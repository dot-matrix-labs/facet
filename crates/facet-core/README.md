# robert-core

The core AI/RAG engine for Robert - handles all main application logic, memory management, prompt engineering, and the RAG (Retrieval-Augmented Generation) loop.

## Overview

`robert-core` is the heart of the Robert application. It contains all the business logic, AI integration, memory management, and RAG pipeline implementation. This crate is instantiated by `robert-server` and depends on `robert-graph` for all database operations.

## Architecture

```
┌──────────────────┐
│  robert-server   │  ← Instantiates and manages robert-core
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   robert-core    │  ← Core AI/RAG Engine (THIS CRATE)
│                  │
│  • Memory Mgmt   │
│  • Prompt Eng    │
│  • RAG Loop      │
│  • AI Integration│
│  • Workflows     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  robert-graph    │  ← Database management
└──────────────────┘
```

## Responsibilities

### 1. Memory Management
- Context window management and optimization
- Conversation history tracking
- Semantic chunking and summarization
- Memory persistence coordination (via robert-graph)

### 2. Prompt Engineering
- Dynamic prompt construction
- Template management
- Context injection and optimization
- Few-shot example selection

### 3. RAG (Retrieval-Augmented Generation)
- Query understanding and expansion
- Semantic search across knowledge base
- Context ranking and selection
- Answer synthesis with retrieved context

### 4. AI Service Integration
- Claude CLI integration (primary)
- Support for multiple AI providers
- Streaming response handling
- Error recovery and fallback strategies

### 5. Business Logic
- Workflow orchestration
- Task decomposition
- Tool use coordination
- Session management

## Key Components

### Context Manager
```rust
pub struct ContextManager {
    // Manages conversation context and memory
    // Handles context window optimization
    // Coordinates with robert-graph for persistence
}
```

### LLM Client
```rust
pub struct LlmClient {
    // Abstracts AI provider interactions
    // Handles Claude CLI communication
    // Supports streaming and batched requests
}
```

### Search Manager
```rust
pub struct SearchManager {
    // Executes semantic search queries
    // Ranks and filters results
    // Integrates with robert-graph vector store
}
```

### RAG Pipeline
```rust
pub struct RagPipeline {
    // Orchestrates full RAG loop
    // Query → Retrieve → Rank → Generate
    // Handles multi-hop reasoning
}
```

## Dependencies

### robert-graph
`robert-core` depends heavily on `robert-graph` for:
- Vector storage and retrieval
- Knowledge graph queries
- Memory persistence
- Database encryption and sync

### AI/ML Libraries
- **async-openai**: OpenAI API client (for future multi-provider support)
- **candle**: ML inference framework
- **fastembed**: Fast embeddings generation
- **tokenizers**: Tokenization for context management

## Usage

`robert-core` is not meant to be used directly. It's instantiated by `robert-server`:

```rust
use robert_core::{Context, LlmClient, SearchManager};

// Typically done by robert-server
let context_manager = ContextManager::new(config);
let llm_client = LlmClient::new(claude_config);
let search_manager = SearchManager::new(graph_store);

// Execute RAG query
let response = execute_rag_query(
    "What did we discuss about deployment?",
    &context_manager,
    &search_manager,
    &llm_client
).await?;
```

## Configuration

```rust
pub struct CoreConfig {
    // Claude CLI settings
    pub claude_binary_path: String,
    pub default_model: String,

    // RAG settings
    pub max_context_chunks: usize,
    pub similarity_threshold: f32,
    pub enable_multi_hop: bool,

    // Memory settings
    pub max_context_tokens: usize,
    pub summarization_threshold: usize,

    // Performance
    pub embedding_batch_size: usize,
    pub max_concurrent_searches: usize,
}
```

## Development

```bash
# Run tests
cargo test -p robert-core

# Run with logging
RUST_LOG=robert_core=debug cargo test

# Build
cargo build -p robert-core
```

## Project Structure

```
robert-core/
├── src/
│   ├── lib.rs              # Public API
│   ├── context.rs          # Context/memory management
│   ├── llm/
│   │   ├── mod.rs          # LLM client abstraction
│   │   └── local.rs        # Local model support
│   ├── search.rs           # Semantic search
│   ├── claude.rs           # Claude CLI integration
│   └── pruning.rs          # Context pruning strategies
├── Cargo.toml
└── README.md               # This file
```

## Future Enhancements

### Phase 1 (Current)
- [x] Basic Claude CLI integration
- [x] Simple context management
- [ ] Semantic search integration
- [ ] Basic RAG loop

### Phase 2
- [ ] Advanced prompt engineering
- [ ] Multi-hop reasoning
- [ ] Local model support (via Candle)
- [ ] Adaptive context windowing

### Phase 3
- [ ] Multi-modal support (images, PDFs)
- [ ] Tool use framework
- [ ] Agentic workflows
- [ ] Fine-tuning support

## Contributing

When modifying `robert-core`:

1. Ensure all database operations go through `robert-graph`
2. Maintain clean separation from transport layer (handled by `robert-server`)
3. Keep AI provider logic modular (support future providers)
4. Write tests for RAG pipeline components
5. Document prompt templates and strategies

## References

- [Robert Server Specification](../../docs/architecture/robert-server-specification.md)
- [RAG Overview](../../docs/architecture/rag-overview.md)
- [Architecture Decisions](../../docs/architecture/architecture-decisions.md)
