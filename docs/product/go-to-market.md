# Robert: Go-To-Market Strategy

**This is a Series A-ready Go-To-Market (GTM) Strategy.**

Most seed-stage founders pitch "everyone." We pitch a **"Trojan Horse"** strategy: Start as a **developer utility** (bottom-up), evolve into a **team productivity tool** (mid-market), and land as **enterprise compliance infrastructure** (top-down).

This is the GTM playbook tailored to Robert's "Firewalled Architecture."

---

## The Core Metrics

**Weekly Active Context (WAC)**: The number of users who query a document >7 days old. Proof of retention and real value delivery.

**North Star**: $2M ARR by Series A
- 10K Pro Users ($10-20/mo) = $1.2-2.4M ARR
- 50 Enterprise Pilots ($50K+ ACV) = $2.5M ARR
- **Target**: $2-5M ARR with proven unit economics

---

## Phase 1: The "Vibe Coder" & The "Paranoid Pro" (Months 0-9)

**Target**: 10,000 Individual Users (Free & Pro)

**Strategy**: "Infiltration via Utility." We don't sell 'Privacy' (boring); we sell 'Super-Powered Local Dev.'

**Why Developers First?**
- Highest context control pain (mixing personal projects + work code)
- Early adopters who influence team decisions
- Willingness to pay for productivity tools ($20/mo without blinking)
- Natural evangelists (tweet about tools they love)
- Mac Apple Silicon users (our initial rollout target)

### 1. The "Launch Week" Blitz (Supabase Style)

Instead of a single Product Hunt launch, execute a **5-day "Feature Drop" week** targeting the Local-First / LoFi community.

**Day 1: The "Anti-Cloud" Manifesto**
- Publish technical deep dive on Hacker News / Latent Space
- Title: "Why We Built a Firewall for OpenAI"
- Attack the "Mystery Meat" RAG problem
- Show concrete Sarah example: outdated pricing killing trust
- Technical credibility: GraphRAG architecture, zero-knowledge sync
- **Goal**: Front page of HN, 500+ upvotes

**Day 2: The "Vibe Coding" Integration**
- Release VS Code extension
- Feature: Query your local repo + local docs *without* sending code to GitHub Copilot
- Value prop: "Stop leaking your startup's IP to Microsoft"
- Demo video: Developer using Robert for codebase search across 3 repos
- **Goal**: 1,000 installs in 24 hours

**Day 3: The "Obsidian/Notion" Bridge**
- One-click import for "Second Brains"
- Target: Productivity nerds and PKM (Personal Knowledge Management) enthusiasts
- Feature: Import 5 years of Obsidian notes, automatically build knowledge graph
- **Goal**: 50 tweets from #PKM community

**Day 4: The "GraphRAG" Benchmark**
- Publish white paper: "Robert beats standard RAG by 30% on complex queries"
- Methodology: Blind A/B testing with 100 real-world questions
- Open source the benchmark suite (builds credibility)
- **Goal**: Shared by AI researchers, cited in papers

**Day 5: Open Source "Local-Sync" Protocol**
- Release small, open-source library for encrypted sync
- Position as public good for the community
- Builds developer goodwill and technical credibility
- **Goal**: 500+ GitHub stars, contribution from outside developers

### 2. Influencer Targeting (The "Braintrust")

Don't pay for ads. Get specific communities to adopt you.

**Newsletter Sponsorships**:
- *Latent Space* (swyx/Alessio) - AI engineers who care about architecture
- *AlphaSignal* - Technical AI audience
- *The Rundown AI* - Broader AI-interested professionals
- *TLDR* - Developer audience
- Budget: $5K/newsletter, 3-4 placements = $15-20K

**Twitter/X Tribes**:
Target the "Local-First" (#LoFi) movement:
- **Simon Willison** (LLM security expert, SQLite creator)
- **Geoffrey Litt** (Ink & Switch, future of computing)
- **Ink & Switch** research team
- **Martin Kleppmann** (local-first software)
- **Maggie Appleton** (design + developer tools)

**The Pitch**: "Stop leaking your IP to Altman. Run GraphRAG locally on your M3 Max."

**Engagement Strategy**:
- Reply thoughtfully to their threads about AI, privacy, local-first
- Share technical posts they'll find interesting
- Ask for feedback on architecture decisions
- Natural relationship building, not transactional asks

### 3. Content Marketing: "The Firewall Blog"

**Weekly Technical Posts** (alternating):
- Week 1: Architecture deep-dives (GraphRAG internals, zero-knowledge sync)
- Week 2: Use cases (how professionals use Robert)
- Week 3: Industry critique (problems with current AI landscape)
- Week 4: Open source contributions and benchmarks

**Target Channels**:
- Company blog (SEO + owned content)
- Cross-post to dev.to, Hashnode (developer reach)
- Submit to Hacker News when truly valuable
- Share on Twitter with context

### 4. Community Building: The "Context Control" Discord

**Purpose**: Create a community of power users who evangelize

**Channels**:
- #show-and-tell: Users share their knowledge graphs
- #feature-requests: Transparent product development
- #help: Peer-to-peer support (reduces our support burden)
- #local-first: Broader ecosystem discussions
- #enterprise: Separate space for business users

**Activation Loop**:
- New user joins → greeted by community manager
- Share their first successful query → celebrated
- Hit 100 documents → "Power User" role
- Invite a teammate → "Team Builder" role

### 5. Success Metrics (Phase 1)

- **10,000 individual users** (mix of free and pro)
- **Weekly Active Context**: 40% of users query documents >7 days old
- **Free-to-Paid conversion**: 5% (Slack-level)
- **Mac NPS**: 60+ (product-market fit signal)
- **GitHub stars**: 2,000+ (technical credibility)
- **MRR**: $100K+ (10K users × 5% paid × $20/mo = $100K MRR)

---

## Phase 2: The "Team Infection" (Months 9-15)

**Target**: 500 Teams (3-20 seats each)

**Strategy**: "Figma-Style Multiplayer." Use collaboration features to force single-player users to invite their boss.

### 1. The "Project Handoff" Trigger

Copy **Figma's growth loop**: Collaboration creates viral adoption.

**Scenario**:
- A freelance designer/developer uses Robert to organize a 6-month project
- Project includes: research docs, client meetings, design rationale, code patterns
- When they finish, client wants to understand the full context
- Freelancer needs to "hand off" the Context Graph

**The Feature**: "Transfer Context Ownership"
- Freelancer exports their project-specific context as a self-contained graph
- Client receives deep, searchable history of the entire project *instantly*
- To access it, client must create a Robert Team account
- Client sees value immediately (6 months of context in searchable format)
- Client invites their internal team to access the shared knowledge

**Growth Mechanics**:
- Every freelancer handoff = 1 new team account
- Every team account = 3-10 new seats
- Freelancers become involuntary salespeople

**Success Story to Feature**:
"Sarah (interior designer) handed off a kitchen remodel project. The client got a searchable graph of every decision, material choice, and vendor conversation. They immediately upgraded to Team plan to share it with their property management company."

### 2. The "Shared Truth" Trigger

Copy **Linear's private teams model**: Make collaboration so good that individuals demand their teams adopt it.

**Scenario**:
- Product team has 50 conflicting PDF specs across Dropbox, Notion, emails, Slack
- One team member imports everything into Robert
- They build a unified knowledge graph with relationships between specs
- Team member answers questions with: "Ask Robert, I synced all the specs"

**The Feature**: "Team Knowledge Graph"
- Users can "publish" local documents to Team Graph
- Team members can query shared knowledge base
- Context stays consistent across team (no "which version?" questions)
- Automatic conflict detection ("This spec contradicts the Q2 roadmap")

**The Hook**: Social pressure
- "Don't ask me where the spec is. Ask Robert."
- Creates FOMO: Team members who aren't on Robert feel out of the loop
- Managers see productivity gain and mandate adoption

**Pricing Design**:
- First user: $20/mo individual
- Add team features: $50/user/mo minimum 3 seats
- Seat expansion: Easy to add users (Slack model)
- **Key**: Bill team admin, not individuals (reduces friction)

### 3. The "Context Handoff" at Job Change

**Insight**: People change jobs every 2-3 years. Context portability is a forcing function.

**Scenario**:
- Developer leaves Company A for Company B
- Has 3 years of curated context (personal learning, career frameworks, industry knowledge)
- Needs to separate: What's theirs vs. what's Company A's

**The Feature**: "Context Migration Wizard"
- Automatically categorizes documents by ownership
- "Personal Learning" → User keeps
- "Company A Projects" → Stays with Company A (if they have Team plan)
- Ambiguous items → User decides
- Exports clean personal graph for new job

**Growth Mechanic**:
- User brings Robert to new company (already hooked on personal use)
- New company sees value in retaining knowledge when employees leave
- IT/HR departments become buyers ("We need institutional knowledge retention")

### 4. Viral Loops: The "Ask Robert" Share Button

**Feature**: Shareable Query Results
- User runs a query that surfaces great insights
- Clicks "Share this context" → generates secure link
- Recipient sees the query + sources + reasoning
- To ask follow-up questions → must sign up for Robert

**Use Case**:
- Consultant shares analysis with client
- Designer shares research findings with stakeholders
- Developer shares architecture decision context with team

### 5. Success Metrics (Phase 2)

- **500 teams** (3-20 seats each)
- **Seat expansion**: 3+ seats per account (viral team growth)
- **Team ARR**: $900K (500 teams × 6 avg seats × $50/user × 6 months = $900K annualized)
- **Cumulative ARR**: $2M+ ($100K individual MRR + $900K team ARR)
- **Net Revenue Retention**: 100%+ (teams add seats over time)
- **Time to team conversion**: <60 days from first individual signup

---

## Phase 3: The "Enterprise Firewall" (Months 15-24)

**Target**: 20 Enterprise Pilots ($50K+ ACV)

**Strategy**: "1Password for AI." Sell to the CISO, not the User.

**Why This Works**: Enterprises have "Shadow AI" risk—employees pasting sensitive data into ChatGPT. CISOs are desperate for solutions.

### 1. The "Shadow AI Audit" Trojan Horse

Walk into a CIO's office with data:

**The Opening**:
"Your employees are pasting 500 sensitive queries a week into ChatGPT. Here is the anonymized pattern analysis."

**The Data** (from our existing users in their org):
- 47% of queries contain potential PII
- 23% include proprietary code snippets
- 15% reference unannounced products/strategies
- 8% include customer data

**The Panic Moment**:
"If you have 1,000 employees, that's 500,000 potentially leaky queries per year. You're one breach away from a GDPR disaster."

**The Pitch**:
"Robert isn't just a chat tool; it's a **Compliance Gateway**. We block PII *before* it hits OpenAI. We give you an audit trail of *what* your employees are asking, without exposing the answers. We are your AI Firewall."

### 2. The Product: "Robert Enterprise"

**Not a Better ChatGPT—It's Infrastructure**

**Core Features**:

**1. The AI Firewall**:
- All employee queries pass through Robert's anonymization proxy
- Real-time PII detection and stripping
- Configurable policies (e.g., "never send customer names")
- Works with any LLM backend (OpenAI, Anthropic, Azure OpenAI)

**2. The Audit Trail**:
- CISO dashboard: What are employees asking? (without seeing sensitive answers)
- Compliance reports: "0 PII leaks this quarter"
- Department analytics: Which teams are AI-heavy users?
- Risk scoring: Flag high-risk query patterns

**3. The Zero-Knowledge Sync**:
- Enterprise knowledge graphs stored encrypted
- Company holds the encryption keys, not Robert
- If government subpoenas Robert, we provide encrypted blobs (unusable)
- **The 1Password pitch**: "We can't see your data even if we wanted to"

**4. Context Governance**:
- Role-based access to team knowledge graphs
- "Sales can access customer context, but not R&D roadmaps"
- Automatic context expiration (GDPR compliance)
- Employee offboarding: Revoke access, retain institutional knowledge

### 3. The Sales Motion

**Target Buyers**:
- **Primary**: CISO / Head of InfoSec
- **Secondary**: CTO / VP Engineering
- **Economic Buyer**: CFO / CIO (budget holder)

**The Wedge** (Bottom-Up):
- 50+ employees already using Robert individually
- IT discovers via expense reports or network traffic analysis
- CISO reaches out: "What is this Robert thing?"
- We respond: "Your employees love it. Let us make it compliant for you."

**The Pitch Sequence**:

**Meeting 1: The Diagnosis**
- Show anonymized data on their Shadow AI usage
- Create fear: "Here's what could leak"
- Position Robert as the solution, not the problem

**Meeting 2: The Architecture Review**
- Deep dive: How the firewall works
- Zero-knowledge sync demo
- Security audit (we provide SOC 2, pen test results)

**Meeting 3: The Pilot**
- 100-user pilot, 90 days
- Metrics: PII blocks, employee satisfaction, productivity gain
- Success criteria agreed upfront

**Meeting 4: The Close**
- Present pilot results
- Pricing: $200-500/user/year (vs $50/user for teams)
- Enterprise features: SSO, SAML, on-prem key management

### 4. Pricing: The "Enterprise Staircase"

**Team Plan** ($50/user/mo):
- Shared knowledge graphs
- Basic sync and backup
- Standard support

**Business Plan** ($100/user/mo):
- Everything in Team +
- Audit logs
- Admin controls
- Priority support

**Enterprise Plan** ($200-500/user/mo):
- Everything in Business +
- AI Firewall with PII detection
- Zero-knowledge architecture
- SAML/SSO
- On-prem key management
- Dedicated success manager
- Custom compliance reports
- SLA guarantees

**Why This Works**:
- Team users "graduate" to Enterprise as they grow
- Enterprise features justify 4-10x price increase
- CISOs have budget for compliance tools

### 5. The "Compliance as a Feature" Positioning

**Competitors**:
- ChatGPT Enterprise: Positioning as "better AI"
- We position as: "AI compliance infrastructure"

**The Difference**:
- ChatGPT Enterprise: "We promise not to train on your data" (trust us)
- Robert Enterprise: "We *can't* see your data" (zero-knowledge proof)

**Market Timing**:
- EU AI Act requires audit trails (2025-2026)
- GDPR enforcement increasing
- Companies getting fined for data leaks
- Insurance companies requiring AI governance

**The Sound Bite**:
"Robert is the 1Password for AI. You wouldn't let employees use personal passwords for company systems. Why let them use personal ChatGPT accounts?"

### 6. Success Metrics (Phase 3)

- **20 Enterprise Pilots** ($50K-100K ACV each)
- **Enterprise ARR**: $1-2M (20 pilots × $50-100K)
- **Total ARR**: $3-4M ($100K individual + $900K team + $1-2M enterprise)
- **Sales cycle**: <90 days (fear-driven buying)
- **Win rate**: 40%+ (if we get to pilot stage)
- **Net Revenue Retention**: 120%+ (seat expansion + upsells)

---

## The Credibility Slide: Benchmarks & Ratios

To prove we're not hallucinating, here are industry-standard benchmarks.

| Stage | Metric Focus | Benchmark (Good) | Benchmark (Great) | Robert Target |
|:---|:---|:---|:---|:---|
| **Seed** | **Weekly Active Context** | 20% of users | 40% of users | 40% |
| **Seed** | **Free-to-Paid Conversion** | 2% | 5% (Slack level) | 5% |
| **Expansion** | **Seat Expansion** | 1.5 seats/account | 3+ seats/account | 3+ |
| **Series A** | **Team ARR** | $500K | $1M+ | $900K |
| **Series A** | **Total ARR** | $1M | $2M+ | $3-4M |
| **Enterprise** | **Net Revenue Retention** | 100% | 120% (Snowflake) | 120% |
| **Enterprise** | **Sales Cycle** | 120 days | <90 days | <90 days |

---

## The Moat: Why This Compounds

**Year 1**: Individual developers use Robert for personal productivity

**Year 2**: Those developers become team leads, bring Robert to their teams

**Year 3**: Those teams become departments, enterprise mandates Robert for compliance

**The Compounding Effects**:

1. **Data Moat**: The longer users curate their knowledge graphs, the more valuable and irreplaceable they become
2. **Network Effects**: Team knowledge graphs are only valuable if the whole team is on Robert
3. **Switching Costs**: Losing your knowledge graph = losing years of professional memory
4. **Compliance Moat**: Once Robert is the "system of record" for AI governance, ripping it out requires re-architecting compliance

**The "Wedding Photos" Psychology**:
People pay for Google Photos not because storage is expensive, but because losing memories is unbearable. Business knowledge graphs trigger the same emotion—but with 10x the willingness to pay.

---

## The Capital Allocation

**$4M Seed Round Deployment**:

**Engineering (40% - $1.6M)**:
- 3 senior engineers × $200K × 18mo = $1.08M
- 1 ML/AI engineer × $250K × 18mo = $450K
- Infrastructure (cloud, security audits) = $70K

**Go-To-Market (40% - $1.6M)**:
- 1 VP Marketing/Growth × $180K × 18mo = $270K
- 1 Developer Relations × $150K × 18mo = $225K
- 1 Enterprise AE × $150K + commission × 12mo = $250K
- Content/influencer budget = $100K
- Launch week execution = $50K
- Conferences/events = $75K
- Tools (CRM, analytics, etc.) = $30K
- Contingency/experiments = $600K

**Operations (20% - $800K)**:
- Founders × 2 × $150K × 18mo = $540K
- Legal, accounting, compliance = $100K
- Rent, admin, insurance = $90K
- Recruiting = $70K

---

## The Ask

**"We are raising $4M Seed to execute Phase 1-2:**
- **Win the 'Vibe Coders'** and prove the Weekly Active Context metric
- **Activate team viral loops** and hit $2M ARR
- **Launch 5 enterprise pilots** to validate compliance positioning

**We have the architecture to build the Firewall. We need the capital to build the Movement."**

---

## Appendix: The Comparables

**Companies that executed similar playbooks**:

**Figma** ($20B exit to Adobe):
- Started: Designer tool (bottoms-up)
- Grew: Team collaboration (viral loops)
- Sold: Enterprise design system (compliance, governance)

**Slack** ($28B exit to Salesforce):
- Started: Developer tool (IRC replacement)
- Grew: Team productivity (viral)
- Sold: Enterprise communication infrastructure

**1Password** ($2B valuation):
- Started: Individual password manager
- Grew: Family plans (5-10 seats)
- Sold: Enterprise identity/security infrastructure

**GitHub** ($7.5B exit to Microsoft):
- Started: Developer hosting (Git repos)
- Grew: Open source collaboration
- Sold: Enterprise code governance

**Robert follows the same trajectory**: Individual utility → Team productivity → Enterprise compliance infrastructure.
