# ADR-006: Local LLM as a Hard Requirement

**Date**: 2025-12-03
**Status**: Accepted

## Context

To support the "Split-Brain" architecture and "The Sarah Test" (privacy-first context control), we cannot rely purely on cloud APIs for the internal loop.

## Decision

A **Local LLM is now a hard requirement** for Robert:
- The app must bundle or download a small model (e.g., via Candle or by managing an Ollama instance) on first launch
- Core features (Ingestion, PII stripping) will fail if local inference is unavailable

## Consequences

### Pros
- Enables true offline functionality
- Privacy guarantees for sensitive operations

### Cons
- Increases app binary size (or initial download size)
- Higher system requirements (RAM/GPU)
