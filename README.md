# Converge Governance Hackathon Prep

This repository is a prep directory and starter kit for the hackathon. It is not meant to be a finished product. It gives the team a governed baseline for building enterprise AI workflows where multiple agents can analyze a problem, propose evidence and converge on a decision that is traceable and auditable.

The default challenge in this repo is AI vendor selection, but the real point is broader: use Converge to build decision systems that can justify what they did, stop honestly when confidence is too low and leave behind an audit trail that a business owner or auditor can inspect.

**Sponsors:** Kong · Vivicta
**Challenge contributor:** Reflective Labs

## How to Use This Template

This repo is a GitHub template. Do not clone it directly — use the green **"Use this template"** button on GitHub to create your own copy. That gives your team a clean repo with the full starter kit and no upstream link.

Once you have your own repo:

1. Each team member clones the team repo.
2. Work on branches and merge through pull requests, or push to main — your repo, your rules.
3. Nothing flows back to this template unless you explicitly open a PR here.

## What This Repo Should Do

This repo should help a team get from zero to a working governed demo quickly:

- Start from a running Rust workspace with a reference vendor-selection truth and a shared app core that can be embedded in a desktop app.
- Show how Converge is used as the multi-agent runtime, not as an optional add-on.
- Encourage a strongly opinionated implementation style: Rust for as much of the system as possible, including backend logic, agent orchestration, policy evaluation, projections, and shared application code.
- Prepare for a self-contained desktop operator experience built with Svelte and Tauri.
- Keep the app local-first: the desktop UI talks to the Rust core locally, and remote calls stay behind capability adapters in the Rust layer. Kong is useful when present, but direct provider wiring is acceptable for now.

Today, the repo already contains the Rust workspace, the reference `evaluate-vendor` truth, a shared app layer, and a simple server harness for local development. Teams are expected to extend it during the hackathon with real agents, real integrations, and the Svelte/Tauri shell in `apps/desktop/`.

## How This Depends On Converge

This project depends directly on Converge. It is not just inspired by Converge patterns; it is built on the Converge runtime and authoring crates:

- `converge-pack`
- `converge-kernel`
- `converge-domain`

Those dependencies are pulled from crates.io, which makes this repo a thin application layer on top of Converge.

Converge provides the core mechanics:

- Agents read shared context and propose facts.
- Facts go through the promotion gate before they become part of shared state.
- Criteria decide whether the run has actually succeeded.
- The engine stops honestly when it converges, blocks, or runs out of budget.

This repo adds the hackathon-specific pieces on top of that foundation:

- Governance domain objects and audit records
- Truth definitions and criterion evaluators
- Reference executors for challenge flows
- A shared app layer for the eventual desktop UX
- A lightweight local server harness for development and testing

If Converge is removed, the main execution model of this repo disappears with it. That dependency should be explicit to every team.

## Canonical Programming Surface

This repo family has been aligned to the current curated surfaces. `converge` presents `converge-pack` / `converge-kernel` and `ChatBackend` as the stable developer-facing shape, and this repo now uses the same shape in its reference truth and desktop path.

Students should learn this surface first:

- Author suggestors with `converge-pack`
- Embed the runtime with `converge-kernel`
- Keep LLM calls on `ChatBackend` + `ChatRequest`
- Use `converge-tool::mock_llm::StaticChatBackend` for offline validation and tests
- Use `organism-pack` for `IntentPacket`, `Plan`, and reasoning primitives when crossing into Organism
- Use `organism-runtime::Registry::with_standard_packs()` when you need built-in Organism packs

Kong is still a useful operational path for the hackathon, but it is not a requirement right now. It should stay below the same capability contract instead of becoming a second programming model students have to learn.

See [kb/Development/Programming API Surfaces.md](kb/Development/Programming%20API%20Surfaces.md) for the canonical import and layering rules.

## Opinionated Stack

This starter is intentionally opinionated.

### Rust First

Use Rust for as much of the solution as possible:

- Agent implementations
- Policy and rule evaluation
- Decision projection and audit logging
- Local application services and integration adapters
- Shared application logic used by both server and desktop layers
- Local mocks for business services when the real services are unavailable

The goal is not to force Rust everywhere for ideological reasons. The goal is to keep the critical logic, traceability, and integration behavior in one strongly typed runtime.

### Desktop Frontend: Svelte + Tauri

The preferred frontend shape is:

- **Svelte** for the operator UI
- **Tauri** for packaging that UI as a desktop application

That gives teams a lightweight desktop shell while keeping most non-UI logic in Rust. The repo now includes a minimal Bun + Svelte + Tauri scaffold centered on validating vendor-selection Gherkin locally before the fuller execution flow is added, and it continues to rely on the shared Rust application layer for the deeper workflow logic.

## Local-First App Flow

The intended runtime shape is:

1. A user opens a Gherkin file or truth-spec JSON file in the desktop app.
2. The Svelte/Tauri shell passes that file into the local Rust app layer.
3. The Rust app normalizes the input into the `evaluate-vendor` truth and runs Converge locally.
4. Agents only make outbound calls when they need external model or business context.
5. Those outbound calls go to Kong and then on to the configured LLM or business-service integrations.

The repo now includes example vendor-selection inputs:

- [vendor-selection.feature](examples/vendor-selection/vendor-selection.feature)
- [vendor-selection.truths.json](examples/vendor-selection/vendor-selection.truths.json)

The shared Rust app layer can preview or execute either format, which is the boundary a Tauri app should use.

## Kong: Optional Remote Integration

This is a self-contained app. The UI should not call a remote governance backend by default. Remote traffic should originate in the Rust core and may go through Kong or direct provider and service adapters for now.

Use Kong when it adds value for:

- **LLM traffic**: prompts, completions, token usage, rate limiting, cost tracking, and guardrails
- **MCP tools**: business-service access exposed through Model Context Protocol
- **Standard APIs**: vendor data, policy data, procurement systems, compliance registries, and similar enterprise services

That means the intended pattern is:

1. Agents run inside the local Rust application and Converge runtime.
2. When an agent needs model reasoning, it should stay on `ChatBackend`. Kong routing is optional for now.
3. When an agent needs business context, MCP and service adapters are preferred, and Kong can front them later.
4. If real business services are not available during the hackathon, mock them locally behind the same capability contracts.

This keeps the programming model realistic: one capability surface for model and tool access, with routing choices hidden underneath.

Keep the application-facing API on the canonical Converge capability surface. In other words: present `ChatBackend` / `ChatRequest` to students and keep any Kong-specific routing or credential handling behind that adapter.

Future direction: add a `KongProvider` or more general `RouterProvider` under that same capability surface, with Kong especially valuable when it can bridge shared MCP tools.

The desktop scaffold now follows that contract: the Tauri edge selects a live `ChatBackend`, app code builds `ChatRequest`, and offline validation uses `converge-tool::StaticChatBackend`.

Kong-backed `.env` if you are using Kong:

```dotenv
KONG_AI_GATEWAY_URL=https://<provided-at-hackathon>
KONG_API_KEY=<your-team-key>
```

Desktop-specific optional settings:

```dotenv
KONG_LLM_ROUTE=default
KONG_LLM_UPSTREAM_PROVIDER=openai
KONG_LLM_UPSTREAM_MODEL=gpt-4
KONG_LLM_REASONING=true
```

Direct provider keys are also acceptable during the current transition, as long as application code stays on the same capability contracts.

## Mocking Business Services

Some teams will not have access to real enterprise systems during the event. That should not block the architecture.

A good hackathon approach is:

- Mock procurement, policy, vendor, or compliance services locally
- Expose those mocks with stable contracts
- Put them behind Kong
- If tool-style access is useful, expose them through MCP and let agents call them that way

Examples of mock services that fit this repo:

- A vendor profile service with certifications, regions, and pricing plans
- A policy service that returns internal guardrails for allowed AI usage
- A procurement approval service that simulates budget thresholds or escalation rules
- A compliance evidence service that returns structured documents for screening agents

## Reference Challenge

The reference truth is `evaluate-vendor`. It is the primary use case for this prep repo. It demonstrates a governed workflow for choosing an AI vendor in a way that is defensible to auditors and stakeholders.

The intended multi-agent flow is:

- A compliance agent screens vendors against policy and regulation
- A cost agent estimates operating cost
- A capability agent compares vendor fit against requirements
- A risk agent scores operational and strategic risk
- A synthesis agent recommends an outcome or asks for human review

Every agent contributes evidence. Converge decides when the workflow has enough evidence to converge.

There is also an advanced dynamic example inspired by Monterro:

- `dynamic-due-diligence`
- [examples/dynamic-due-diligence/README.md](examples/dynamic-due-diligence/README.md)

That truth shows the research shape students are likely to want for richer demos:
Organism seeds breadth and depth strategies, Converge drives a dynamic evidence loop, contradictions are made explicit, and the final brief is returned as structured projection data.

There is also a focused policy example for the last mile of the business flow:

- [examples/policy-vendor-commitment/README.md](examples/policy-vendor-commitment/README.md)

That example shows how `converge-policy` is used to authorize, escalate, or
reject a vendor commitment before procurement acts on a recommendation.

## Current Repo Structure

```text
kb/                      Obsidian knowledgebase — THE documentation
.claude/skills/          Claude Code slash commands for team workflow
scripts/workflow/        Shared workflow helpers for Claude, Codex, and terminal users

apps/
  desktop/               Svelte + Tauri shell to be built during the hackathon

crates/
  governance-kernel/     Domain model and in-memory store
  governance-truths/     Truth catalog, criteria, Converge bindings
  governance-server/     Local harness and truth executors
  governance-app/        Shared Rust app layer for the desktop shell

examples/                Sample vendor-selection, due-diligence, and policy request files
```

The reference implementation lives in [evaluate_vendor.rs](crates/governance-server/src/truth_runtime/evaluate_vendor.rs). It currently uses placeholder agents so teams can focus on replacing them with real logic.

## Before You Get Started

You need Rust (1.94+). Install it from [rustup.rs](https://rustup.rs) if you don't have it.

Once you have Rust, install the tools you need with cargo:

```bash
cargo install just
cargo install tauri-cli    # only needed for desktop work
```

For the desktop app you also need [Bun](https://bun.sh):

```bash
curl -fsSL https://bun.sh/install | bash
```

## Getting Started

```bash
git clone <this-repo>
cd converge-governance-hackathon
cp .env.example .env   # edit with your team's Kong credentials

just hit-the-ground-running
```

This builds the workspace, runs all tests, and checks lint. If it passes, you are ready to go.

After that:

```bash
just server             # start the local harness (http://localhost:8080)
just install-desktop    # install desktop frontend dependencies
just dev-desktop        # run the desktop app in dev mode
```

`just server` is only a local harness for exercising the runtime while the Tauri shell is still being built.

To start fresh:

```bash
just clean              # delete all build artifacts
```

Desktop packaging commands are also prepared in the top-level `Justfile`:

- `just install-desktop`
- `just dev-desktop`
- `just build-desktop`
- `just package-desktop`
- `just deploy`

The desktop commands use Bun as the frontend package manager and task runner.
The desktop app is configured to build for macOS, Windows, and Linux. Tauri produces native binaries, so you can only build for the OS you are running on.

Tauri 2 also supports iOS and Android, but mobile builds require additional setup (Xcode for iOS, Android SDK for Android) and are not in scope for this hackathon. If someone wants to pursue it, the Tauri mobile docs cover what is needed.

Then exercise the reference truth:

```bash
curl -X POST http://localhost:8080/v1/truths/evaluate-vendor/execute \
  -H 'Content-Type: application/json' \
  -d '{"inputs": {"vendors": "Acme AI, Beta ML, Gamma LLM"}}'
```

Or run the advanced dynamic due-diligence example:

```bash
curl -X POST http://localhost:8080/v1/truths/dynamic-due-diligence/execute \
  -H 'Content-Type: application/json' \
  -d @examples/dynamic-due-diligence/dynamic-due-diligence.request.json
```

## What Teams Should Build

- Replace placeholder agents with real logic
- Make the desktop shell able to load local `.feature` and `.truths.json` vendor-selection files
- Add more packs and criteria where the use case needs them
- Keep all model calls on `ChatBackend` and all search calls on the provider capability surface
- Use Kong when it helps with routing, observability, or shared MCP tool access
- Add business-service access through MCP or service contracts that can later be routed through Kong
- Mock missing enterprise services locally instead of hardcoding everything into agents
- Build a desktop operator experience with Svelte and Tauri on top of the Rust layers

## Local Harness API

```text
GET  /health
GET  /v1/truths
POST /v1/truths/{key}/execute
GET  /v1/decisions
GET  /v1/vendors
GET  /v1/audit
```

## Documentation

The canonical documentation lives in `kb/` — an Obsidian vault that humans open in Obsidian and AI agents read with file tools. Start with `kb/Home.md`.

| Section | What's there |
|---|---|
| `kb/Architecture/` | System design, layers, convergence loop |
| `kb/Domain/` | Vendor selection challenge, agents, truths, types |
| `kb/Development/` | Getting started, writing agents, writing truths |
| `kb/Integrations/` | Kong, MCP, external services |
| `kb/Converge/` | Platform concepts, building blocks, domain packs |
| `kb/Workflow/` | Claude guide, Codex guide, daily journey cheat sheet |

## Agent Workflows

This project supports both Claude Code and Codex.

- Claude users can use the 14 project skills in `.claude/skills/`. Start with `/focus`, use `/fix` or `/check` while working, and close with `/done`.
- Codex users should start with `AGENTS.md`, then read `CODEX.md`. The detailed workflow mapping lives in `kb/Workflow/Working with Codex.md`.
- Shared deterministic repo-state checks are available to everyone as `just focus`, `just sync`, and `just status`.
- GitHub issue and PR workflows assume `gh` is authenticated. Local repo-health workflows still work when GitHub is unavailable.

See `CLAUDE.md`, `CODEX.md`, `kb/Workflow/Working with Claude.md`, `kb/Workflow/Working with Codex.md`, `kb/Workflow/Daily Journey.md`, and `kb/Workflow/Skills Reference.md`.
