---
source: mixed
---

# Desktop Naming — Axiom & Helm

## What is this thing?

Not an IDE. Not a dashboard. Not a workspace.

It is **a governed execution environment** — the place where intent becomes governed action. The interface between human intent and system behavior.

## The Layer Model

| Layer | Name | Role |
|-------|------|------|
| Control surface | **Helm** | What operators see — inspect, validate, act |
| Truth layer | **Axiom** | What must be true — contracts, validation, policy lens |
| Intelligence | Organism | How the system reasons — huddle, debate, gap-chasing |
| Governance | Converge | Whether proposals can become facts |
| Capability | Providers | External models, search, and tools |

For naming purposes, the product-facing story is still simple:
1. Operators sit in **Helm**
2. They define and validate truth contracts in **Axiom**
3. Organism forms the team to satisfy the contract
4. Converge runs the governed loop, promotes facts, and records evidence
5. The app owns product writeback and artifacts

## Why Axiom

- Deep, tied to truth and validation
- Feels foundational — a self-evident truth that requires no proof
- Not overused in product naming
- Fits the governance-as-code positioning perfectly

## Why Helm

- Instantly understandable — control, steering
- Works well in demos and presentations
- The surface participants interact with first
- "You're at the helm" — agency and responsibility

## Current Decision

**Helm** is the public-facing desktop name (slide deck, hackathon, participant-facing).
**Axiom** is the executable truth-and-policy specification layer underneath.
**Organism** is the named intelligence layer after Axiom, but it stays architectural rather than product-branded in the desktop.
**Converge** remains the governance substrate and should be named directly when explaining authority or promotion.

The full stack does not need to be exposed on day one. Participants sit in Helm. Axiom surfaces when we go deeper into authoring and validation. Organism and Converge surface when we explain how governed reasoning actually works.

## Open Questions

- Does Axiom become a named crate / module, or stay conceptual?
- Should the Tauri window title say "Helm" or "Converge Helm"?
- Should we ever surface Organism directly in the desktop UI, or keep it as architecture vocabulary only?
