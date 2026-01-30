# ADR-002: Hybrid Graph Architecture (SurrealDB + Petgraph)

**Date**: 2025-12-02
**Status**: Accepted
**Updated**: 2025-12-05

## Context

We evaluated using `petgraph` (in-memory) vs `surrealdb` (persistent) for the GraphRAG engine. The core Alpha use case ("The Sarah Test") relies on **Local Search** (filtering edges by metadata, e.g., "exclude archived") rather than **Global Search** (community detection/clustering).

Initial decision was to use SurrealDB exclusively, but implementation revealed the need for both.

## Decision

We will use a **hybrid approach**:
- **SurrealDB** as the primary persistent graph store for long-term storage and metadata-based queries
- **Petgraph** for ephemeral in-memory graph operations and algorithms when needed
- SurrealDB remains the source of truth; petgraph is used for transient computations

## Consequences

### Pros
- Persistent storage with SurrealDB for reliability
- Access to graph algorithms via petgraph when needed
- Flexibility for both local search and global graph computations
- Can optimize specific operations by choosing the right tool

### Cons
- Slightly more complex than single-tool approach
- Need to manage data transfer between SurrealDB and petgraph when necessary
- Two graph dependencies to maintain
