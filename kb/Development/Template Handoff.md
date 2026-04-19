---
tags: [handoff, template, release]
---

# Template Handoff: Hackathon vs Stable Surface

This page captures the recommended split so the repo can be treated as a **disposable
participant template** while system-level experiments stay in a dedicated integration repo.

## Decision

- The hackathon repo remains the starting template participants clone.
- Converge / Organism / Axiom behavior should point at stable released versions
  (GitHub release tags or crates.io).
- **System integration work** (server hardening, durable adapter harness, experimental
  integration flows) should be separated so participants can ignore it unless they are
  working the runway/system track.

## Keep in the Template (participant-safe)

- Core challenge flow and participant tasks:
  - truth definitions and `governance-truths` examples
  - Cedar policy experiments
  - suggestor/runtime scaffolding on canonical API surfaces (`converge-pack`, `converge-kernel`, `converge-provider`, `converge-axiom`, `organism-runtime`)
  - desktop starter and onboarding docs
  - lightweight local API to run truths
- Any API that is part of the participant exercise should remain documented as
  `kb/Converge/` and `kb/Development/` content.

## Move out to a System/Runway repo

The following are useful, but should be treated as **system-level experimentation**:

- `crates/governance-server/src/http_api.rs`
  - server router extraction for injectable state/adapter composition
- `crates/governance-server/tests/durable_adapter_system.rs`
  - adapter-driven endpoint persistence integration tests
- `crates/governance-server/src/experience.rs`
  - in-memory experience adapter implementation used by current harness wiring
- `kb/Converge/Experience and Recall.md`
  - section `Hackathon-only surface` that marks this pattern as experimental

If these are needed later, re-home as a dedicated module in the system repo
without removing the participant-facing template scaffold.

## Dependency strategy for release

1. Cut release tags in:
   - `converge`
   - `organism`
   - `axiom`
2. In template `Cargo.toml`, switch from path / workspace-local pointers to
   versioned dependencies.
3. Keep local override path only behind an optional `.cargo/config.toml` override
   for internal development.
4. Add a short “release matrix” section to the template README with exact versions
   expected by participants.

## Suggested handoff packaging

- Freeze a **template commit** and create a branch/tag participants can clone from.
- Keep an explicit integration branch/tag for the system harness work.
- Publish a two-item checklist in the next change:
  - Template branch: runs `just test` and starts locally for participants.
  - System branch: runs adapter/integration tests and server-path durability checks.

## What is already complete

- endpoint + durable adapter pattern is implemented and test-covered (`governance-server`)
- Domain event stream concept exists in kernel write-path with fail-soft forwarding semantics
- handoff status is marked in `kb/Converge/Experience and Recall.md`

## Open follow-up owners

1) **Template lead:** remove or isolate system-only files into the runway repo.
2) **Release lead:** publish stable tags for converge / organism / axiom.
3) **Docs lead:** keep participant docs pointing to those stable versions only.

## Rationale

This split protects participants from integration churn while preserving all value
from the current work for future runway-level system development.
