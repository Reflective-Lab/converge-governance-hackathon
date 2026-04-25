---
name: KB Mutation Log
type: reference
---

# Knowledge Base Mutation Log

## 2026-04-25

**Added team pitch for governed vendor selection:**
- Created `kb/Presentations/Team Pitch - Governed Vendor Selection.md` with a four-act presentation structure, live demo commands, and a three-day fulfillment plan.
- Linked the page from `kb/Home.md` under a new Presentations section.
- Separated the reliable governed-selection flow from the future-facing Pareto breakout story, while noting that both modes have executable headless paths.
- Updated the pitch notes after the demo-readiness pass to point at `just demo`, `just demo-live`, `just demo-step`, `just demo-ai-vendors --business`, and `just demo-verify`.
- Updated the runbook after presentation feedback: Step 3 now emphasizes boxed process/agent outcomes and explicit HITL/Cedar delegation, Step 4 explains gate mechanics, Step 5 is framed as a negative-control authority run, Step 6 explains prior-context learning, and Step 7 uses richer provider-mix data with a governed Pareto frontier.
- Documented live-provider fallback behavior: the presentation script tries the intended OpenRouter model tiers, then falls through to direct configured providers if OpenRouter is unavailable or out of credits.
- Added liveness indicators to the terminal walkthrough: each long-running evaluation prints an estimated duration and rotates through the desktop spinner verbs from `apps/desktop/src/lib/spinner.ts`.
- Added an explicit RFI/RFP boundary note: the demo focuses on governed evaluation and recommendation after criteria/vendor responses exist, while problem definition, RFI longlisting, PoC, negotiation, contracting, and onboarding are treated as upstream/downstream assumptions.
- Expanded the presentation script and meeting notes with explicit upstream-assumption setup text and downstream-action handoff text for management review, negotiation, legal/security/finance validation, PoC, contracting, onboarding, and learning feedback.

## 2026-04-24

**Added web search provider mix demo story:**
- Created `kb/Integrations/Web Search Demo Story.md` explaining the high-level demo narrative for Brave as wide discovery and Tavily as deep retrieval.
- Linked the page from `kb/Home.md` under Integrations.
- Framed provider selection as agent-specific governance: compliance uses deep evidence, cost uses wide-then-deep lookup, risk uses wide signal discovery, and synthesis consumes promoted facts before escalating.
- Extended the story with a "Not Every Agent Is An LLM" slide showing knowledgebase/RAG, policy, optimization/math, statistics, machine learning, and data analysis as first-class decision capabilities.
- Added a "Relentless Exploration, Governed Promotion" slide to express persistent, open-ended evidence seeking while keeping Cedar, authority, provenance, and hard constraints in control.
- Added the two headless demo modes: governed selection for RFI/RFP convergence, and creative Pareto breakout for router/provider-mix discovery without bypassing governance.
- Added the desktop slide mapping for slides 15-25 and noted that the desktop presenter can select an arbitrary subset by original slide number.

**Updated Experience and Recall status:**
- Documented the persistent `ExperienceRegistry` in `governance-server`.
- Added the `GET /v1/experience/{truth_key}` read surface used by the desktop Vendor Decision Lab.
- Clarified that experience calibrates planning and recall, while audit remains the authoritative legal record.

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
