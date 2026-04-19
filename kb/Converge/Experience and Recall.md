---
tags: [converge]
---
# Experience and Recall

Converge in this repo currently has three explicit observation layers:

- **Audit**: authoritative compliance records that can be replayed, signed off, and shown to auditors.
- **Log / telemetry**: runtime, optional observability for LLM calls.
- **Experience**: potential reusable learning stream (planned), not yet a separate durable store.

## What is Audit

Audit is the governance substrate and is stored in `governance-kernel`.

- The core store holds a durable `audit_trail: Vec<AuditEntry>` in `GovernanceKernel`.
- Kernel mutators (`register_vendor`, `record_*`, `record_decision`, etc.) call `audit(...)` on every mutation.
- `AuditEntry` is `action + actor + details + timestamp` and is always stored with the run state.
- The HTTP API exposes it at `GET /v1/audit`.
- This is the source of truth for trust/review questions (who did what and when).

Implementation anchors:

- `crates/governance-kernel/src/lib.rs` (audit trail, domain mutator hooks, `recent_audit`)
- `crates/governance-server/src/main.rs` (`/v1/audit`)
- `crates/governance-app/src/lib.rs` (`list_audit`)

## What is Log / Telemetry

Telemetry is not used for facts or authority decisions; it is a **runtime projection**.

- Shared types and sink abstraction live in `crates/governance-telemetry`:
  - `LlmCallTelemetry`
  - `LlmUsageSummary`
  - `LlmCallSink`
  - `InMemoryLlmCallCollector` and `NoopLlmCallSink`
- `governance-server` and the desktop DD path both record calls by emitting into a sink.
- `truth_runtime::TruthExecutionResult` still exposes `llm_calls: Option<Vec<LlmCallTelemetry>>` for compatibility.
- No raw prompts, tool payloads, or provider internals are persisted here; values are intentionally operational (`context`, `provider`, `model`, `usage`, `elapsed_ms`, `finish_reason`).
- `persist_projection=false` can skip persistence while still allowing telemetry capture for a single execution.

## What is Experience Store (Status)

An explicit experience store is not yet wired as a standalone substrate.

- `GovernanceKernel` currently models domain events in `pending_events` (`DomainEvent`) and exposes `drain_events()`.
- This gives us a clean migration path toward `converge-experience` without changing truth logic.
- Planned step:
  - map `DomainEvent` values to `ExperienceEvent` equivalents,
  - append them through an adapter in the truth runtime execution boundary,
  - keep `AuditEntry` as the authoritative legal log and move recommendation memory into the experience layer.

## Kong-free implementation boundary

The governance boundary is independent of Kong.

- All agent-facing code stays on `converge-provider-api` contracts (`ChatBackend`, `ChatRequest`) and never depends on Kong internals.
- Kong is an optional routing transport selected at runtime by environment config.
- In this repo, removing or omitting Kong credentials gives the same core behavior with direct provider backends.
- This keeps audit/telemetry semantics identical whether calls pass through Kong or direct APIs.

## Hackathon-only surface

The `governance-server` endpoint + durable adapter wiring in this repo is intentionally experimental.
It is a local harness for the `../runway` style integration, not a claim about permanent authority
architecture. Keep this behavior scoped as a hackathon convenience until a dedicated integration layer
is introduced elsewhere.

For now, treat:
- `DomainEventStream` adapter composition, and
- the new endpoint/system test proving adapter persistence
as *temporary integration plumbing* used for this project only.

See also:
- [[Converge/Building Blocks]], [[Converge/Core Concepts]]
- [[Integrations/Kong Gateway]], [[Integrations/Why Kong]], [[Development/Provider Configuration]]
