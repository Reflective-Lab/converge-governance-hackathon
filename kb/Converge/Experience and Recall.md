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
