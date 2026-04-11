---
tags: [converge]
---
# Governed Artifacts

`converge_core::governed_artifact` provides lifecycle management for system outputs.

## State Machine

```
Draft → UnderReview → Approved → Active
Active → Suspended → Retired
Any state → RolledBack (with severity and impact tracking)
```

The state machine enforces valid transitions and tracks who changed what.

## When to Use

Use when suggestor outputs become operational artifacts:

- Approved vendor lists
- Policy documents
- Compliance certificates
- Decision records that need formal lifecycle management

See also: [[Converge/Core Concepts]], [[Converge/Building Blocks]]
