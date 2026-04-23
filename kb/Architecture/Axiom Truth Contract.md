---
tags: [architecture, axiom, truth]
---
# Axiom Truth Contract

Axiom is the truth contract layer.

Axiom owns the definition, validation, simulation, and policy lens for what a governed decision must satisfy. It does not run convergence, compose formations, promote facts, or learn decision priors. It makes business truth explicit, testable, and auditable before Organism plans the decision and before Converge executes it.

## Layer Contract

| Layer | Authority | Owns |
|---|---|---|
| Axiom | Normative | What a valid decision must prove |
| Organism | Strategic | How to form the team to prove it |
| Converge | Operational | Governed execution, promotion, and audit |
| Hackathon apps | Product | UI, artifacts, demo data, local writeback |

Short version: Axiom defines what must be true. Organism decides how to form the team to satisfy it. Converge governs execution and promotion of facts. Hackathon apps wire the product experience, artifacts, and writeback.

## Executable Specification

Axiom is the executable specification layer for governed decisions. It turns business intent, policies, examples, invariants, and acceptance criteria into typed truth contracts.

Those contracts are used to:

- validate inputs before execution
- simulate failure modes
- explain governance requirements
- test whether a decision process is admissible
- expose the policy lens for what evidence and approvals are required

Axiom prevents Organism and Converge from optimizing the wrong thing.

## Non-Goals

Axiom never becomes the agent runtime.

Axiom never owns formation compilation.

Axiom never bypasses Converge promotion.

Axiom never stores the business decision as authoritative state.

## Hackathon Use

The hackathon app is the decision-product surface.

It uses Axiom to author and validate the vendor-selection truth. It uses Organism to compile that truth into a governed formation. It uses Converge to execute the formation and produce promoted evidence. The app owns UI, imported artifacts, demo data, and writeback into the local product model.

Implementation rule: keep Axiom-facing code on truth contracts and validation. If code starts selecting suggestors, running tournaments, promoting facts, or writing authoritative decision state, it has crossed into Organism, Converge, or app responsibility.

See also: [[Architecture/Overview]], [[Architecture/Layers]], [[Domain/Truths]], [[Development/Programming API Surfaces]]
