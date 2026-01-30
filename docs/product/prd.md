# Robert - The Memory Layer for AI
## Product Requirements Document

## Executive Summary

**Robert** is the neutral, trusted "Context OS" that sits between users and AI reasoning providers—combining **sophisticated memory control** with **agentic autonomy**. While tools like ChatGPT and Claude vertically integrate memory with reasoning, and Mem.ai offers "thought partnership" without action, Robert unbundles the AI stack AND executes workflows autonomously.

The AI market is bifurcating: **Reasoning** (OpenAI, Anthropic, Google) is becoming commoditized infrastructure, competing on price and intelligence per token. **Memory**—the proprietary, highly sensitive context that defines who the user is and what they know—remains locked in provider black boxes.

**The critical gap**: Current AI tools (ChatGPT, Mem.ai, Notion AI) are **retrieval engines, not execution engines**. Users ask questions, get answers, then manually execute workflows. Robert provides **domain-specific autonomous agents** that execute end-to-end workflows within user-defined boundaries.

Robert solves this through a **Firewalled Architecture + Agentic Execution**: Memory stays Personal (user-owned, structured, portable), while Reasoning remains Commoditized (rented, ephemeral, interchangeable). We manage the user's digital brain, structure it into a knowledge graph, and **autonomously execute workflows** using the reasoning model of their choice.

## Product Vision

**Context control + agentic execution: AI that acts on your behalf within defined domains.**

We believe AI should be:
- **Controllable**: Users curate what AI knows, not accept black-box profiles
- **Portable**: Memory travels across jobs, platforms, and providers
- **Segregated**: Personal vs. professional contexts kept separate
- **Transparent**: Users see what memories are active for any query
- **Firewalled**: Reasoning providers see anonymized context, never full user profiles
- **Platform-agnostic**: Works across Windows, macOS, Linux, iOS, Android
- **Agentic**: Executes workflows autonomously, not just retrieves information

### The Problem

**Current AI systems have fatal flaws:**

1. **The "Mystery Meat" Profile**: Users have no control over what AI "knows." A business owner cannot tell ChatGPT: "Remember my 2024 pricing, but explicitly forget the 2019 PDF for generation purposes."

2. **Platform Fragmentation**: Context is scattered across devices and platforms. Apple Intelligence only works on Apple. Copilot only works on Microsoft. No neutral layer exists.

3. **Vertical Integration Conflicts**: To get smart answers, you must give OpenAI your data. To keep secrets, you must use a dumb model. No middle ground.

4. **The "Thought Partner" Trap (Mem.ai)**: Retrieval without execution. Users ask "What should I quote for this project?" AI retrieves pricing data and gives an answer. User still manually creates the quote in a separate tool. **No workflow execution, no domain ownership, no autonomy.**

5. **The "Walled Garden" Problem (Notion AI)**: Only indexes documents within Notion. Enterprise teams use Jira, Slack, Figma, Linear, Google Docs, email, and local files. **Notion AI can't answer cross-tool questions.** No intelligent context sharding across teams, roles, or personal vs. professional boundaries.

**What users actually need:**
- Curated intelligence without black-box memory
- Context segregation (work vs. personal, current job vs. portable career knowledge)
- Data portability when changing jobs or providers
- Cross-platform AI continuity
- Privacy through structural neutrality
- **Autonomous agents that execute workflows, not just retrieve information**
- **Universal indexing across all tools** (not locked to one vendor's ecosystem)
- **Sophisticated context sharding** (team graphs, personal graphs, role-based access, cross-tool intelligence)

### Our Solution

**ContextOS: An Operating System for AI Memory**

Robert is not just a chat client; it's a Memory Layer that enables:

- **The Sarah Test**: An interior designer marks a 2019 pricing document as "outdated" without deleting it. Robert keeps it for tax records but never uses it for pricing quotes. She "fine-tuned" her AI without writing code.

- **Firewalled Architecture**: Three-party model—User (local keys, hot state) → Robert (trusted utility cloud for sync/compute) → Reasoning Provider (untrusted, sees only anonymized context).

- **GraphRAG**: Knowledge graphs instead of simple vector search. Entities, relationships, temporal reasoning. Answers questions like "How has our strategy changed since last year?"—impossible for standard RAG.

- **Reactive Pruning**: Users don't organize proactively. They fix AI mistakes at the moment of frustration. Zero-cost curation when motivated.

- **Enterprise Network Effect**: Individual users curate personal knowledge graphs. Teams sync proprietary graphs (legal precedents, client portfolios). We monetize synchronization of structured wisdom, not compute.

### Competitive Landscape

**Why competitors can't build Robert:**

| Player | Conflict | Robert's Advantage |
|--------|----------|-------------------|
| **OpenAI/Anthropic** | Business model relies on mining user data. Cannot credibly offer provider-neutral switching. | We have no conflict: protecting data *is* the business model. |
| **Apple/Microsoft/Google** | Platform lock-in is their strategy. Structurally cannot bridge ecosystems. | Platform-agnostic by design. The Dropbox lesson. |
| **Dropbox** | Most natural competitor and likely acquirer. Should build this. | Speed to market. Purpose-built, AI-native, no legacy business. |

**Robert's differentiators:**
- ✅ **ContextOS** - Sophisticated control over memory boundaries
- ✅ **Platform-agnostic** - Works everywhere, not just Apple/Microsoft ecosystems
- ✅ **Structural neutrality** - No business model conflict with data sovereignty
- ✅ **GraphRAG** - Knowledge graphs enable temporal reasoning
- ✅ **Firewalled privacy** - Anonymization proxy between user and reasoning providers

### Vision Statement

> "Just as the Browser unbundled the OS from the Web, Robert unbundles Memory from the Model—becoming the trusted gateway that ensures AI serves the user, not the other way around."

## Target Users

### Phase 1: "Vibe Coder" & "Paranoid Pro" (Months 0-9)

**Prosumers with high-end hardware (Mac Apple Silicon first):**
- **Developers**: Local-first coding assistance without sending code to GitHub Copilot
- **Designers**: 10-year portfolio curated as knowledge graph
- **Consultants**: Frameworks, proposals, case studies as portable context
- **Small business owners**: Client relationships and pricing history under control

**Key characteristics:**
- Value their time at $100-500/hour
- Sophisticated, high-income professionals
- Mixing personal and professional AI usage
- Privacy-conscious but pragmatic (not ideological)
- Mac users who expect polished, native experiences

### Phase 2: Team Infection (Months 9-18)

**Small businesses converting from individual use:**
- Law firms curating "IP Precedents" graphs
- Design agencies with shared client portfolios
- Consultancies with institutional frameworks
- Medical practices with patient care protocols

**Key characteristics:**
- 5-50 employees
- Need context segregation (work vs. personal)
- Willing to pay $50-200/user/month for team sync
- High switching costs once graph is curated

### Phase 3: Enterprise Firewall (Months 18-36)

**Compliance-driven enterprise adoption:**
- CISOs buying "Shadow AI Audit" capability
- Enterprises with role-based memory access
- Organizations needing audit trails
- Companies with regulatory compliance requirements

**Key characteristics:**
- 500+ employees
- $200-500/user/month willingness to pay
- Context governance as critical infrastructure
- Treating knowledge graphs as proprietary competitive assets

## Goals

### Primary Goals (Alpha / v0.5) - "ContextOS Core"
1. ✅ **Local knowledge base** - Users maintain ≥10 documents, cite in ≥30% of queries
2. ✅ **Context control** - Mark documents as archived, set temporal relevance, segregate contexts
3. ✅ **GraphRAG** - Knowledge graphs with entities, relationships, temporal reasoning
4. ✅ **Transparent attribution** - Context sidebar shows what memories are active
5. ✅ **Reactive pruning** - Users curate via "Mark as Outdated" at moment of failure
6. ✅ **Native macOS app** - Beautiful, polished Mac-first experience

### Primary Goals (v1.0) - "Production Ready"
1. ✅ **Multi-provider reasoning** - 40% of users actively use ≥2 providers
2. ✅ **Privacy firewall** - PII stripping, token obfuscation before cloud providers
3. ✅ **Sync & Cloud Compute** - E2E encrypted sync across Mac/iPhone/Desktop, ephemeral compute
4. ✅ **Hierarchical memory** - Hot/Warm/Cold storage (MemGPT pattern)
5. ✅ **Weekly Active Context (WAC)** - Metric tracking documents >7 days old

### Future Goals (Roadmap)
1. 🔄 **Domain agents (v1.5)** - Pricing agent, research agent, meeting prep agent, writing agent
2. 🔄 **Team knowledge graphs** - Shared memory with access control
3. 🔄 **Context namespacing** - First-class contexts with explicit boundaries
4. 🔄 **Query mixing (Tor for AI)** - Defeat timing analysis via aggregated queries
5. 🔄 **Advanced agentic workflows** - Browser/desktop control, email integration
6. 🔄 **Windows/Linux clients** - Cross-platform expansion
7. 🔄 **Mobile apps** - iOS/Android companions

## Product Scope

### In Scope (Alpha / v0.5)

#### Platform & Deployment
- ✅ **macOS desktop application** (macOS 11+, Apple Silicon optimized)
- ✅ **Local-first architecture** (Local inference or direct API calls)
- ✅ **Native .app bundle** with DMG installer
- ✅ **Headless CLI** (`robert-cli`) for server/scripted environments

#### User Interface
- ✅ **Tauri-based desktop app** with Svelte frontend
- ✅ **Chat interface** for natural language queries
- ✅ **Context sidebar** showing active memories with attribution
- ✅ **Knowledge graph browser** visualizing entities and relationships
- ✅ **Settings panel** for basic configuration
- ✅ **Native macOS integration** (menus, notifications)

#### Memory Management Features
- ✅ **GraphRAG knowledge graphs** - Entities, relationships, temporal reasoning
- ✅ **Context control** - Mark as outdated, archive, exclude from contexts
- ✅ **Transparent attribution** - See what memories AI used for each query
- ✅ **Reactive pruning** - One-click curation at moment of frustration
- ✅ **Context segregation** - Personal vs. work

### In Scope (v1.0)

#### Advanced Features
- ✅ **Multi-provider switching** - OpenAI, Anthropic, local models
- ✅ **Hierarchical memory** - Hot (context window), Warm (summaries), Cold (archive)
- ✅ **E2E encrypted sync** via zero-knowledge cloud storage
- ✅ **Privacy Firewall** - PII stripping, token obfuscation
- ✅ **Ephemeral cloud compute** - Stateless workers for embeddings
- ✅ **Audit trails** - Log what was sent to reasoning providers

### Out of Scope (v1.0)

#### Deferred to Future Versions
- ❌ **Windows/Linux desktop apps** (v2.0+)
- ❌ **Mobile apps** (iOS/Android) (v2.0+)
- ❌ **Team workspaces** (shared knowledge graphs) (v2.5+)
- ❌ **Query mixing (Tor-style)** (v3.0)
- ❌ **Agentic automation** (browser/desktop control) (v3.0)
- ❌ **Context marketplace** (community templates) (v3.5)

#### Explicitly Excluded
- ❌ Training AI models on user data
- ❌ Advertising or user profiling
- ❌ Selling user data to third parties
- ❌ Cryptocurrency/blockchain features
- ❌ Social media automation (spam prevention)

## Requirements

### Functional Requirements

#### FR1: Universal Knowledge Base Management
- **FR1.1**: Import from local files (PDF, Markdown, text, web pages)
- **FR1.2**: Connect to enterprise tools via APIs (Jira, Slack, Notion, Figma, Linear, Google Workspace)
- **FR1.3**: Index ephemeral/transient data (project tracking, conversations, design iterations)
- **FR1.4**: Auto-extract entities (people, projects, concepts) with source tool attribution
- **FR1.5**: Build unified knowledge graph across all tools with relationship preservation
- **FR1.6**: Support ≥10GB document corpus per user (cross-tool aggregation)
- **FR1.7**: Incremental indexing (add documents without full rebuild)
- **FR1.8**: Search knowledge base with semantic similarity across all sources

#### FR2: Context Control & Intelligent Sharding (ContextOS)
- **FR2.1**: Mark documents as "archived" (keep for reference, exclude from generation)
- **FR2.2**: Set temporal relevance (current vs. historical)
- **FR2.3**: Segregate contexts (personal, work, projects, teams)
- **FR2.4**: Context sidebar showing active memories with weights and source tools
- **FR2.5**: One-click pruning actions ("Mark as Outdated", "Exclude from Context")
- **FR2.6**: Audit trail of what memories were used for each query
- **FR2.7**: Team knowledge graphs (shared context across team members)
- **FR2.8**: Personal knowledge graphs (portable across jobs)
- **FR2.9**: Role-based context filtering (PM sees project graph, designer sees design nodes)
- **FR2.10**: Cross-tool intelligent routing ("This query needs Jira + Slack, not Figma")
- **FR2.11**: Context ownership rules (team-owned vs. user-owned data)

#### FR3: GraphRAG
- **FR3.1**: Entity extraction from documents
- **FR3.2**: Relationship inference between entities
- **FR3.3**: Temporal reasoning (time-aware queries)
- **FR3.4**: Graph traversal for query answering
- **FR3.5**: Hierarchical memory (Hot/Warm/Cold tiers)
- **FR3.6**: Cached traversal patterns for common queries

#### FR4: Agentic Execution (v1.5+)
- **FR4.1**: Domain definition interface (user assigns agent to workflow)
- **FR4.2**: Boundary setting (rules, approval thresholds, constraints)
- **FR4.3**: Trigger detection (email keywords, calendar events, file changes)
- **FR4.4**: Multi-step workflow execution (retrieve → process → generate → act)
- **FR4.5**: Human-in-loop approval gates (flag for review when needed)
- **FR4.6**: Workflow templates (pricing agent, research agent, meeting prep, writing)
- **FR4.7**: Execution history and audit trail (what agent did, when, why)
- **FR4.8**: Agent feedback loop (thumbs up/down, refinement suggestions)

#### FR5: Multi-Provider Reasoning
- **FR5.1**: Support OpenAI, Anthropic, local models
- **FR5.2**: One-click provider switching
- **FR5.3**: Per-query provider selection
- **FR5.4**: Cost tracking per provider
- **FR5.5**: Automatic fallback if provider fails

#### FR6: Privacy Firewall
- **FR6.1**: PII detection and stripping (names, emails, SSN, credit cards)
- **FR6.2**: Token obfuscation (replace sensitive data with tokens)
- **FR6.3**: Anonymization proxy before reasoning providers
- **FR6.4**: Audit log of data sent to cloud
- **FR6.5**: User consent for cloud inference

#### FR7: Sync & Cloud Compute
- **FR7.1**: E2E encrypted cloud storage for knowledge graphs
- **FR7.3**: Ephemeral compute for embedding generation
- **FR7.4**: Cross-device sync (Mac ↔ iPhone ↔ Desktop)
- **FR7.5**: Conflict resolution for multi-device edits



### Non-Functional Requirements

#### NFR1: Performance
- **NFR1.1**: Query response < 2 seconds for typical knowledge base
- **NFR1.2**: Graph traversal < 200ms P95 latency
- **NFR1.3**: Indexing 1000 documents < 12 hours (cloud-assisted)
- **NFR1.4**: Sync time < 5 seconds when coming online
- **NFR1.5**: Memory footprint < 500MB (app + hot state)

#### NFR2: Reliability
- **NFR2.1**: 99% uptime for local operations
- **NFR2.2**: Zero user-reported data loss
- **NFR2.3**: Graceful degradation when cloud unavailable
- **NFR2.4**: Automatic recovery from sync conflicts
- **NFR2.5**: <1% crash rate

#### NFR3: Usability
- **NFR3.1**: Non-technical users can mark documents as archived
- **NFR3.2**: Context curation requires <5 seconds per action
- **NFR3.3**: Clear error messages with recovery suggestions
- **NFR3.4**: Keyboard shortcuts for power users
- **NFR3.5**: Visual consistency with macOS design

#### NFR4: Compatibility
- **NFR4.1**: macOS 11 Big Sur and later
- **NFR4.2**: Both Intel and Apple Silicon Macs
- **NFR4.3**: Screen resolutions from 1280x800 to 5K

#### NFR5: Security & Privacy
- **NFR5.1**: E2E encryption with password-derived keys (Argon2)
- **NFR5.2**: Zero-knowledge cloud architecture
- **NFR5.3**: External security audit confirms no PII leakage
- **NFR5.4**: Reasoning providers cannot reconstruct user identity from 1000+ queries
- **NFR5.5**: No telemetry or data collection (opt-in only)
- **NFR5.6**: Secure storage of API keys (macOS Keychain)

#### NFR6: Maintainability
- **NFR6.1**: Modular architecture
- **NFR6.2**: Unit test coverage > 70%
- **NFR6.3**: Integration tests for core flows
- **NFR6.4**: Automated CI/CD pipeline
- **NFR6.5**: Documentation for developers

#### NFR7: Distribution
- **NFR7.1**: Code-signed macOS application
- **NFR7.2**: Notarized for Gatekeeper
- **NFR7.3**: DMG installer
- **NFR7.4**: Automatic update mechanism
- **NFR7.5**: Release notes for each version

## User Stories

### US1: The Sarah Test (Interior Designer)
**As a** small business owner with historical pricing data
**I want to** mark old pricing documents as "archived" without deleting them
**So that** my AI never quotes outdated rates but I keep records for taxes

**Acceptance Criteria:**
- User can click "Mark as Outdated" on document in context sidebar
- Document remains in knowledge base for historical queries
- AI excludes archived documents from pricing-related generation
- User can query "What did we charge in 2019?" and get correct historical answer

**UI Flow:**
```
AI quotes wrong price → User sees 2019 doc in context sidebar →
Clicks "Mark as Outdated" → AI never uses that doc for pricing again
```

### US2: Context Segregation (Consultant)
**As a** consultant changing jobs
**I want to** separate my portable career knowledge from employer-specific context
**So that** I take my frameworks with me but leave client IP behind

**Acceptance Criteria:**
- User can create "Personal Career" and "Client X Project" contexts
- Documents tagged to specific contexts
- When leaving job, export "Personal Career" context
- Employer-specific context stays segregated and deletable

**UI Flow:**
```
Create Contexts → Tag Documents → Export Personal → Delete Work Context
```

### US3: Multi-Provider Reasoning (Developer)
**As a** developer
**I want to** use Claude for coding and GPT-4 for writing
**So that** I get the best model for each task without switching apps

**Acceptance Criteria:**
- User can select provider per query
- Memory stays consistent across providers
- Cost tracking shows spend per provider
- One-click switching between providers

**UI Flow:**
```
Type query → Select provider dropdown → Submit → Memory consistent across both
```

### US4: Reactive Pruning (Marketer)
**As a** marketer tracking competitors
**I want to** fix AI mistakes when it uses outdated competitor data
**So that** my reports stay accurate without manual organization

**Acceptance Criteria:**
- AI gives wrong answer using old data
- User clicks document in context sidebar
- Options: "Mark as Outdated", "Exclude from Context", "Delete"
- Future queries use updated context

**UI Flow:**
```
AI error → User frustrated → Clicks offending document →
"Mark as Outdated" → Problem fixed forever
```

### US5: Cross-Device Sync (Designer)
**As a** designer
**I want to** start research on Mac and continue on iPhone
**So that** my context travels with me

**Acceptance Criteria:**
- Knowledge graph syncs across devices
- <5 second sync when coming online
- No conflicts or data loss
- Query on iPhone sees Mac's context

**UI Flow:**
```
Research on Mac → Add documents →
Open iPhone app → Context synced → Continue research
```

## Business Model

### Phase 1: The Tool (Seed/Series A)
- **Target**: Developers & Prosumers (Mac Apple Silicon first)
- **Value**: "Vibe Coding" on local repos + Context Control
- **Model**: Freemium / Pro Subscription

**Pricing:**
- **Free Tier**: Local-only, single device, 1GB knowledge base
- **Pro ($10-20/mo)**: Cloud sync, 10GB, multi-device, priority support

### Phase 2: The Network (Series B)
- **Target**: Teams (5-50 employees)
- **Value**: Shared knowledge graphs with access control
- **Revenue**: $50/user/mo for team sync

**The Moat**: When a team curates a shared knowledge graph (legal precedents, client portfolios), it becomes a proprietary asset. We charge to sync that structured wisdom.

**Analogy**: GitHub doesn't own the code; they charge for collaboration. We charge for collaboration around Memory.

### Phase 3: Enterprise (Series B+)
- **Target**: Enterprises (500+ employees)
- **Value**: Context governance, compliance, audit trails
- **Revenue**: $200-500/user/mo

**The "Wedding Photos" Justification**: People pay for Google Photos before internet bills. Why? Risk of losing wedding photos is unbearable. Extrapolate to business knowledge graphs—a designer's 10-year portfolio, a law firm's precedents. Once curated, it's irreplaceable intellectual capital. Users will pay $200-500/year to protect years of work.

### Revenue Model Progression
- **Individual (Free or $10-20/mo)**: Customer acquisition, not profit center
- **Team ($50/user/mo)**: First real revenue
- **Enterprise ($200-500/user/mo)**: High-margin, sticky (>95% retention)

**Success Metric**: Path to $100M ARR = 100K individuals (break-even/CAC) + 20K teams ($120M potential) + 100 enterprises ($120M potential)

## Go-to-Market Strategy

### Phase 1: Vibe Coder & Paranoid Pro (Months 0-9)
**Target**: 10,000 Individual Users (Free & Pro)

**Strategy**: "Infiltration via Utility"

**Launch Week Blitz (Supabase Style)**:
- **Day 1**: "Anti-Cloud" Manifesto on Hacker News / Latent Space
- **Day 2**: VS Code extension for local coding without GitHub Copilot
- **Day 3**: Obsidian/Notion bridge for "Second Brains"
- **Day 4**: GraphRAG benchmark white paper
- **Day 5**: Open source "Local-Sync" protocol

**Distribution**:
- Product Hunt (Top 5 of the day goal)
- Hacker News front page
- Mac productivity blogs
- Developer Twitter

### Phase 2: Team Infection (Months 9-18)
**Target**: 1,000 Team Customers (5-10 users each)

**Strategy**: Bottoms-up. Individual users bring Robert to teams.

**Sales Motion**:
1. User curates personal graph
2. Realizes "my team needs this"
3. Self-serve team creation
4. Viral loop: team members become users

### Phase 3: Enterprise Firewall (Months 18-36)
**Target**: 50 Enterprise Customers (500+ users each)

**Strategy**: "Shadow AI Audit" trojan horse

**Sales Motion**:
1. CISO discovers employees using ChatGPT with company data
2. Robert offers audit + compliance solution
3. Deploy Robert as "official AI gateway"
4. Enterprise pays $200-500/user/mo

## Success Metrics

### Adoption Metrics (3 months post-launch)
- 10,000 downloads
- 1,000 active users (weekly)
- 100+ GitHub stars
- 4.5+ average rating
- Featured on Product Hunt top 5

### Product Metrics
- **Weekly Active Context (WAC)**: 70% of users query documents >7 days old
- **Context Control**: 90% of users curate context (mark as outdated, archive)
- **Multi-Provider**: 40% of users actively use ≥2 reasoning providers
- **Cross-Device**: 50% of users actively use ≥2 devices

### Business Metrics
- **Individual → Team Conversion**: 10% of individual users convert to team plans
- **Enterprise Pipeline**: 5 qualified enterprise leads within 6 months
- **Retention**: >95% annual retention for teams (curated graphs are irreplaceable)

### Privacy Metrics
- **External Audit**: Confirms reasoning providers cannot reconstruct user identity
- **User Trust**: 90% of users "trust" Robert with sensitive data (survey)
- **Opt-In Cloud**: <10% initially (grows with trust to 40%)

## Competitive Analysis

### Memory/Context Platforms

| Feature | **Robert (v1.0)** | **Notion AI** | **Obsidian + Copilot** | **Mem.ai** | **ChatGPT Memory** |
|---------|------------------|--------------|----------------------|-----------|------------------|
| **Context Control** | ✅ ContextOS | ❌ Black box | ⚠️ Manual | ❌ Automatic | ❌ Black box |
| **Multi-Provider** | ✅ OpenAI/Anthropic/Local | ❌ Locked to OpenAI | ❌ GitHub only | ❌ Proprietary | ❌ OpenAI only |
| **Platform-Agnostic** | ✅ Cross-platform | ⚠️ Web + mobile | ⚠️ Desktop only | ⚠️ Web + mobile | ❌ OpenAI ecosystem |
| **Universal Indexing** | ✅ Jira/Slack/Figma/Local/Email/All | ❌ **Notion only** | ⚠️ Local files only | ⚠️ Limited integrations | ❌ ChatGPT conversations only |
| **Context Sharding** | ✅ Team/Personal/Role-based/Cross-tool | ❌ **Flat workspace** | ❌ No teams | ❌ Single user | ❌ Single user |
| **GraphRAG** | ✅ Knowledge graphs | ❌ Vector search | ❌ Full-text | ⚠️ Basic graph | ❌ Vector |
| **Privacy Firewall** | ✅ Anonymization | ❌ Data sent | ❌ Data sent | ❌ Data sent | ❌ Data sent |
| **Local-First** | ✅ Hot state local | ❌ Cloud-only | ⚠️ Local files | ❌ Cloud-only | ❌ Cloud-only |
| **Agentic Execution** | ✅ Domain agents (v1.5) | ❌ Retrieval only | ❌ Retrieval only | ❌ Retrieval only | ❌ Retrieval only |
| **Enterprise** | ✅ Team graphs | ✅ Workspace | ❌ No teams | ✅ Business | ❌ Plus only |

**Unique Selling Points:**
1. **ContextOS** - Sophisticated control reasoning providers can't build
2. **Universal indexing** - Index across ANY tool (Jira, Slack, Figma, local files), not locked to one vendor
3. **Intelligent context sharding** - Team graphs, personal graphs, role-based access, cross-tool intelligence
4. **Structural neutrality** - No business model conflict with data sovereignty
5. **GraphRAG** - Temporal reasoning ("How has strategy changed?")
6. **Platform-agnostic** - The Dropbox lesson applied to AI
7. **Firewalled privacy** - Anonymization proxy between user and providers
8. **Agentic execution** - Domain-specific agents that execute workflows, not just retrieve

## Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Chrome/Gemini Nano threat** | Medium | High | Embrace it. Casual users stay on Chrome, prosumers upgrade to Robert for ContextOS. |
| **Dropbox builds this** | High | Medium | Speed to market. We're purpose-built, they must pivot organization. Acquisition target. |
| **Users won't curate** | High | Medium | Reactive pruning at moment of frustration (Sarah test). Zero-cost curation when motivated. |
| **Local AI won't scale** | High | Low | Cloud-assisted compute for heavy tasks. Ephemeral containers, not data retention. |
| **Enterprise sales cycle** | Medium | High | Bottoms-up via individuals. Team infection before enterprise sale. |
| **Privacy breach** | Critical | Low | External audit, E2E encryption, zero-knowledge architecture. |

## Technical Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                   Tauri Desktop App                     │
│                                                          │
│  ┌────────────────────┐      ┌────────────────────┐    │
│  │  Svelte Frontend   │ IPC  │   Rust Backend     │    │
│  │                    │◄────►│                    │    │
│  │  - Chat Interface  │      │  - GraphRAG        │    │
│  │  - Context Sidebar │      │  - Entity Extract  │    │
│  │  - Graph Browser   │      │  - Anonymization   │    │
│  │  - Settings Panel  │      │  - Multi-Provider  │    │
│  │                    │      │  - E2E Sync        │    │
│  └────────────────────┘      └──────────┬─────────┘    │
│                                          │              │
└──────────────────────────────────────────┼──────────────┘
                                           │
                              ┌────────────┴──────────────┐
                              │                           │
                        Local Memory             Trusted Proxy
                              │                           │
                    ┌─────────▼──────────┐    ┌──────────▼─────────┐
                    │ Hot State (Device) │    │ Robert Cloud       │
                    │ - Encryption keys  │    │ - E2E sync         │
                    │ - Active context   │    │ - Ephemeral compute│
                    │ - Graph (hot tier) │    │ - Anonymization    │
                    └─────────┬──────────┘    └──────────┬─────────┘
                              │                          │
                              │                          │
                              └──────────┬───────────────┘
                                         │
                              ┌──────────▼─────────┐
                              │ Reasoning Providers│
                              │ - OpenAI           │
                              │ - Anthropic        │
                              │ - Local models     │
                              │ (Untrusted)        │
                              └────────────────────┘
```

## Product Roadmap

### Version 0.5 (Alpha) (Q4 2025) - **ContextOS Core**
- ✅ GraphRAG knowledge graphs
- ✅ Context control (mark as outdated, segregate contexts)
- ✅ Transparent attribution (context sidebar)
- ✅ Reactive pruning (one-click curation)
- ✅ Native macOS app (Apple Silicon optimized)
- ✅ Basic reasoning (Single provider or Local)

### Version 1.0 (Q1 2026) - **Production Ready**
- ✅ Multi-provider reasoning (OpenAI, Anthropic, local)
- ✅ E2E encrypted sync (cross-device)
- ✅ Privacy Firewall (PII stripping)
- ✅ Ephemeral cloud compute
- ✅ Hierarchical memory optimization

### Version 1.5 (Q2-Q3 2026) - **Polish & Scale**
- 🔄 Team knowledge graphs (shared memory)
- 🔄 Context namespacing (first-class contexts)
- 🔄 Enhanced graph visualization
- 🔄 Performance optimizations
- 🔄 Mac-specific integrations (Shortcuts, Automator)

### Version 2.0 (Q4 2026) - **Cross-Platform**
- 🔄 Windows desktop app
- 🔄 Linux desktop app
- 🔄 iOS companion app
- 🔄 Android companion app
- 🔄 Enterprise features (SSO, audit trails, compliance)

### Version 2.5 (Q1 2027) - **Enterprise**
- 🔄 Role-based memory access
- 🔄 Advanced audit trails
- 🔄 Integration with corporate systems
- 🔄 Query mixing (Tor for AI)

### Version 3.0 (Q2 2027+) - **AI & Collaboration**
- 🔄 Agentic workflows (multi-step automation)
- 🔄 Context marketplace (community templates)
- 🔄 Advanced error recovery
- 🔄 Workflow learning system

---

## Appendix

### Technology Stack Summary
- **Desktop Framework**: Tauri 2.0
- **Frontend**: Svelte + TypeScript + Tailwind CSS
- **Backend**: Rust 1.70+
- **Knowledge Graphs**: Neo4j / Memgraph (GraphRAG)
- **Vector Database**: Qdrant / Milvus
- **Encryption**: Argon2 (key derivation), AES-256-GCM (E2E sync)
- **Sync**: CRDT-based (electric-sql pattern)
- **Async Runtime**: tokio
- **Build System**: Cargo + Vite

### Reference Links
- Tauri: https://v2.tauri.app/
- GraphRAG Research: https://www.microsoft.com/en-us/research/project/graphrag/
- MemGPT (Hierarchical Memory): https://memgpt.ai/
- Neo4j: https://neo4j.com/
- Qdrant: https://qdrant.tech/

### Glossary
- **ContextOS**: Operating System for AI Memory—sophisticated control over what AI knows
- **GraphRAG**: Graph Retrieval-Augmented Generation using knowledge graphs
- **Firewalled Architecture**: Memory (Personal) decoupled from Reasoning (Commoditized)
- **Reactive Pruning**: Curation at moment of failure, not proactive organization
- **WAC (Weekly Active Context)**: Metric tracking users querying documents >7 days old
- **Zero-Knowledge Sync**: E2EE cloud storage where Robert holds no keys
- **Ephemeral Compute**: Stateless cloud workers for heavy tasks, data destroyed after processing
- **Anonymization Proxy**: PII stripping + token obfuscation before reasoning providers

---

**Document Version**: 3.0
**Last Updated**: 2025-12-02
**Status**: Approved for Development
**Target Release**: v1.0 - Q1 2026

---

## Cuts

The following content represents earlier product directions that have been superseded by the ContextOS / Memory Layer vision. Preserved for historical reference.

### Deprecated: Browser Automation Product Vision

*The original PRD positioned Robert as a browser automation tool focused on visual feedback and local-first execution. This direction has been replaced by the ContextOS / Memory Layer for AI positioning.*

**Original Executive Summary** (Deprecated):
> Robert is an open-source, local-first browser automation tool that brings the power of automation to everyone—not just programmers. While tools like Zapier and IFTTT require API knowledge, Claude agents are complex to create, and GPT's interface provides no visibility, Robert lets users **watch automation happen in real-time**, learn by observation, and maintain full control with the ability to abort operations at any moment.

**Browser Automation Features** (Deprecated - may be revived in v3.0 as agentic workflows):
- Chrome automation via CDP
- Visual script builder
- Markdown-based automation scripts
- Screenshot and text capture
- Browser profile management
- Ephemeral sessions

**User Profiles and Multi-User Support** (Deprecated):
- Password-protected user profiles
- Browser profile isolation
- Command system with generative UI
- AI-assisted command refinement

**Chat-Driven AI Workflow System** (Deprecated):
- Injected chat interface on web pages
- AI-generated CDP scripts
- Template-based prompts
- Feedback loop for continuous improvement

**Workflow Learning System** (Deprecated - future feature):
- Workflow graph (`.workflow.md`)
- Step frame format (`.frames.json`)
- Confidence-based navigation
- Self-improving system

*Note: The browser automation capabilities remain technically viable and may be reintroduced as part of agentic workflows in v3.0+, but are no longer the primary product focus.*
