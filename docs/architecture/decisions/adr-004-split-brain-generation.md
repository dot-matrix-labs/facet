# ADR-004: Split-Brain Generation Architecture

**Date**: 2025-12-03
**Status**: Accepted

## Context

We need to balance privacy (PII protection), cost (token usage), and intelligence (reasoning capability). Sending all raw retrieved chunks to a cloud LLM is expensive and risky. Relying solely on a local LLM limits reasoning quality.

## Decision

We will adopt a **Split-Brain Architecture**:

1. **Local "Little Brain"**: A small, efficient local model (Phi-4/Gemma 2) handles:
   - Memory Access
   - Synthesis
   - PII Redaction
   - Prompt Engineering

2. **Cloud "Big Brain"**: A large remote model (GPT-4o/Claude 3.5) handles:
   - Heavy Reasoning
   - Final Generation
   - Receives only sanitized and optimized prompts

## Consequences

### Pros
- Maximizes privacy (PII stays local)
- Minimizes cost (compressed prompts)
- Maximizes quality (cloud reasoning)

### Cons
- Requires local hardware acceleration (Metal on Mac)
- Adds complexity to the RAG loop
