# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) documenting key architectural decisions made during the development of Robert.

## Active ADRs

- [ADR-001: Core Crate Structure](./adr-001-core-crate-structure.md) - Modular crate organization (robert-app, robert-server, robert-core, robert-graph)
- [ADR-002: Hybrid Graph Architecture](./adr-002-hybrid-graph-architecture.md) - SurrealDB for persistence + Petgraph for ephemeral operations
- [ADR-003: Robert Server Architecture](./adr-003-robert-server-architecture.md) - Server component with local + remote modes
- [ADR-004: Split-Brain Generation Architecture](./adr-004-split-brain-generation.md) - Local "Little Brain" + Cloud "Big Brain"
- [ADR-006: Local LLM as Hard Requirement](./adr-006-local-llm-requirement.md) - Required local model for privacy operations
- [ADR-007: Custom Rust Agent for Local Loop](./adr-007-custom-rust-agent.md) - Custom agent implementation instead of Rig framework

## Rejected ADRs

- [ADR-005: Adoption of Rig for Agentic RAG](./adr-005-rig-adoption-rejected.md) - Rejected in favor of custom implementation

## ADR Format

Each ADR follows this structure:
- **Title**: ADR-XXX: Brief title
- **Date**: When the decision was made
- **Status**: Accepted | Rejected | Superseded | Deprecated
- **Context**: The issue or problem being addressed
- **Decision**: What we decided to do
- **Consequences**: Pros and cons of the decision
