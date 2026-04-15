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
