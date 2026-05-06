---
tags: [architecture]
---
# Architecture Overview

This repo is an opinionated local-first application built on top of the full Reflective Labs stack.

## The Five Layers

```
┌─────────────────────────────────────────────────────────────┐
│ Helm (control surface)                                     │
│ Desktop UI — Svelte/Tauri — what operators see             │
├─────────────────────────────────────────────────────────────┤
│ Axiom (truth layer)                                        │
│ Truth contracts, validation, simulation, policy lens       │
│ The "what must be true" before governance runs             │
├─────────────────────────────────────────────────────────────┤
│ Organism (intelligence)                                    │
│ Intent → Huddle → Debate → Suggestors                      │
│ The "how" — reasoning, research, gap-chasing               │
├─────────────────────────────────────────────────────────────┤
│ Converge (governance)                                      │
│ Engine, promotion gates, Cedar policy, budget, audit       │
│ The "whether" — authority, trust, stop rules               │
├─────────────────────────────────────────────────────────────┤
│ Providers (capability)                                     │
│ OpenRouter, Anthropic, OpenAI, Gemini, Brave, Tavily       │
└─────────────────────────────────────────────────────────────┘
```

The control flow between those layers is:

1. **Helm** calls local Tauri commands.
2. **Axiom** validates the truth contract: inputs, examples, invariants, policy lens, and acceptance criteria.
3. **Organism** plans the formation or collaboration pattern needed to satisfy the truth.
4. **Converge** runs the governed loop, promotes valid facts, applies policy and budget, and records the audit trail.
5. **Hackathon app code** projects promoted evidence into the local product model when writeback is requested.
6. **Providers** supply the external capabilities behind `ChatBackend`, `WebSearchBackend`, `DdLlm`, and `DdSearch`-style adapters.

## Repo Ownership Split

**This repo owns the hackathon product surface:**
- The desktop operator surface in `apps/desktop`
- Governance domain records (vendors, decisions, audit entries)
- Truth contract wiring for hackathon use cases
- Projection from converged facts into domain records
- Offline validation and local-first developer workflows
- The shared application layer and local harness

**Axiom owns the truth contract path:**
- Definition of what a governed decision must prove
- Validation of inputs, examples, invariants, and acceptance criteria
- Simulation of failure modes and admissibility checks
- Policy lens for required evidence, gates, and approvals

**Organism owns the intelligence path:**
- Intent decomposition and planning
- Huddle, debate, research, and gap-chasing
- Formation strategy and suggestor composition
- Suggestor-level reasoning that feeds governed proposals

**Converge owns the governance path:**
- Suggestor execution cycles
- Shared context and context partitions
- Fact proposal and promotion
- Cedar-backed policy and authority checks
- Criteria evaluation, budgets, stop reasons, and audit trail

**Providers own the external world:**
- LLM backends and routing
- Web search and research services
- MCP tools and other capability adapters

Teams should build *with* Converge, not around it. The value is governed convergence, not just "multiple calls to an LLM."

Axiom is not another agent layer. Its authority is normative: it says what a valid decision must prove. Organism's authority is strategic: it chooses the formation to prove it. Converge's authority is operational: it runs the governed loop and records what was actually promoted. The hackathon app owns product experience, imported artifacts, demo data, and writeback.

## Foundation Baseline

The user-side application currently consumes Converge `v3.8.1`, Organism `v1.5.0`, Axiom `v0.7.0`, and the Converge extension stack pinned in the root Cargo manifest. Local sibling patches are retained only for unpublished platform and extension crates.

Converge `v3.8.1` strengthens several boundaries this app follows: typed pack and policy identifiers, `ContextState` for owned runtime context, `&dyn Context` for evaluator reads, and private `ProposedFact` confidence through builder methods. Those constraints are intentional; they make the user-facing examples match the governance model instead of relying on stringly typed shortcuts.

Organism `v1.5.0`, Axiom `v0.7.0`, and Ferrox Solvers `v0.4.1` are aligned to Converge `v3.8.1`, so the user-side stack has one governance/provider contract across intent planning, truth validation, runtime execution, optimization, and direct suggestor authoring.

## Opinionated Implementation

- **Rust-first** for Axiom-facing contracts, domain logic, policy enforcement, integrations, and mocks
- **Svelte** for Helm
- **Tauri** for the Helm ⇄ Axiom bridge
- **Converge** for the commit boundary and governance engine
- **Organism** for intelligence, planning, and debate
- **Provider capability adapters** for LLMs, search, and tool access

## Local Input Model

The desktop app accepts two local input formats for [[Domain/Vendor Selection|vendor selection]]:

- Gherkin `.feature` files
- Truth-spec `.truths.json` files

Both are normalized in the Rust app layer before execution.

See also: [[Architecture/Layers]], [[Architecture/Axiom Truth Contract]], [[Architecture/Convergence Loop]]
