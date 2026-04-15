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
│ App layer — truth definitions, projections, validation     │
│ the "what" that governance decides                         │
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
2. **Axiom** builds the truth, intent, and projection model for the run.
3. **Axiom** starts `Engine.run()`, which activates the governed intelligence loop.
4. **Organism** performs huddle, debate, research, and gap-chasing, emitting `ProposedFact` and `AgentEffect`.
5. **Converge** decides what is promotable, applies policy and budget, and records the audit trail.
6. **Providers** supply the external capabilities behind `ChatBackend`, `WebSearchBackend`, `DdLlm`, and `DdSearch`-style adapters.

## Repo Ownership Split

**This repo owns Helm and Axiom for the hackathon experience:**
- The desktop operator surface in `apps/desktop`
- Governance domain records (vendors, decisions, audit entries)
- Truth definitions for hackathon use cases
- Projection from converged facts into domain records
- Offline validation and local-first developer workflows
- The shared application layer and local harness

**Organism owns the intelligence path:**
- Intent decomposition and planning
- Huddle, debate, research, and gap-chasing
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

## Opinionated Implementation

- **Rust-first** for Axiom, domain logic, policy enforcement, integrations, and mocks
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

See also: [[Architecture/Layers]], [[Architecture/Convergence Loop]]
