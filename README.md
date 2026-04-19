# Converge Governance Hackathon

**Build a governed AI decision system that can justify every decision it makes.**

This is a template repository. Clone it, replace the placeholder agents with real logic, and ship a system where every fact has provenance, every decision has evidence, and every run either converges or stops honestly.

```
┌─────────────────────────────────────────────────────────────┐
│  Helm          Desktop UI — what operators see              │  ← teams build this
├─────────────────────────────────────────────────────────────┤
│  Axiom         Truth definitions, validation, projection    │  ← teams build this
├─────────────────────────────────────────────────────────────┤
│  Organism      Intent, planning, reasoning                  │
├─────────────────────────────────────────────────────────────┤
│  Converge      Engine, promotion gates, Cedar policy, audit │
├─────────────────────────────────────────────────────────────┤
│  Providers     LLMs, search, tools                          │
└─────────────────────────────────────────────────────────────┘
```

Teams work in the top two layers — **Helm** and **Axiom**. The bottom three layers are the foundation: governance that cannot be bypassed, policies enforced by Cedar, and an engine that converges to a fixed point or tells you exactly why it couldn't.

## The Challenge

Enterprise AI vendor selection. Multiple agents evaluate vendors for compliance, cost, risk, and capability. Their proposals pass through promotion gates before becoming facts. Cedar policies authorize consequential actions. The engine runs agents in cycles until no agent has anything new to add — convergence.

This is the reference challenge, but the architecture works for any governed decision process: budget approval, access control, procurement, risk scoring.

## What's Already Built

| Component | Status | What Teams Do |
|---|---|---|
| Convergence engine | Working | Use it — don't modify |
| Cedar policy gates | Working | Write policies for your domain |
| Four governed truths | Working | Extend with real agent logic |
| Domain model | Working | Add types for your domain |
| Desktop shell | Scaffold | Build the operator experience |
| Placeholder agents | Hardcoded | Replace with LLM, analytics, real checks |

### The Four Truths

| Truth | What It Governs |
|---|---|
| `evaluate-vendor` | Multi-criteria vendor evaluation — compliance, cost, risk, synthesis |
| `dynamic-due-diligence` | Organism-planned research loop with gap-chasing and contradiction detection |
| `audit-vendor-decision` | Trust pack — audit trail for every vendor decision |
| `authorize-vendor-commitment` | Cedar policy gates for procurement authorization |

## How Convergence Works

Agents run in cycles. Each cycle, they read shared context, propose facts, and return. The engine validates proposals through the promotion gate. When no agent has anything new to propose — fixed point. Convergence.

```
Cycle 1: ComplianceScreener → proposes compliance facts
Cycle 2: CostAnalysis wakes up (sees compliance) → proposes cost facts
Cycle 3: RiskScorer wakes up (sees evaluations) → proposes risk scores
Cycle 4: DecisionSynthesis wakes up (sees all facts) → proposes recommendation
Cycle 5: No new facts → fixed point → converged
```

**Every run terminates honestly.** Converged, budget exhausted, invariant violated, or human review required. The system never silently gives up.

## Why This Architecture

**Security by construction.** `ProposedFact` is not `Fact`. No agent can bypass the promotion gate. Cedar policies enforce authorization. `unsafe_code = "forbid"` everywhere.

**Anti-fragility by design.** Adversarial review challenges every plan. Simulation swarms stress-test across five dimensions. The system gets stronger from disagreement, not weaker.

**Governance as foundation.** Every fact carries provenance — who proposed it, when, with what confidence. Every decision has an evidence chain an auditor can follow. This isn't bolted on; it's the architecture.

## Cheat Sheet

```
just hit-the-ground-running   # first time — build, test, lint
just server                   # start local harness (localhost:8080)
just desktop                  # launch Tauri desktop app
just check                    # fast compile check
just test                     # run all tests
just lint                     # clippy + format check
just fmt                      # auto-format
just clean                    # nuke build artifacts
```

**With AI agents (Claude, Codex, Gemini):**

```
/focus                        # session opener — milestone, deliverables, context
/sync                         # pull latest, PRs, issues, build health
/next                         # pick next task from milestone
/fix <issue#>                 # branch → implement → test → PR
/check                        # lint + test + report
/pr                           # push and create pull request
/done                         # end session — progress, changelog, observations
/audit                        # weekly — security, compliance, drift
/help                         # show all skills
```

## Prerequisites

Install these before cloning:

| Tool | Version | Install |
|---|---|---|
| **Rust** | 1.94+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Bun** | latest | `curl -fsSL https://bun.sh/install \| bash` |
| **just** | latest | `cargo install just` (or `brew install just` on macOS) |

**Platform dependencies for Tauri 2:**

- **macOS** — Xcode Command Line Tools: `xcode-select --install`
- **Linux** — `sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
- **Windows** — [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11)

## Quick Start

```bash
# 1. Clone your team's repo
git clone <your-team-repo>
cd converge-governance-hackathon

# 2. Build, test, and verify everything works
just hit-the-ground-running

# 3. Launch the desktop app
just desktop
```

That's it. Step 2 compiles the Rust workspace and runs tests. Step 3 installs frontend deps and launches the Tauri dev window.

### Run the Server (alternative)

```bash
just server  # localhost:8080
```

```bash
# Execute a vendor evaluation
curl -X POST http://localhost:8080/v1/truths/evaluate-vendor/execute \
  -H 'Content-Type: application/json' \
  -d '{"inputs": {"vendors": "Acme AI, Beta ML, Gamma LLM"}}'
```

### Run the Desktop (standalone)

```bash
just desktop          # installs deps + launches Tauri dev mode
```

## What Participants Build

### 1. Replace Placeholder Agents

The four agents in `evaluate-vendor` return hardcoded facts. Replace them with real logic:

```rust
// Implement the Suggestor trait from converge-pack
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

### 2. Write Cedar Policies

Define what actions are authorized, at what thresholds, by whom:

```cedar
permit(
    principal == Role::"procurement-lead",
    action == Action::"commit-vendor",
    resource
) when {
    resource.amount < 50000
};
```

### 3. Build the Desktop UI

The Svelte/Tauri shell is scaffolded. Build the operator experience — load `.feature` files, visualize the governance flow, show the audit trail.

### 4. Add More Truths

Budget approval, access control, risk scoring — any decision process that needs governance. Define the truth, write the agents, add Cedar policies.

## API Surfaces

| What | Crate | Key Imports |
|---|---|---|
| Write agents | `converge-pack` | `Suggestor`, `AgentEffect`, `ProposedFact`, `ContextKey` |
| Run the engine | `converge-kernel` | `Engine`, `Context`, `Budget` |
| Call LLMs | `converge-provider` | `ChatBackend`, `ChatRequest` |
| Test offline | `converge-axiom` | `StaticChatBackend` |
| Plan with Organism | `organism-pack` | `IntentPacket`, `Plan`, reasoning systems |

## Tech Stack

| Layer | Technology |
|---|---|
| System logic | **Rust** (edition 2024, 1.94+) |
| Governance | **Converge v3** — promotion gates, convergence engine |
| Policy | **Cedar** — Amazon's authorization language |
| Intelligence | **Organism v1** — intent, planning, reasoning |
| LLM access | **converge-provider** — Anthropic, OpenAI, Gemini, Mistral, OpenRouter |
| Desktop | **Tauri 2** + **SvelteKit 5** |
| Package manager | **Bun** |
| Task runner | **just** |

## Working with AI Agents

This repo is designed for three AI agents — **Claude Code**, **Codex**, and **Gemini** — as first-class collaborators. Each reads the same canonical documentation and uses the same workflow infrastructure.

### The Documentation Layers

| Layer | What | Who reads it |
|---|---|---|
| `AGENTS.md` | Canonical project documentation — philosophy, stack, architecture, rules | All agents (entrypoint) |
| `CLAUDE.md` / `CODEX.md` / `GEMINI.md` | Agent-specific configuration, each points to AGENTS.md | Respective agent |
| `kb/` | Obsidian vault — architecture, domain, development, integrations, workflow | Agents + participants/teams (lazy-loaded) |
| `MILESTONES.md` | What ships and when — current milestone, deliverables, deadlines | `/focus`, `/next`, `/done` |
| `CHANGELOG.md` | What shipped — notable changes by version | `/done` updates it |
| `Justfile` | Deterministic shell recipes — build, test, lint, serve | Agents + participants/teams |
| `.claude/skills/` | 14 slash commands — multi-step AI-driven workflows | Claude Code |
| `scripts/workflow/` | Shell scripts backing `just focus`, `just sync`, `just status` | All agents + terminal |

### Skills vs Justfile vs Scripts

Three layers, different purposes:

| Need | Use | Example |
|---|---|---|
| Run a single command | `just` recipe | `just lint`, `just test`, `just server` |
| Multi-step workflow with reasoning | `/skill` command | `/fix 42` — reads issue, branches, implements, tests, PRs |
| Repo state inspection | `scripts/workflow/` | `focus.sh` — build health, PRs, issues, recommendations |

Skills call `just` recipes internally. `/check` runs `just lint && just test`. `/fix 42` runs `just check` as a verification step. The layers compose.

### The 14 Skills

| Skill | When | What it does |
|---|---|---|
| `/focus` | Session start | Reads milestone, shows days left, open deliverables |
| `/sync` | Morning | Pulls latest, shows PRs, issues, build health |
| `/next` | Pick work | Lists unchecked deliverables from current milestone |
| `/fix <#>` | During work | Branch → implement → test → lint → commit → PR |
| `/check` | Before commit | Runs lint + test, reports findings |
| `/ticket <desc>` | Create issue | Explores code, writes requirements, suggests size |
| `/pr` | Ship work | Pushes branch, creates pull request |
| `/review <#>` | Review work | Reads diff, checks security/correctness/style |
| `/dev` | Start coding | Launches server and/or desktop in dev mode |
| `/wip` | Switch devices | Saves and pushes work-in-progress |
| `/done` | End session | Updates milestones, changelog, captures observations |
| `/deploy` | Ship to prod | Runs checks, packages, deploys with confirmation |
| `/audit` | Monday | Security, compliance, drift, dependency scan |
| `/help` | Anytime | Shows the cheat sheet |

### Daily Rhythm

```
Morning:   /focus → /sync → /next       Pick up where you left off
Work:      /fix <#>, /check, /pr        Implement, verify, ship
Evening:   /done                         Record progress, observations
Monday:    /audit                        Weekly health check
```

### Agent-Specific Notes

**Claude Code** — use slash commands directly. Skills live in `.claude/skills/`.

**Codex** — ask for workflows by name: "focus", "fix issue 42", "review PR 17". Reads `CODEX.md` → `AGENTS.md` → `kb/` as needed.

**Gemini** — uses native tools + shared scripts. Can invoke sub-agents for deep analysis. Reads `GEMINI.md` → `AGENTS.md` → `kb/` as needed.

All three agents follow the same rules: `unsafe` is forbidden, agents emit proposals not facts, `just lint` before done.

## Project Structure

```
kb/                          Obsidian vault — the documentation (read first)
crates/
  governance-kernel/         Domain model + in-memory store
  governance-truths/         Truth catalog + Converge bindings
  governance-server/         Local HTTP harness + truth executors
  governance-app/            Shared Rust app layer (desktop + server)
apps/
  desktop/                   Svelte + Tauri shell
examples/
  vendor-selection/          Gherkin + JSON input specs
  dynamic-due-diligence/     Research loop example
  policy-vendor-commitment/  Cedar policy gates
  audit-vendor-decision/     Trust pack audit trail
```

## Local API

```
GET  /health                       Health check
GET  /v1/truths                    List available truths
POST /v1/truths/{key}/execute      Execute a truth
GET  /v1/decisions                 Query decisions
GET  /v1/vendors                   Query vendors
GET  /v1/audit                     Query audit trail
```

## Knowledge Base

The `kb/` directory is an Obsidian vault. Open it in Obsidian or read the markdown files directly. Start with `kb/Home.md`.

**Reading order for participants/teams:**
1. `kb/Architecture/Overview.md` — the five-layer model
2. `kb/Domain/Vendor Selection.md` — the challenge
3. `kb/Domain/Agents.md` — what each agent does
4. `kb/Development/Writing Suggestors.md` — how to write your own
5. `kb/Architecture/Convergence Loop.md` — how the engine converges

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
