# ADR 008: Selection of SurrealDB for GraphRAG

## Status
Accepted

## Date
2025-12-05

## Context
"Robert" requires a data store to implement GraphRAG (Graph Retrieval-Augmented Generation). The system needs to store both graph data (nodes, edges) and vector embeddings for semantic search.

Key constraints and requirements include:
1.  **Local-First / Embedded**: The application is primarily a Rust desktop agent. It should ideally run as a single binary without requiring users to install and manage external server processes (like Docker or separate DB daemons).
2.  **Vector & Graph support**: The solution needs to query relationships (Graph) and perform similarity search (Vector).
3.  **Rust Integration**: Strong Rust ecosystem support is preferred.
4.  **Scalability**: While starting local, the architecture should support scaling to a "Team Mode" where multiple agents might share a knowledge base, without hitting hard vertical bottlenecks.

We evaluated two primary candidates:
*   **SurrealDB**: A multi-model (Graph + Document + Vector) database written in Rust.
*   **Memgraph**: An in-memory graph database compatible with Cypher, written in C++.

## Decision
We have decided to use **SurrealDB**.

## Detailed Analysis

### 1. Deployment Model (Embedded vs. Server)
*   **SurrealDB** supports a true embedded mode (via `RocksDB` or `SurrealKV`). This allows the `robert` agent to encompass its database within the application process. This is critical for the User Experience of a local desktop tool.
*   **Memgraph** primarily operates as a client-server architecture. While high-performance, it typically requires running a separate binary or container, which complicates distribution and management for a local user.

### 2. Multi-Model Capabilities
*   **SurrealDB** natively treats data as Graph, Document, and Vector simultaneously. We can store a Node, embed it with a vector, and traverse it as a graph in the same query context.
*   **Memgraph** is a specialized Graph database. While it is extremely fast for complex algorithms (PageRank, BFS), vector search often requires additional configuration or extensions.

### 3. Scalability & Bottlenecks
*   **SurrealDB** separates compute from storage. In a shared team scenario, it can use **TiKV** as a distributed storage backend. This allows horizontal scaling of both storage and compute nodes independently. This architecture prevents "hard" bottlenecks; performance can be improved by adding hardware.
*   **Memgraph** is in-memory. While faster per-node for algorithms, it is vertically limited by the RAM of a single machine (typically). Sharding writes in Memgraph is complex compared to the seamless horizontal scaling of SurrealDB's storage layer.

### 4. Ecosystem
*   **SurrealDB** is written in Rust and provides a first-party Rust SDK that handles the embedded logic.
*   **Memgraph** provides a Rust client (`rsmgclient`) that connects to the C++ server over the network.

## Consequences
*   **Positive**:
    *   Simplified "install" experience for users (no external DB setup).
    *   Unified store for Graph and Vectors simplifies `robert-graph` code.
    *   Future-proof path to team deployment via TiKV cluster without rewriting the query layer.
*   **Negative**:
    *   SurrealDB is younger than Memgraph/Neo4j; we may encounter edge cases in complex graph algorithms (though fundamental traversals are fully supported).
    *   We are dependent on `surrealdb`'s crate stability.

## References
*   [SurrealDB Embedded](https://surrealdb.com/docs/surrealdb/integration/embedded)
*   [SurrealDB Architecture](https://surrealdb.com/docs/surrealdb/introduction/architecture)
