# Hackathon Milestones

> Single source of truth for what ships and when.
> Every session starts by reading this file. Scope work to the current milestone.
>
> See `~/dev/work/EPIC.md` for the coarse-grained outcomes these milestones advance.

---

## Current: Ready for Students
**Deadline:** TBD | **Epic:** E6 (Hackathon is ready for students)

**Goal:** A student with Rust experience can clone, build, and explore Converge governance in under 10 minutes. They can write a truth, add a Cedar policy, and see it execute — without hitting walls.

### Getting started (zero to running)

- [ ] `just setup` recipe installs all dependencies and verifies toolchain
- [ ] `just seed` populates meaningful starting scenario (vendor evaluation, budget approval)
- [ ] `just dev` starts both server and desktop app in one command
- [ ] Getting Started guide tested from clean machine — under 10 minutes to first governance run
- [ ] CAPABILITIES.md makes it obvious what students can build with

### API surface alignment (student-facing)

- [x] Canonical authoring examples use `converge-pack` for `Suggestor`, `AgentEffect`, `ProposedFact`, and `ContextKey`
- [x] Canonical in-process runtime examples use `converge-kernel` for `Engine`, `Context`, `Budget`, criteria, and run hooks
- [x] Canonical LLM examples use `ChatBackend` + `ChatRequest` instead of `KongGateway` / `LlmRequest`
- [x] Canonical Organism examples use `organism-pack` + `organism-runtime` for `IntentPacket`, `Plan`, and standard-pack registry wiring
- [x] `apps/desktop/src-tauri` is migrated off legacy `converge-provider` / `converge-axiom` APIs such as `KongGateway` and `StaticLlmProvider`
- [x] One `Programming API Surfaces` guide is linked from README, Getting Started, and `kb/Home.md`

### Example truths (3+ modifiable)

- [ ] **Vendor selection truth** — Multi-criteria evaluation with Cedar policy gates (students can add criteria)
- [x] **Dynamic due-diligence truth** — Research loop with dynamic gap-chasing, contradictions, and structured synthesis
- [ ] **Budget approval truth** — Amount thresholds requiring HITL approval (students can adjust policy)
- [ ] **Access control truth** — Role-based governance with delegation tokens (students can define new roles)
- [ ] Each example has: truth definition, Cedar policy, test, documentation in kb/

### Cedar policies (hands-on)

- [ ] Policy fixtures with comments explaining each clause
- [ ] At least 3 policies students can modify and see immediate effects
- [ ] Test harness: change policy → re-run truth → see different outcome
- [ ] Cedar policy validator with clear error messages

### Desktop visualization

- [ ] Governance decision flow visible in real time
- [ ] Truth execution timeline (proposal → policy check → promotion → convergence)
- [ ] "What happened and why" view for each governance decision
- [ ] Agent proposal/promotion lifecycle visible

### Seed data

- [ ] Realistic vendor evaluation scenario (3+ vendors, 5+ criteria)
- [ ] Budget approval scenario with amounts near threshold boundaries
- [ ] Pre-loaded facts and proposals students can inspect

### Documentation

- [ ] kb/ pages reviewed for accuracy and completeness
- [ ] Truth addition process documented with working example
- [ ] "Build your first truth" tutorial in kb/Development/
- [ ] Common pitfalls and troubleshooting guide

---

## Next: Kong AI Gateway Integration

**Epic:** E7 — Two-layer AI governance with Kong as external governance layer

**Goal:** Position Kong as the external governance layer for all AI system access. Participants see the two-layer story: Converge governs decisions, Kong governs external access.

### K1: Kong Wiring (COMPLETE)

- [x] `KongBackend` implementing `DynChatBackend` in `converge-provider`
- [x] `select_chat_backend()` recognizes `"kong"` as a provider
- [x] `.env` wiring: `KONG_AI_GATEWAY_URL` and `KONG_API_KEY` consumed
- [x] Desktop app and server route through Kong when configured
- [x] Falls back to direct providers or offline when Kong unreachable

### K2: Kong Governance Plugins

- [ ] AI Rate Limiting — per-team token budgets (configured in Konnect)
- [ ] AI PII Sanitizer — redact sensitive data before prompts reach models
- [ ] AI Prompt Guard — block prompts containing prohibited patterns
- [ ] AI Audit Log — every request logged with tokens, cost, latency

### K3: Governance Data Pipeline

- [ ] Kong audit/metrics accessible from Rust (API or log ingestion)
- [ ] Cost-per-evaluation flows into `DecisionRecord` metadata
- [ ] Token usage per agent visible in truth execution results
- [ ] Desktop dashboard panel: Kong governance data alongside Converge
- [ ] "What it cost" summary in truth execution view

### K4: Demo Story

- [ ] All agents run with real LLM calls through Kong
- [ ] Kong dashboard showing rate limits, PII redaction, cost tracking
- [ ] Desktop app showing two-layer governance (Converge + Kong)
- [ ] "Change a policy, see a different outcome" demo
- [ ] "Kong blocks an agent" demo (honest stopping)
- [ ] Presentation-ready walkthrough script

**Dependencies:** K1 → K2 → K3 → K4 (sequential)
