---
tags: [converge, domain-packs]
---
# Domain Packs

Pre-built agent packs from `converge-domain`. Ready to register and run — no custom agents needed.

## Available Packs

| Pack | What it does | Key agents |
|------|-------------|------------|
| `trust` | Audit trails, access control, provenance, compliance scanning | SessionValidatorAgent, RbacEnforcerAgent, AuditWriterAgent, ProvenanceTrackerAgent, ComplianceScannerAgent, ViolationRemediatorAgent, PiiRedactorAgent |
| `money` | Financial transactions, invoicing, reconciliation | InvoiceCreatorAgent, PaymentAllocatorAgent, ReconciliationMatcherAgent, PeriodCloserAgent |
| `delivery` | Promise fulfillment, scope tracking, blockers | PromiseCreatorAgent, WorkBreakdownAgent, BlockerDetectorAgent, RiskAssessorAgent, StatusAggregatorAgent |
| `knowledge` | Signal capture, hypothesis testing, experiments, canonical decisions | SignalCaptureAgent, HypothesisGeneratorAgent, ExperimentRunnerAgent, DecisionMemoAgent, CanonicalKnowledgeAgent |
| `data_metrics` | Metrics, dashboards, anomaly detection, alerting | MetricRegistrarAgent, DataValidatorAgent, DashboardBuilderAgent, AnomalyDetectorAgent, AlertEvaluatorAgent |

## Example: audit-vendor-decision

Uses the trust pack with zero custom agents:

```rust
let mut engine = Engine::new();
engine.register_in_pack("trust-pack", SessionValidatorAgent);
engine.register_in_pack("trust-pack", RbacEnforcerAgent);
engine.register_in_pack("trust-pack", AuditWriterAgent);
engine.register_in_pack("trust-pack", ProvenanceTrackerAgent);
engine.register_in_pack("trust-pack", ComplianceScannerAgent);
```

Convergence chain: validate session → enforce RBAC → write audit → track provenance → scan compliance → converged.

## Mixing Domain Packs with Custom Agents

```rust
// Custom agents
engine.register_in_pack("compliance-pack", ComplianceScreenerAgent { vendor_names });
engine.register_in_pack("cost-pack", CostAnalysisAgent);

// Domain pack agents (free audit trails)
engine.register_in_pack("trust-pack", AuditWriterAgent);
engine.register_in_pack("trust-pack", ProvenanceTrackerAgent);
```

Trust pack agents automatically pick up access decisions and audit the entire evaluation flow.

## Invariants

Domain packs also provide invariants checked during convergence:

- `AllActionsAuditedInvariant` — every access decision must have an audit entry
- `AuditImmutabilityInvariant` — audit entries must be marked immutable
- `ViolationsHaveRemediationInvariant` — open violations must have remediation plans

## When to Use What

- **Domain packs** for cross-cutting concerns: audit, access control, provenance
- **Custom agents** for business logic: vendor scoring, cost analysis, risk assessment
- **Both** in the same engine for maximum governance

See also: [[Domain/Truths]], [[Converge/Building Blocks]]
