# ADR-001: Core Crate Structure

**Date**: 2025-12-02
**Status**: Accepted

## Context

We are pivoting from a browser automation tool to a "Memory Layer" (ContextOS). We need a modular architecture to support GraphRAG, Context Management, and the Tauri App.

## Decision

We will split the backend into focused crates:

1. `facet-graph`: Dedicated to GraphRAG implementation (Vector store + Graph store)
2. `facet-core`: Contains the business logic, ContextOS state management, and ingestion pipelines
3. `facet-webdriver`: Retained for future agentic capabilities (browser control)
4. `facet-app`: The Tauri application shell (UI + IPC layer)
5. `facet-server`: REST API server that manages facet-core instances

## Consequences

### Pros
- Clear separation of concerns
- `facet-graph` can be tested in isolation (TDD)
- `facet-core` orchestrates logic without being tied to UI
- Clean API boundary between components

### Cons
- Slightly more boilerplate in `Cargo.toml` workspace management
