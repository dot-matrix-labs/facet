# ADR-005: Adoption of Rig for Agentic RAG

**Date**: 2025-12-03
**Status**: Rejected (Superseded by ADR-007)

## Context

Building an Agentic RAG loop (planning, tool use, multi-step retrieval) from scratch is error-prone. We need a robust abstraction layer for LLM interactions and agent orchestration in Rust.

## Decision

We will use **`rig-core`** (the "LangChain for Rust") as our agent framework:
- It provides unified APIs for LLM providers (OpenAI, Cohere, Local)
- It has built-in support for RAG primitives and vector stores
- It is type-safe and Rust-native

## Consequences

### Pros
- Accelerates development
- Type safety reduces runtime errors
- Unified interface for switching models

### Cons
- New dependency to learn and manage

## Update (2025-12-03)

**Rejected** after evaluation. Rig's traits are overly restrictive for local-only loops using `candle` and `fastembed`, requiring excessive boilerplate.

See ADR-007 for the adopted approach.
