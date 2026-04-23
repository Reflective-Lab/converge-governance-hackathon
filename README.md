# Converge Governance Hackathon

**Build a governed AI decision system that can justify every decision it makes.**

Clone this repo. Replace the placeholder agents with real logic. Ship a system where every fact has provenance, every decision has evidence, and every run either converges or stops honestly.

```
┌─────────────────────────────────────────────────────────────┐
│  Helm          Desktop UI — what operators see              │  ← you build this
├─────────────────────────────────────────────────────────────┤
│  Axiom         Truth definitions, validation, projection    │  ← you build this
├─────────────────────────────────────────────────────────────┤
│  Organism      Intent, planning, adversarial, simulation    │  ← you compose this
├─────────────────────────────────────────────────────────────┤
│  Converge      Engine, promotion gates, Cedar policy, audit │
├─────────────────────────────────────────────────────────────┤
│  Providers     LLMs, search, tools                          │
└─────────────────────────────────────────────────────────────┘
```

Teams work in the top two layers. The bottom three are the foundation: governance that cannot be bypassed, policies enforced by Cedar, and an engine that converges to a fixed point or tells you exactly why it couldn't.

## Quick Start

```bash
git clone <your-team-repo>
cd converge-governance-hackathon
just hit-the-ground-running   # build, test, lint — verify everything works
just server                   # start local harness (localhost:8080)
```

```bash
# Execute a vendor evaluation
curl -X POST http://localhost:8080/v1/truths/evaluate-vendor/execute \
  -H 'Content-Type: application/json' \
  -d '{"inputs": {"vendors": "Acme AI, Beta ML, Gamma LLM"}}'
```

## Prerequisites

| Tool | Version | Install |
|---|---|---|
| **Rust** | 1.94+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Bun** | latest | `curl -fsSL https://bun.sh/install \| bash` |
| **just** | latest | `cargo install just` (or `brew install just` on macOS) |

**Tauri 2 (for desktop):** macOS needs `xcode-select --install`. Linux needs `libwebkit2gtk-4.1-dev build-essential libssl-dev`.

## The Challenge

Enterprise AI vendor selection. Multiple agents evaluate vendors for compliance, cost, risk, and capability. Their proposals pass through promotion gates before becoming facts. Cedar policies authorize consequential actions. The engine runs agents in cycles until no agent has anything new to add — convergence.

This is the reference challenge, but the architecture works for any governed decision process.

## What You Build

### 1. Replace Placeholder Agents

The agents in `evaluate-vendor` return hardcoded facts. Replace them with real logic:

```rust
struct ComplianceScreenerAgent { /* inject services */ }

impl Suggestor for ComplianceScreenerAgent {
    fn name(&self) -> &str { "compliance-screener" }
    fn dependencies(&self) -> &[ContextKey] { &[] }
    fn accepts(&self, ctx: &dyn ContextView) -> bool { /* when to wake up */ }
    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        // Real compliance checks — LLM calls, policy lookups, service queries
    }
}
```

### 2. Use Organism Patterns

Go beyond flat agent swarms. Organism gives you structured decision-making:

```rust
// Structured planning with intent decomposition
let intent = IntentPacket::new("Evaluate AI vendors", expires)
    .with_context(json!({"vendors": vendors}))
    .with_authority(vec!["vendor_evaluation".into()]);

// Pick a collaboration topology
let charter = CollaborationCharter::huddle();  // strict, with dissent tracking
// or: discussion_group(), panel(), self_organizing()
```

See `kb/Converge/Organism Patterns.md` for the full pattern catalog.

### 3. Write Cedar Policies

```cedar
permit(
    principal == Role::"procurement-lead",
    action == Action::"commit-vendor",
    resource
) when {
    resource.amount < 50000
};
```

### 4. Build the Desktop UI

The Svelte/Tauri shell is scaffolded. Build the operator experience — load `.feature` files, visualize the governance flow, show the audit trail.

## How Convergence Works

```
Cycle 1: ComplianceScreener → proposes compliance facts
Cycle 2: CostAnalysis wakes up (sees compliance) → proposes cost facts
Cycle 3: RiskScorer wakes up (sees evaluations) → proposes risk scores
Cycle 4: DecisionSynthesis wakes up (sees all facts) → proposes recommendation
Cycle 5: No new facts → fixed point → converged
```

Every run terminates honestly. Converged, budget exhausted, invariant violated, or human review required. The system never silently gives up.

## API Surfaces

| What | Crate | Key Imports |
|---|---|---|
| Write agents | `converge-pack` | `Suggestor`, `AgentEffect`, `ProposedFact`, `ContextKey` |
| Run the engine | `converge-kernel` | `Engine`, `Context`, `Budget` |
| Call LLMs | `converge-provider` | `ChatBackend`, `ChatRequest` |
| Plan with Organism | `organism-pack` | `IntentPacket`, `Plan`, `CollaborationCharter` |
| Organism runtime | `organism-runtime` | `Registry`, readiness, built-in packs |

## Cheat Sheet

```
just hit-the-ground-running   # first time — build, test, lint
just server                   # start local harness (localhost:8080)
just desktop                  # launch Tauri desktop app
just test                     # run all tests
just test-coverage            # tests with coverage report
just lint                     # clippy + format check
```

## Project Structure

```
crates/
  governance-kernel/         Domain model + in-memory store
  governance-truths/         Truth catalog + Converge bindings
  governance-server/         Local HTTP harness + truth executors
  governance-app/            Shared Rust app layer (desktop + server)
apps/
  desktop/                   Svelte + Tauri shell
kb/                          Obsidian vault — the documentation
examples/                    Gherkin + JSON input specs
```

## Dependency Versions

All dependencies are pinned to git tags. Cargo enforces version consistency.

| Dependency | Tag | What |
|---|---|---|
| Converge | v3.7.3 | Governance engine, promotion gates, Cedar policy, formation substrate |
| Organism | v1.4.0 | Intent, planning, adversarial, simulation, learning |
| Axiom | v0.7.0 | Truth validation, Gherkin parsing, policy lens |

## Judging Criteria

1. **Governance quality** — traceable decisions an auditor can follow
2. **Agent sophistication** — real analysis, not hardcoded facts
3. **Honest stopping** — the system admits when it can't decide
4. **Policy coverage** — Cedar policies that enforce real constraints
5. **Demo quality** — end-to-end vendor evaluation walkthrough

## Sponsors

Kong · Vivicta

Challenge contributor: [Reflective Labs](https://reflective.se)

## License

[MIT](LICENSE) — Copyright 2024–2026 Reflective Group AB
