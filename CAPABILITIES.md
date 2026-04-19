# Capabilities

What this project gives hackathon participants.

## Governance Loop

- **Proposal protocol** — agents emit proposals, never direct facts
- **Promotion gates** — proposals validated before becoming facts
- **Cedar policy evaluation** — Amazon Cedar authorizer decides who can do what
- **Convergence** — engine iterates until fixed-point or honest stop
- **Audit trail** — every decision has provenance

## Cedar Policy Engine

| Policy type | What it governs |
|---|---|
| Agent authority levels | Advisory, participatory, supervisory, sovereign |
| Commitment actions | Propose, commit, promote |
| Amount thresholds | Spending limits requiring human approval |
| Delegation tokens | Ed25519-signed, time-scoped, replay-protected |

## Truth Catalog

| Truth | What it governs |
|---|---|
| `evaluate-vendor` | Multi-criteria vendor evaluation with swarm consensus |
| `dynamic-due-diligence` | Research loop with contradictions and synthesis |
| `audit-vendor-decision` | Audit trail and compliance scan |
| `authorize-vendor-commitment` | Cedar policy gates for procurement |

## Organism Patterns

Beyond flat agent swarms — structured organizational intelligence:

| Pattern | What it gives you |
|---|---|
| Intent decomposition | Break complex goals into governed subtasks |
| Collaboration topologies | Huddle, panel, discussion, self-organizing |
| Adversarial review | Five skepticism kinds challenge every plan |
| Simulation swarm | Five-dimension parallel stress testing |
| Learning episodes | Outcomes calibrate planning priors |
| Domain packs | 15 pre-built organizational workflows |

See `kb/Converge/Organism Patterns.md` for implementation details.

## API Surface

| Role | Crate | Primary types |
|---|---|---|
| Converge authoring | `converge-pack` | `Suggestor`, `AgentEffect`, `ProposedFact`, `ContextKey` |
| Converge runtime | `converge-kernel` | `Engine`, `Context`, `Budget`, criteria |
| LLM access | `converge-provider` | `ChatBackend`, provider selection |
| Organism authoring | `organism-pack` | `IntentPacket`, `Plan`, `CollaborationCharter` |
| Organism runtime | `organism-runtime` | `Registry`, readiness, built-in packs |

## What You Can Build

- Custom truths with domain-specific governance rules
- Cedar policies for new authority patterns
- Agents (Suggestors) that propose facts through the promotion gate
- Organism-planned multi-agent workflows with adversarial review
- Visualizations of convergence behavior
- Multi-agent scenarios with competing proposals

## Getting Started

```bash
just hit-the-ground-running   # build, test, lint
just server                   # start governance server
just test                     # run tests
just test-coverage            # tests with coverage report
```

See `kb/Development/Getting Started.md` for the full walkthrough.
