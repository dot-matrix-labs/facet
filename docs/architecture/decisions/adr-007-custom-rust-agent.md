# ADR-007: Custom Rust Agent for Local Loop

**Date**: 2025-12-03
**Status**: Accepted

## Context

We evaluated `rig-core` (ADR-005) for the local RAG loop. The evaluation revealed that adapting local Rust libraries (`candle`, `fastembed`) to Rig's REST-centric traits (`CompletionModel`, `EmbeddingModel`) requires significant boilerplate and introduces unnecessary friction.

## Decision

We will **DROP Rig** for the core local loop and implement a **Custom Rust Agent**:
- We will write a lightweight, type-safe loop directly using `candle-core` and `surrealdb`
- We will prioritize direct control over the inference loop (for optimizations like KV-caching with Phi-4) over generic abstractions

## Consequences

### Pros
- **Simplicity**: Less boilerplate, no "fighting the framework"
- **Performance**: Direct control over `candle` tensors and generation loop
- **Flexibility**: Easier to integrate bleeding-edge models (Phi-4) without waiting for upstream library support

### Cons
- We must implement our own "Tool" traits and planning logic
- This is acceptable for the scoped "Split-Brain" use case
