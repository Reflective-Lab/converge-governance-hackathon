---
tags: [converge]
---
# Context Keys

Facts are partitioned by `ContextKey`. Suggestors declare which keys they depend on — the engine only wakes suggestors when their dependencies change.

| Key | Purpose | Example facts |
|-----|---------|---------------|
| `Seeds` | Initial evidence, screening results | `compliance:screen:acme-ai` |
| `Hypotheses` | Tentative conclusions, intermediate analysis | `hypothesis:acme-best-fit` |
| `Evaluations` | Scored assessments, cost estimates, decisions | `cost:estimate:acme-ai`, `decision:recommendation` |
| `Corrections` | Revisions to earlier facts | `correction:cost:acme-ai` |
| `Metadata` | Run metadata, configuration | `meta:run-config` |

This partitioning keeps convergence efficient: agents don't re-run unless there's new information in a partition they care about.

See also: [[Converge/Building Blocks]], [[Architecture/Convergence Loop]]
