# ADR-001: Core Crate Structure

**Date**: 2025-12-02
**Status**: Accepted

## Context

We are pivoting from a browser automation tool to a "Memory Layer" (ContextOS). We need a modular architecture to support GraphRAG, Context Management, and the Tauri App.

## Decision

We will split the backend into focused crates:

1. `robert-graph`: Dedicated to GraphRAG implementation (Vector store + Graph store)
2. `robert-core`: Contains the business logic, ContextOS state management, and ingestion pipelines
3. `robert-webdriver`: Retained for future agentic capabilities (browser control)
4. `robert-app`: The Tauri application shell (UI + IPC layer)
5. `robert-server`: REST API server that manages robert-core instances

## Consequences

### Pros
- Clear separation of concerns
- `robert-graph` can be tested in isolation (TDD)
- `robert-core` orchestrates logic without being tied to UI
- Clean API boundary between components

### Cons
- Slightly more boilerplate in `Cargo.toml` workspace management
