---
name: KB Mutation Log
type: reference
---

# Knowledge Base Mutation Log

## 2026-04-23

**Added Axiom truth contract boundary:**
- Created `kb/Architecture/Axiom Truth Contract.md` defining Axiom as the normative executable specification layer for governed decisions.
- Updated architecture pages so Axiom defines what must be true, Organism forms the team to satisfy it, Converge governs execution and promotion, and hackathon apps own UI/artifacts/writeback.
- Clarified that Axiom is not an agent runtime, formation compiler, promotion path, prior learner, or authoritative business state store.

## 2026-04-19

**Added Organism Patterns page:**
- Created `kb/Converge/Organism Patterns.md` — six-stage pipeline, four collaboration topologies, five skepticism kinds, five simulation dimensions, 15 domain packs, intent resolution levels
- Added link to `kb/Home.md` under Converge Platform section

**Handoff split documented for participant template vs system integration:**
- Added `kb/Development/Template Handoff.md` documenting:
  - keep vs move boundaries for repository layers,
  - system-level work to re-home in a runway/integration repo,
  - release strategy for stable `converge` / `organism` / `axiom` consumption,
  - handoff responsibilities for template, release, and docs owners.
- Linked the new page from `kb/Home.md` under Development.

## 2026-04-18

**Added Kong API reference pages:**
- `kb/Integrations/Kong Chat Response.md` — Response body fields and gateway response headers (what we capture per LLM call)
- `kb/Integrations/Kong Admin API.md` — Konnect admin API surface: control planes, services, routes, plugins, consumers, analytics, audit logs
- Updated `Kong Gateway.md` cross-references

## 2026-04-17

**Added in-app provider/model selection for agents:**
- `governance-truths/src/lib.rs` — Added `AgentModelConfig` with `AgentRequirements` for each agent (ComplianceScreener, CostAnalysis, CapabilityMatcher, RiskScorer, DecisionSynthesis). Each agent specifies cost, latency, quality, and reasoning requirements.
- `governance-server/src/main.rs` — Added `GET /v1/agents/available-models` endpoint with **health checks**:
  - Uses `ModelSelector` from converge-provider to find models matching each agent's requirements
  - For each candidate model, sends a minimal test request ("test", 10 tokens max) to validate the API key
  - Filters out invalid/expired keys, timeouts, and unreachable providers
  - Returns only working models in the response
- `apps/desktop/src/lib/ProviderSelector.svelte` — New component that fetches available models at startup, shows each agent with its recommended model, persists selection to localStorage, and validates that all agents have available providers before starting demo.
- `apps/desktop/src/App.svelte` — Integrated ProviderSelector as new "providers" phase between slides and demo. Flow: slides → providers setup → demo.

**Why this matters:** Prevents 401 "authentication denied" errors mid-demo. Participants can:
1. Add an API key to `.env` (Anthropic, OpenAI, OpenRouter, Kong)
2. Restart the server
3. The app auto-detects and validates the key
4. Provider Setup screen shows only working providers
5. Each agent gets the right model for its job (fast models for screening, capable models for synthesis)

**Added three comprehensive visualization architecture pages:**
- `Converge/Threlte Visualization.md` — Why Threlte was chosen for 3D agent convergence visualization, with design principles, reactive data flow, performance trade-offs, and stack coherence rationale.
- `Converge/Visualization Alternatives.md` — Detailed comparison of 7 alternatives (Raw Three.js, React Three Fiber, Bevy, Canvas 2D, Babylon.js, Custom WebGL, Unreal/Unity) with trade-offs, decision matrix, and use cases.
- `Converge/Bevy Deep Dive.md` — ECS architecture fundamentals, code examples, performance characteristics, and 5 scenarios where Bevy wins (physics, 1000+ agents, complex events, video export, offline rendering).

**Updated kb/Home.md** — Added new "Visualization" section linking all three pages.

**Replaced all student → participant references** — Bulk update across 16 files (AGENTS.md, README.md, MILESTONES.md, CAPABILITIES.md, kb/ pages, examples, and apps) to use "participants" or "participant-facing" instead of "students" or "student-facing". This aligns the project terminology with official hackathon language.

**Enhanced `Integrations/Why Kong.md`** — Added "Participant Pitch" section with the two-layer governance diagram, live demo walkthrough, and comparison table. This is the participant-facing framing used in the 30-minute hackathon kickoff meeting.
