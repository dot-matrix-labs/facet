# Implementation Plan - Robert Alpha (v0.5)

## Goal
Deliver "ContextOS Core" with a **Split-Brain Generation Architecture**:
1.  **Local Memory**: Partitioned GraphRAG (Personal/Work) running on SurrealDB.
2.  **Local Reasoning**: Small LLM (Phi-4/Gemma 2 via Candle/Ollama) for synthesis, PII redaction, and prompt engineering.
3.  **Cloud Reasoning**: Heavy lifting by OpenAI/Anthropic, fed by sanitized, optimized local prompts.

## User Review Required
> [!IMPORTANT]
> **Architecture Shift**: This plan solidifies the "Split-Brain" approach. Local compute is now a *requirement* for privacy and prompt optimization, not just an optional feature.
> **Dependency**: We are introducing `rig-core` (Rust "LangChain") to manage the Agentic RAG loop.

## Proposed Changes

### Phase 1: Foundation & Project Setup
#### [MODIFY] [Project Structure](file:///Users/lucas/code/rust/robert/Cargo.toml)
- [x] Initialize Tauri 2.0 workspace (`robert-app`)
- [x] Create `robert-graph` (GraphRAG engine)
- [x] Create `robert-core` (Business logic)
- [x] Restore `robert-cli` for headless management (downloads, memory, inference).
- [ ] **Dependency Update**: Add `rig-core` to `robert-core` for agent orchestration.

### Phase 2: The Memory Layer (Partitioned Graph)
#### [MODIFY] [GraphRAG Engine](file:///Users/lucas/code/rust/robert/crates/robert-graph/src/lib.rs)
- [x] **SurrealStore**: Embedded SurrealDB implementation.
- [ ] **Partitioning**: Update schema to support `graph_id` (Personal, Professional, Business).
- [ ] **Ingestion**: Ensure chunks are tagged with the correct partition.

#### [NEW] [File Ingestion](file:///Users/lucas/code/rust/robert/crates/robert-core/src/ingest.rs)
- [ ] File watcher for local folders.
- [ ] Parsers (PDF, MD, TXT).
- [ ] **Local Entity Extraction**: Use Local LLM to extract entities/relations during ingestion.

### Phase 3: Local Intelligence (The "Little Brain")
#### [NEW] [Local Inference](file:///Users/lucas/code/rust/robert/crates/robert-core/src/llm/local.rs)
- [ ] **Engine**: Integrate `candle` (or `ollama-rs` client) for running local models.
- [ ] **Model Strategy**: Default to `Phi-4` or `Gemma 2` (quantized).
- [ ] **Capabilities**:
    - [ ] `synthesize(chunks) -> summary`
    - [ ] `extract_pii(text) -> (redacted_text, pii_map)`
    - [ ] `optimize_prompt(query, context) -> compressed_prompt`

### Phase 4: Agentic RAG Loop (The "Bridge")
#### [NEW] [Agent Orchestration](file:///Users/lucas/code/rust/robert/crates/robert-core/src/agent.rs)
- [ ] **Custom Agent**: Implement a lightweight `Agent` struct.
- [ ] **Tools**:
    - [ ] `search_graph(query, partition)`: Vector + Traversal search.
    - [ ] `read_file(path)`: Direct file access.
- [ ] **Loop Logic**:
    1.  **Plan**: Local LLM analyzes query -> decides which partitions to search.
    2.  **Retrieve**: Agent executes searches/traversals.
    3.  **Synthesize**: Local LLM summarizes findings & redacts PII.
    4.  **Generate**: Send sanitized context + query to Cloud LLM (if complex) or answer locally (if simple).

### Phase 5: ContextOS Features
#### [MODIFY] [Context Control](file:///Users/lucas/code/rust/robert/crates/robert-core/src/context.rs)
- [x] Context Manager.
- [ ] **Reactive Pruning**: Connect UI "Mark as Outdated" to graph edge weights.

## Verification Plan

### Automated Tests
- **Unit Tests**:
    - `robert-graph`: Verify partitioning (searching "Personal" doesn't return "Work" nodes).
    - `robert-core`: Verify PII redaction (input with fake SSN -> output redacted).
- **Integration Tests**:
    - `ingest_pipeline`: Ingest file -> Check SurrealDB for nodes -> Check entities extracted.

### Manual Verification
- **The "Split-Brain" Test**:
    1.  Ingest a "Secret Project" doc into "Work" partition.
    2.  Ask a query in "Personal" context -> Should NOT find it.
    3.  Ask a query in "Work" context -> Should find it.
    4.  Verify logs: Ensure "Secret Project" details were PII-redacted before hitting OpenAI (if cloud used).
