---
tags: [converge, hitl]
---
# Human-in-the-Loop Gates

Converge pauses convergence when a decision is too sensitive for full automation.

## How Gating Works

```rust
use converge_core::gates::hitl::{HitlPolicy, GateDecision, GateVerdict, TimeoutPolicy};

let hitl = HitlPolicy::gate_all()
    .with_timeout(TimeoutPolicy {
        duration_secs: 3600,
        action: TimeoutAction::Reject,
    });
```

When a gated proposal arrives, the engine pauses and emits a `GateRequest`. Your app presents it to a reviewer and collects a `GateDecision`:

```rust
GateDecision {
    verdict: GateVerdict::Approved,
    reason: "CFO reviewed and approved vendor selection".into(),
    reviewer: "jane.doe@company.com".into(),
}
```

The engine resumes with `engine.resume()`.

## Criterion-Level Blocking

Signal that a criterion itself needs human input:

```rust
CriterionResult::Blocked {
    reason: "requires procurement approval above $50k".into(),
    approval_ref: Some("PROC-2026-0412".into()),
}
```

## Integration Patterns

### 1. Desktop UI (recommended default)
Engine pauses → GateRequest sent to Tauri frontend → Svelte renders review panel → operator approves/rejects → engine resumes. Simplest, works offline.

### 2. Slack
Engine pauses → Block Kit message posted to channel → reviewer clicks button → interaction webhook fires → engine resumes. Good for team visibility and async review.

### 3. Email
Engine pauses → email sent with approve/reject link → link hits endpoint → engine resumes. Slowest. Use for escalation, not primary review.

## Timeout Policies

- `TimeoutAction::Reject` — no review in time, proposal rejected. Safe default.
- `TimeoutAction::Approve` — no review in time, auto-approved. Only for low-risk proposals.

## Selective Gating

Not every proposal needs review. Common pattern:

- Auto-approve compliance screening (rule-based, low risk)
- Gate cost estimates above a threshold (financial impact)
- Always gate the final recommendation (high-stakes)

Filter in `HitlPolicy::requires_approval()` by checking proposal content, agent, or confidence score.

See also: [[Converge/Building Blocks]], [[Domain/Agents]]
