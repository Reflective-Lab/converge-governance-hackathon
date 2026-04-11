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
| Agent runtime | Converge (`converge-core`, `converge-domain`) | Governed multi-agent convergence with promotion gates |
| LLM/API access | `converge-provider` via Kong | Single governed gateway for all external calls |
| Spec validation | `converge-tool` | Offline-first truth and Gherkin validation |
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

The only remote calls are outbound to Kong: LLM traffic, MCP tools, proxied business APIs. Everything else is local.

See `kb/Architecture/` for the full picture.

## Rules

These are not suggestions.

- No `unsafe` code. Ever.
- Use typed enums, not strings with semantics.
- Agents emit proposals, not direct facts — Converge promotes them.
- Every mutation needs an Actor.
- `just lint` clean before considering work done.
- No feature flags. No backwards-compat shims. Change the code.
- No unnecessary abstractions. Three similar lines beat a premature helper.
- If a real service is unavailable, mock it behind the same trait boundary. Don't hardcode data into agents.

## Converge

Use it. This project exists to demonstrate governed multi-agent convergence.

All crates at **v2.1.2** on crates.io.

| Crate | What it gives you | Status |
|---|---|---|
| `converge-core` | Engine, Agent, Fact, Context, promotion gates, convergence loop, HITL gates, budgets | In use |
| `converge-domain` | Pre-built packs: trust, money, delivery, knowledge, data_metrics | In use |
| `converge-provider` | KongGateway, KongRoute, LlmProvider, McpClient | In use (desktop) |
| `converge-tool` | Spec validation, Gherkin parsing, truth-spec parsing | In use (desktop) |
| `converge-experience` | Experience tracking and recall across runs | Available |
| `converge-knowledge` | Knowledge management, signal capture, canonical decisions | Available |
| `converge-mcp` | MCP server/client for tool-based agent access | Available |
| `converge-optimization` | Multi-criteria optimization via OR-Tools | Available |

When you need a capability that doesn't exist in Converge, say so. We patch Converge. We don't work around it.

See `kb/Converge/Crate Catalog.md` for the full catalog with usage guidance.

## How to Add a New Truth

1. Define in `governance-truths/src/lib.rs` (key, packs, criteria)
2. Create `governance-server/src/truth_runtime/your_truth.rs`
3. Write agents implementing `converge_core::Agent`
4. Write a criterion evaluator implementing `CriterionEvaluator`
5. Wire in `truth_runtime/mod.rs` dispatcher
6. Add domain types to `governance-kernel` if needed
7. Update `kb/Domain/Truths.md`
8. `just test` green, `just lint` clean

## Integration

All external access goes through Kong. No direct API calls to OpenAI, Anthropic, or any other provider.

```rust
let gateway = KongGateway::from_env()?;
let llm = gateway.llm_provider(route);
```

See `kb/Integrations/Kong Gateway.md`.

## Workflows

Run `just focus` at session start. See `kb/Workflow/Daily Journey.md` for the full cheat sheet.

Workflow commands are available as Claude skills (`/focus`, `/sync`, etc.), `just` recipes (`just focus`, `just sync`, `just status`), or Gemini intents.

| Workflow | Purpose |
|---|---|
| `/focus` / `just focus` | Session opener — orient yourself, see team activity |
| `/sync` / `just sync` | Team sync — who did what, PRs waiting, unclaimed issues |
| `/status` / `just status` | Build health, test results |
| `/checkpoint` | End-of-session — what you moved, what's left for the team |
| `/dev` | Start local dev environment |
| `/fix` | Fix a GitHub issue by number |
| `/ticket` | Create an issue any teammate can pick up |
| `/parallel` | Run tasks in parallel worktrees (PRs need team review) |
| `/review` | Review a pull request |
| `/pr` | Create a pull request |
| `/merge` | Squash-merge a PR |
| `/quality` | Code quality metrics and trends |
| `/audit` | Security and dependency audit |
| `/wip` | Save work-in-progress and push |
| `/feedback` | Turn observations into GitHub issues |

Agent-specific workflow details: `kb/Workflow/Working with Claude.md`, `kb/Workflow/Working with Codex.md`, `kb/Workflow/Working with Gemini.md`.

