---
tags: [converge]
---
# Converge Core Concepts

Converge is a correctness-first multi-agent runtime. Instead of letting agents do whatever they want, every action goes through a governance model.

## The Model

```
Suggestor proposes a fact
  → Promotion gate validates it (authority, schema, confidence)
  → Fact is promoted into the shared context
  → Other suggestors read the updated context
  → Engine runs cycles until criteria are met or budget is exhausted
```

## Key Properties

- **No agent can bypass governance.** Facts have private constructors — you can't create one without going through the promotion gate.
- **Every fact is traceable.** Who proposed it, when, with what confidence, and what authority.
- **Convergence is observable.** The criterion evaluator tells you exactly which success conditions were met or unmet.
- **Stopping is honest.** If the system can't converge, it tells you why (budget exhausted, criteria blocked, human intervention required).

## Dependencies

This repo uses:
- `converge-core` 3.0.0 — Engine, Suggestor, Fact, Context, promotion gate, convergence loop
- `converge-domain` 3.0.0 — pre-built [[Converge/Domain Packs|domain packs]] (trust, money, delivery, knowledge, data_metrics)
- `converge-provider` 3.0.0 — [[Integrations/Kong Gateway|Kong]] integration for LLM and API access
- `converge-tool` 3.0.0 — spec validation (used in desktop app)

> **TODO:** Scan converge crate sources for deeper documentation of internals.

See also: [[Converge/Building Blocks]], [[Architecture/Overview]]
