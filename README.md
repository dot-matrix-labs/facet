# Facet: The Memory Layer for AI

> **Just as the Browser unbundled the OS from the Web, Facet unbundles Memory from the Model—becoming the trusted gateway that ensures AI serves the user, not the other way around.**

Facet is the neutral, trusted **"Context OS"** that sits between users and AI reasoning providers—combining sophisticated memory control with platform-agnostic access. While tools like ChatGPT and Claude vertically integrate memory with reasoning, Facet separates them: your memory stays personal, portable, and under your control. The reasoning remains commoditized, ephemeral, and interchangeable.

## The Problem

Current AI systems have fatal flaws:

1. **The "Mystery Meat" Profile**: Users have no control over what AI "knows." A business owner cannot tell ChatGPT: "Remember my 2024 pricing, but explicitly forget the 2019 PDF for generation purposes."

2. **Platform Fragmentation**: Context is scattered across devices and platforms. Apple Intelligence only works on Apple. Copilot only works on Microsoft. No neutral layer exists.

3. **Vertical Integration Conflicts**: To get smart answers, you must give OpenAI your data. To keep secrets, you must use a dumb model. No middle ground.

4. **No Context Control**: When you change jobs, what context is yours vs. your employer's? No way to segregate personal vs. professional contexts or prevent accidental data leakage.

## The Solution: ContextOS

Facet is an **Operating System for AI Memory** that enables:

### The Sarah Test

An interior designer marks a 2019 pricing document as "outdated" without deleting it. Facet keeps it for tax records but never uses it for pricing quotes. She "fine-tuned" her AI without writing code.

**How it works:**
- **Transparent Attribution**: See exactly which documents the AI used for each answer
- **Reactive Pruning**: Click "Mark as Outdated" when the AI makes a mistake—zero-cost curation at the moment of frustration
- **Sophisticated Control**: Documents can be active (use for generation) vs. archived (keep for reference)
- **Context Segregation**: Separate personal vs. professional contexts with explicit boundaries

### Firewalled Architecture

Three-party model for privacy without sacrificing capability:

1. **User (Local)**: Encryption keys, hot state, final control
2. **Facet (Trusted Utility Cloud)**: Sync, heavy compute, anonymization—structurally aligned with protecting your data
3. **Reasoning Provider (Commodity)**: OpenAI, Anthropic, local models—sees only anonymized context, never your full profile

### GraphRAG: Knowledge Graphs, Not Just Search

Standard RAG is a bag-of-words search. Facet builds **knowledge graphs**:

- **Entities, not Keywords**: Map `Project Alpha → owned_by_Client_X → status_Active`
- **Temporal Reasoning**: Answer "How has our strategy changed since last year?"
- **Hierarchical Memory**: Hot/Warm/Cold tiers that mimic human memory
- **Relationship Inference**: Understand document evolution and dependencies

## Architecture

Facet is built as a modular Rust application:

```
┌──────────────────────────────────────────────────────────────┐
│                   Firewalled Architecture                     │
└──────────────────────────────────────────────────────────────┘

User Device (Local)                  Facet Cloud (Trusted)
┌─────────────────────┐             ┌─────────────────────┐
│   facet-app        │             │  Sync & Backup      │
│   (Tauri/Svelte)    │◄───────────►│  (E2E Encrypted)    │
│                     │             │                     │
│  • Chat Interface   │             │  • Zero-Knowledge   │
│  • Context Sidebar  │             │  • Ephemeral Compute│
│  • Graph Browser    │             │  • PII Stripping    │
└──────────┬──────────┘             └──────────┬──────────┘
           │                                   │
           │                                   │
     ┌─────▼──────────┐                       │
     │  facet-core   │                       │
     │                │                       │
     │  • GraphRAG    │                       │
     │  • Memory Mgmt │                       │
     │  • Anonymizer  │                       │
     └────────┬───────┘                       │
              │                               │
              │                               │
     ┌────────▼────────┐                      │
     │  facet-graph   │                      │
     │  (SurrealDB)    │                      │
     │                 │                      │
     │  • Vector Store │                      │
     │  • Entities     │                      │
     │  • Relationships│                      │
     └─────────────────┘                      │
                                              │
                                              ▼
                              Reasoning Providers (Untrusted)
                              ┌──────────────────────┐
                              │  • OpenAI            │
                              │  • Anthropic         │
                              │  • Local Models      │
                              │  (sees only anonymized│
                              │   context, no keys)  │
                              └──────────────────────┘
```

## Components

### Core Crates

- **[facet-app](./crates/facet-app)** - Desktop application (Tauri + Svelte)
  - Modern desktop UI for macOS, Windows, Linux
  - Context sidebar with transparent attribution
  - Graph visualization and exploration
  - Spawns local facet-server or connects to remote

- **[facet-server](./crates/facet-server)** - REST API Server (Warp + Tokio)
  - Provides REST API for clients
  - Manages authentication and sessions
  - Instantiates facet-core for request handling
  - Supports local and remote deployment modes

- **[facet-core](./crates/facet-core)** - AI/RAG Engine
  - GraphRAG implementation
  - Hierarchical memory (Hot/Warm/Cold)
  - Context control and boundary management
  - Multi-provider orchestration

- **[facet-graph](./crates/facet-graph)** - Database Layer (SurrealDB)
  - Knowledge graph storage
  - Vector embeddings for semantic search
  - Entity and relationship management
  - E2E encryption at rest

- **[facet-cli](./crates/facet-cli)** - Command Line Tool
  - Document ingestion
  - Query interface
  - Server management (start/stop/status)

- **[types](./crates/types)** - Shared Types
  - User profiles and authentication
  - Cryptography utilities
  - Serialization/deserialization

## Quick Start

### Prerequisites

- **Rust** 1.75+ (for building from source)
- **Node.js** 18+ (for facet-app frontend)
- **Chrome/Chromium** (auto-downloads on first use)

### Build and Run

```bash
# Clone repository
git clone https://github.com/yourusername/facet.git
cd facet

# Build all components
cargo build --workspace

# Run desktop app
cd crates/facet-app
npm install
npm run tauri dev
```

### Using the CLI

```bash
# Build CLI
cargo build --bin facet-cli

# Ingest documents
facet-cli ingest ~/Documents/notes.md

# Query your knowledge base
facet-cli query "What are the key points from my notes?"
```

## Key Features

### Current (v0.5 Alpha)
- ✅ Desktop application (Tauri)
- ✅ Local knowledge base management
- ✅ User profiles and authentication
- ✅ Encrypted local storage
- ✅ Basic GraphRAG implementation

### In Progress (v1.0)
- 🚧 Multi-provider reasoning (OpenAI, Anthropic, local models)
- 🚧 Privacy firewall (PII stripping, anonymization)
- 🚧 E2E encrypted sync
- 🚧 Hierarchical memory optimization
- 🚧 Context control UI

### Planned (v1.5+)
- 📋 Team knowledge graphs (shared memory with access control)
- 📋 Context namespacing (first-class personal/work separation)
- 📋 Domain agents (pricing agent, research agent, meeting prep)
- 📋 Advanced agentic workflows
- 📋 Cross-platform expansion (Windows, Linux, iOS, Android)

## Why Facet Wins

### vs. ChatGPT/Claude Desktop
**Conflict of Interest**: OpenAI's business model depends on user profiling. They cannot credibly offer provider-neutral switching or true data sovereignty.

**Facet's Advantage**: Protecting user data *is* our business model. We have no conflict—we charge users to protect their context from reasoning providers.

### vs. Apple Intelligence/Microsoft Copilot
**Platform Lock-In**: Apple Intelligence works beautifully—but only on Apple devices. Users live in heterogeneous environments (work Windows + personal Mac + iPhone).

**Facet's Advantage**: Platform-agnostic by design. The Dropbox lesson: cross-platform interoperability matters. We work everywhere.

### vs. Notion AI
**Walled Garden**: Only indexes Notion documents. Enterprise users have data in Jira, Slack, Figma, Linear, Google Docs, email, local files.

**Facet's Advantage**: Universal indexing across *any* tool. Intelligent context sharding across teams, roles, and boundaries.

### vs. Mem.ai
**Retrieval, Not Action**: "Thought partner" = better search engine. Users still manually execute workflows.

**Facet's Advantage**: Domain-specific agents that execute end-to-end workflows. We act, not just retrieve.

## Documentation

- **[Product Vision](./docs/product/prd.md)** - Full product requirements
- **[The Hypothesis](./docs/product/hypothesis.md)** - Why separating memory from reasoning matters
- **[Architecture Overview](./docs/architecture/architecture-decisions.md)** - Technical architecture and ADRs
- **[RAG Implementation](./docs/architecture/rag-overview.md)** - How GraphRAG works
- **[Build Instructions](./docs/development/build-instructions.md)** - Build and development guide

## Project Structure

```
facet/
├── crates/
│   ├── facet-app/          # Desktop application
│   ├── facet-core/         # Core AI/RAG engine
│   ├── facet-graph/        # Database layer
│   ├── facet-cli/          # CLI tool
│   └── types/               # Shared types
├── docs/
│   ├── product/             # Product strategy and vision
│   ├── architecture/        # Technical architecture
│   ├── development/         # Build and dev guides
│   └── features/            # Feature documentation
└── README.md                # This file
```

## Development

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p facet-core

# Integration tests
cargo test --workspace --test integration_tests
```

### Code Quality

```bash
# Format code
cargo fmt --workspace

# Lint
cargo clippy --workspace

# Type check
cargo check --workspace
```

## Deployment Modes

### Mode 1: Local Desktop (Default)
Perfect for individual users—complete privacy, data never leaves your machine:

```
facet-app (Desktop)
  └─► Local facet-core instance
      └─► Local facet-graph database (~/.facet/data)
```

### Mode 2: Cloud Sync (Teams/Mobile)
E2E encrypted sync across devices with ephemeral cloud compute:

```
facet-app (Mac) ◄──► Facet Cloud (E2EE) ◄──► facet-app (iPhone)
         │                   │                         │
         └───────────────────┴─────────────────────────┘
                             │
                    Shared encrypted graph
```

### Mode 3: Enterprise (Self-Hosted)
Deploy on-premises or in your cloud for full control:

```bash
# Docker deployment
docker run -p 8443:8443 \
  -v /data/facet:/data \
  -e DATABASE_PATH=/data \
  facet-server
```

## Business Model

We are not building a $20/month tool. We are building **Git for Corporate Intelligence.**

### Revenue Progression

1. **Individual ($10-20/mo)**: Customer acquisition, break-even
   - Local memory + multi-provider access
   - Free tier available (local-only, single device)

2. **Team ($50/user/mo)**: First real revenue
   - Shared knowledge graphs
   - Context segregation (work vs. personal)
   - Cross-device sync

3. **Enterprise ($200-500/user/mo)**: High-margin, sticky
   - Advanced context governance
   - Role-based memory access
   - Audit trails and compliance
   - Team knowledge graphs as proprietary assets

### The "Wedding Photos" Justification

People pay for Google Photos before their internet bill. Why? The risk of losing wedding photos is unbearable.

Now extrapolate to **business knowledge graphs**:
- A designer's 10-year portfolio
- A law firm's institutional precedents
- A consultant's frameworks and case studies

Once curated, it's irreplaceable intellectual capital. Users will pay $200-500/year to protect years of work. The switching cost is existential—losing your knowledge graph means losing professional memory.

## The Stakes

**If Facet succeeds**, it becomes one of the 3-5 products that define how humanity interfaces with AI—the trusted gateway that:
- Keeps hyperscale providers in check (one-click provider switching)
- Breaks the business model conflict (protecting data *is* our model)
- Solves the interoperability problem (works everywhere)
- Preserves user agency (you own your memory)

**The alternative**: Fragmented AI experiences siloed across corporate platforms, forced ecosystem choices, no portability, surveillance-driven personalization as the only option.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Run clippy: `cargo clippy --workspace`
6. Format code: `cargo fmt --workspace`
7. Submit a pull request

## License

MIT License - see [LICENSE](./LICENSE) for details.

## Acknowledgments

Built with:
- [Tauri](https://tauri.app/) - Cross-platform desktop framework
- [SurrealDB](https://surrealdb.com/) - Graph database
- [FastEmbed](https://github.com/Anush008/fastembed-rs) - Vector embeddings
- Rust, Svelte, and the amazing open source ecosystem

---

**The work ahead is technical, strategic, and urgent.**

For investors, partners, or contributors: hello@facet.ai
