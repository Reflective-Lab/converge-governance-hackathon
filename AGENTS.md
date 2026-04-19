# Converge Governance Hackathon

This is an opinionated project. Read this file. Follow it.

This is the canonical agent entrypoint — all agents (Claude, Codex, Gemini, or otherwise) start here. Long-form documentation lives in `kb/`.

## Philosophy

We use strongly typed languages that compile to native code. Rust for the system. Svelte for the UI. Tauri for the desktop shell. No React. No virtual machines. No garbage collectors in the hot path.

This is not a rejection of those tools — they were necessary when humans had to comprehend complexity and master hard software engineering processes unaided. That era is ending. With AI agents as first-class collaborators, we can work directly with the metal: explicit ownership, zero-cost abstractions, compile-time guarantees. The compiler is the first reviewer. The type system is the first test suite.

Converge is the execution model. Agents propose facts. Facts are promoted through governance gates. Decisions converge or honestly stop. Every fact has provenance. Every decision has evidence.

## The Knowledgebase

`kb/` is an Obsidian vault. It is THE documentation for this project. Not the README. Not inline comments. Not a wiki somewhere. The knowledgebase.

- Humans open it in Obsidian.
- AI agents read it with file tools.
- It is a living system. When you learn something, update the kb.
- When architecture changes, the kb changes first.

**Do NOT read the entire kb on startup.** It is ~13,500 tokens across 25 pages. Lazy-load:

1. Read `kb/Home.md` only when you need to find something (it's the index).
2. Follow ONE wikilink to the specific page you need.
3. Read that page. If it links to something else you need, follow that link.
4. Never bulk-read `kb/` — treat it like documentation you look up, not a preamble you memorize.

Structure:
```
kb/
  Home.md                  Index — scan this to find the right page
  Architecture/            System design, layers, convergence loop
  Domain/                  Vendor selection, agents, truths, types
  Development/             Getting started, writing agents, writing truths
  Integrations/            Kong, MCP, external services
  Converge/                Platform concepts, building blocks, domain packs
  Workflow/                Skills vs Justfile, daily journey, skills reference
```

If you produce knowledge that doesn't belong in code, it belongs in the kb.

## Stack

| Layer | Technology | Why |
|---|---|---|
| System logic | Rust | Ownership, zero-cost abstractions, compile-time safety |
| Agent runtime | Converge (`converge-pack`, `converge-kernel`, `converge-domain`) | Governed multi-agent convergence with promotion gates |
| LLM/API access | `converge-provider-api` + `converge-provider`, with optional Kong routing | Canonical capability contract first, concrete adapters second |
| Spec validation | `converge-axiom` | Offline-first truth and Gherkin validation |
| Desktop shell | Tauri | Native performance, Rust backend, no Electron |
| UI | Svelte | Compiled, no virtual DOM, minimal runtime |
| Package manager | Bun | Fast, native, replaces npm/yarn/pnpm |
| Task runner | just | Simple, explicit, no magic |
| Version control | git | |

## Build

```bash
just test          # cargo test --workspace
just server        # cargo run -p governance-server (localhost:8080)
just lint          # cargo clippy --workspace
just fmt           # cargo fmt --all
just dev-desktop   # Tauri dev mode
just check         # cargo check --workspace (fast, no tests)
just focus         # Session opener — repo health + recent activity
just sync          # Team sync — PRs, issues, recent commits
just status        # Build health, test results
```

## Architecture

```
governance-server (HTTP API — dev harness, not product surface)
  └── truth_runtime/
        └── evaluate_vendor.rs    <- THE REFERENCE — study this first
              ├── ComplianceScreenerAgent
              ├── CostAnalysisAgent
              └── DecisionSynthesisAgent

governance-app (shared layer for desktop)
  └── GovernanceApp — view models, truth execution, queries

governance-kernel (domain model + in-memory store)
  └── Vendor, PolicyRule, ComplianceCheck, RiskScore, DecisionRecord, AuditEntry

governance-truths (truth catalog + converge bindings)
  └── TruthDefinition, build_intent(), EvaluateVendorEvaluator
```

The Svelte frontend calls local Rust commands through Tauri. Not HTTP. Not REST. Not gRPC. Local function calls across the Tauri bridge.

Remote calls may go through Kong or direct provider and service adapters for now. Keep application code on `ChatBackend`, `WebSearchBackend`, and MCP surfaces. Everything else is local.

See `kb/Architecture/` for the full picture.

## Rules

These are not suggestions.

- No `unsafe` code. Ever.
- Use typed enums, not strings with semantics.
- Suggestors emit proposals, not direct facts — Converge promotes them.
- Every mutation needs an Actor.
- `just lint` clean before considering work done.
- No feature flags. No backwards-compat shims. Change the code.
- No unnecessary abstractions. Three similar lines beat a premature helper.
- If a real service is unavailable, mock it behind the same trait boundary. Don't hardcode data into suggestors.

## Converge

Use it. This project exists to demonstrate governed multi-agent convergence.

Participant-facing guidance in this repo should track the current curated Converge surfaces, even if some internals still use lower-level crates.

| Crate | What it gives you | Status |
|---|---|---|
| `converge-pack` | Authoring contract: `Suggestor`, `AgentEffect`, `ProposedFact`, `ContextKey` | Preferred for new participant-facing examples |
| `converge-kernel` | In-process runtime API: `Engine`, `Context`, `Budget`, criteria, run hooks | Preferred for new participant-facing examples |
| `converge-provider-api` | Chat contracts and capability-routing vocabulary | Preferred for capability-facing code |
| `converge-core` | Constitutional types and re-exports used by current internals | Internal / transitional |
| `converge-domain` | Pre-built suggestor packs: trust, money, delivery, knowledge, data_metrics | In use |
| `converge-provider` | Chat backends, search adapters, tool clients, backend selection | In use |
| `converge-axiom` | Spec validation, Gherkin parsing, truth-spec parsing, mock chat backends | In use (desktop) |
| `converge-experience` | Experience tracking and recall across runs | Available |
| `converge-knowledge` | Knowledge management, signal capture, canonical decisions | Available |
| `converge-mcp` | MCP server/client for tool-based agent access | Available |
| `converge-optimization` | Multi-criteria optimization via OR-Tools | Available |

When you need a capability that doesn't exist in Converge, say so. We patch Converge. We don't work around it.

For cross-repo familiarity with Organism, present intent and planning through `organism-pack` and runtime wiring through `organism-runtime`, then let Converge handle promotion, evaluation, and commitment.

See `CAPABILITIES.md` for what this hackathon project provides to students. See `~/dev/work/converge/CAPABILITIES.md` for the full Converge capability catalog.

See `kb/Converge/Crate Catalog.md` for the full catalog with usage guidance.

## How to Add a New Truth

1. Define in `governance-truths/src/lib.rs` (key, packs, criteria)
2. Create `governance-server/src/truth_runtime/your_truth.rs`
3. Write suggestors implementing `converge_pack::Suggestor`
4. Write a criterion evaluator implementing `CriterionEvaluator`
5. Wire in `truth_runtime/mod.rs` dispatcher
6. Add domain types to `governance-kernel` if needed
7. Update `kb/Domain/Truths.md`
8. `just test` green, `just lint` clean

## Integration

Kong is optional for now. Direct provider and search adapters are acceptable while the capability contract is being hardened.

Present the programming boundary as `ChatBackend` / `ChatRequest` plus `WebSearchBackend` and MCP clients. Keep any Kong-specific routing behind that adapter layer. Do not add new participant-facing examples around `KongGateway` / `LlmRequest` as the primary import surface.

Future direction: add a `KongProvider` or more general `RouterProvider` under the same capability contracts, with Kong especially valuable for shared MCP tool access.

See `kb/Integrations/Kong Gateway.md`.

## Workflows

Run `just focus` at session start. See `kb/Workflow/Daily Journey.md` for the full cheat sheet.

Workflow commands are available as Claude skills (`/focus`, `/sync`, etc.), deterministic repo-state recipes (`just focus`, `just sync`, `just status`), or Gemini intents.

| Workflow | Purpose |
|---|---|
| `/focus` / `just focus` | Session opener — orient yourself, see team activity |
| `/sync` / `just sync` | Team sync — who did what, PRs waiting, unclaimed issues |
| `/next` | Pick next task from the current milestone |
| `/dev` | Start local dev environment |
| `/check` | Lint, compile check, tests |
| `/fix` | Fix a GitHub issue by number |
| `/pr` | Create a pull request |
| `/ticket` | Create an issue any teammate can pick up |
| `/done` | End-of-session — what you moved, what's left for the team |
| `/review` | Review a pull request |
| `/wip` | Save work-in-progress and push |
| `/deploy` | Deploy to target environment |
| `/audit` | Security, dependency, compliance, and drift audit |
| `/help` | Show available skills |

The daily habit:
```
Morning:    /focus → /sync → /next
Work:       /fix, /check, /pr
Evening:    /done
Monday:     /audit
Anytime:    /help
```

Agent-specific workflow details: `kb/Workflow/Working with Claude.md`, `kb/Workflow/Working with Codex.md`, `kb/Workflow/Working with Gemini.md`.

## Milestones

Read `MILESTONES.md` at the start of every session. Scope all work to the current milestone. See `~/dev/work/EPIC.md` for the strategic context.
