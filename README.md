# Converge Governance Hackathon Prep

This repository is a prep directory and starter kit for the hackathon. It is not meant to be a finished product. It gives the team a governed baseline for building enterprise AI workflows where multiple agents can analyze a problem, propose evidence and converge on a decision that is traceable and auditable.

The default challenge in this repo is AI vendor selection, but the real point is broader: use Converge to build decision systems that can justify what they did, stop honestly when confidence is too low and leave behind an audit trail that a business owner or auditor can inspect.

**Sponsors:** Kong · Vivicta
**Challenge contributor:** Reflective Labs

## What This Repo Should Do

This repo should help a team get from zero to a working governed demo quickly:

- Start from a running Rust workspace with a reference vendor-selection truth and a shared app core that can be embedded in a desktop app.
- Show how Converge is used as the multi-agent runtime, not as an optional add-on.
- Encourage a strongly opinionated implementation style: Rust for as much of the system as possible, including backend logic, agent orchestration, policy evaluation, projections, and shared application code.
- Prepare for a self-contained desktop operator experience built with Svelte and Tauri.
- Keep the app local-first: the desktop UI talks to the Rust core locally, and the only remote calls go outward to Kong and the LLM providers behind it.

Today, the repo already contains the Rust workspace, the reference `evaluate-vendor` truth, a shared app layer, and a simple server harness for local development. Teams are expected to extend it during the hackathon with real agents, real integrations, and the Svelte/Tauri shell in `apps/desktop/`.

## How This Depends On Converge

This project depends directly on Converge. It is not just inspired by Converge patterns; it is built on the Converge runtime crates:

- `converge-core`
- `converge-domain`

Those dependencies are currently wired as local path dependencies in [Cargo.toml](/Users/kpernyer/dev/work/converge-governance-hackathon/Cargo.toml), which makes this repo a thin application layer on top of Converge.

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

- [vendor-selection.feature](/Users/kpernyer/dev/work/converge-governance-hackathon/examples/vendor-selection/vendor-selection.feature)
- [vendor-selection.truths.json](/Users/kpernyer/dev/work/converge-governance-hackathon/examples/vendor-selection/vendor-selection.truths.json)

The shared Rust app layer can preview or execute either format, which is the boundary a Tauri app should use.

## Kong: Only Outbound Remote Integration

This is a self-contained app. The UI should not call a remote governance backend by default. The only remote traffic should be outbound calls from the Rust core to Kong and the services it fronts.

Use Kong for:

- **LLM traffic**: prompts, completions, token usage, rate limiting, cost tracking, and guardrails
- **MCP tools**: business-service access exposed through Model Context Protocol
- **Standard APIs**: vendor data, policy data, procurement systems, compliance registries, and similar enterprise services

That means the intended pattern is:

1. Agents run inside the local Rust application and Converge runtime.
2. When an agent needs model reasoning, it calls the LLM through Kong.
3. When an agent needs business context, it uses whatever Kong-exposed API or MCP contract the Kong team defines.
4. If real business services are not available during the hackathon, mock them locally and expose them through the same Kong-facing adapter shape.

This keeps the demo realistic: one governed path for both model access and tool access.

Use `converge-provider` as the default Kong adapter. Do not hand-roll Kong HTTP calls in app code unless you are doing something the provider crate does not support yet.

The current desktop app follows this pattern:

1. Load `.env` in the Tauri layer.
2. Read `KONG_AI_GATEWAY_URL` and `KONG_API_KEY`.
3. Create `KongGateway::from_env()`.
4. Build a `KongRoute` for the LLM use case.
5. Call `gateway.llm_provider(route)` for guided validation or rewrite flows.

Minimal `.env`:

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

## Current Repo Structure

```text
apps/
  desktop/               Svelte + Tauri shell to be built during the hackathon

crates/
  governance-kernel/     Domain model and in-memory store
  governance-truths/     Truth catalog, criteria, Converge bindings
  governance-server/     Local harness and truth executors
  governance-app/        Shared Rust app layer for the desktop shell

examples/                Sample vendor-selection Gherkin and truth files
docs/                    Architecture and Kong guidance
```

The reference implementation lives in [evaluate_vendor.rs](/Users/kpernyer/dev/work/converge-governance-hackathon/crates/governance-server/src/truth_runtime/evaluate_vendor.rs). It currently uses placeholder agents so teams can focus on replacing them with real logic.

## Getting Started

```bash
git clone <this-repo>
cd converge-governance-hackathon

just test
just build
just install-desktop
just dev-desktop
just server
```

`just server` is only a local harness for exercising the runtime while the Tauri shell is still being built.

Desktop packaging commands are also prepared in the top-level `Justfile`:

- `just install-desktop`
- `just dev-desktop`
- `just build-desktop`
- `just package-desktop`
- `just deploy`

The desktop commands use Bun as the frontend package manager and task runner.
`just package-desktop` and `just deploy` currently target the macOS `.app` bundle path, which succeeds locally without the extra DMG bundling step.

Then exercise the reference truth:

```bash
curl -X POST http://localhost:8080/v1/truths/evaluate-vendor/execute \
  -H 'Content-Type: application/json' \
  -d '{"inputs": {"vendors": "Acme AI, Beta ML, Gamma LLM"}}'
```

## What Teams Should Build

- Replace placeholder agents with real logic
- Make the desktop shell able to load local `.feature` and `.truths.json` vendor-selection files
- Add more packs and criteria where the use case needs them
- Push all LLM calls through Kong
- Add business-service access through the Kong-facing contracts the platform team provides
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

## Supporting Docs

- [Architecture](/Users/kpernyer/dev/work/converge-governance-hackathon/docs/architecture.md)
- [Kong Integration](/Users/kpernyer/dev/work/converge-governance-hackathon/docs/kong-integration.md)
- [Vendor Selection Challenge](/Users/kpernyer/dev/work/converge-governance-hackathon/docs/01-vendor-selection.md)
