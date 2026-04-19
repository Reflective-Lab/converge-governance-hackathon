---
tags: [converge]
---
# Experience and Recall

Converge tracks experience events during runs and lets agents query past experience.

## Tracking Experience

```rust
let observer = Arc::new(|event: &ExperienceEvent| {
    log::info!("experience: {:?}", event);
});

TypesRunHooks {
    criterion_evaluator: Some(Arc::new(evaluator)),
    event_observer: Some(observer),
}
```

## Recall System

The `converge_core::recall` module lets agents query past decisions:

| Type | Purpose |
|---|---|
| `RecallQuery` | Searches for relevant past decisions |
| `RecallCandidate` | Scores results by relevance |
| `RecallPolicy` | Controls what can be recalled and by whom |

A cost analysis agent can recall what similar vendors cost in past evaluations and use that as a baseline.

See also: [[Converge/Building Blocks]], [[Converge/Core Concepts]]

## Hackathon Telemetry Boundary

In this repo, telemetry is implemented as a runtime `llm_calls` projection, not yet a first-class experience stream.

- `governance-server` appends an optional `llm_calls: Option<Vec<LlmCallTelemetry>>` field to `TruthExecutionResult`.
- Desktop DD flow reports `llm_calls: Vec<LlmCallTelemetry>` in `DdReport` with per-call provider/model, timing, token usage, and finish reason.
- Existing audit trail (`/v1/audit`, decision records, and invariants) remains the authoritative compliance log.
- `LlmCallTelemetry` intentionally includes only redacted operational context values, not raw prompts, tool payloads, or provider internals.
- Next alignment step is to add a proper ExperienceEvent adapter so these call-level facts can be emitted into the same experience/audit substrate used by other Converge products.
