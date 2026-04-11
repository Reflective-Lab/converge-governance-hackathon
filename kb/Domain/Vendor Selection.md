---
tags: [domain, challenge]
---
# Vendor Selection Challenge

Enterprise AI vendor selection as a multi-agent governance problem. Four dimensions evaluated independently by specialized [[Domain/Agents|agents]] that converge on a defensible recommendation.

## The Problem

Select AI vendors (LLM providers, embedding services, inference platforms) across compliance, cost, capability, and risk. Today done in spreadsheets and meetings — automate it with governed multi-agent convergence.

## Operator Flow

1. Load a local vendor-selection source file
2. Preview the parsed vendor list and truth execution intent
3. Run the `evaluate-vendor` truth locally in the Rust core
4. Call out to [[Integrations/Kong Gateway|Kong]] only when an agent needs LLM or business-service access

## Input Formats

**Gherkin:**
```gherkin
Feature: Enterprise AI vendor selection

  Scenario: Evaluate candidate vendors for governed rollout
    Given truth "evaluate-vendor"
    And vendors:
      | name      |
      | Acme AI   |
      | Beta ML   |
      | Gamma LLM |
```

**Truth-spec JSON:**
```json
{
  "title": "Enterprise AI vendor selection",
  "truth_key": "evaluate-vendor",
  "vendors": ["Acme AI", "Beta ML", "Gamma LLM"],
  "inputs": {
    "decision_context": "Select a primary AI vendor for an enterprise rollout."
  }
}
```

Both produce the same execution. The Gherkin file is not decoration — it is a loadable input that the desktop app parses and executes directly. The parser extracts the truth key and vendor table, builds an inputs map, and hands it to the same executor.

Example input files are in the repo:
- `examples/vendor-selection/vendor-selection.feature`
- `examples/vendor-selection/vendor-selection.truths.json`

## Success Criteria

```
"all-vendors-screened"        — Every vendor has compliance:screen:* facts
"costs-analyzed"              — Every vendor has cost:estimate:* facts
"recommendation-produced"     — decision:recommendation fact exists
"confidence-above-threshold"  — Confidence > 7000 bps OR honestly blocked
```

## What's Already Built

The reference executor in `evaluate_vendor.rs` has 3 placeholder suggestors producing hardcoded facts. What needs to become real:

- Replace hardcoded compliance results with actual policy evaluation
- Replace hardcoded cost estimates with real pricing analysis
- Add CapabilityMatcherAgent and RiskScorerAgent
- Make DecisionSynthesisAgent call an LLM via Kong
- Handle the case where a vendor fails compliance (should block, not recommend)

## Judging Criteria

1. **Governance quality** — traceable decisions an auditor can follow
2. **Agent sophistication** — real analysis (LLM, analytics, optimization) vs hardcoded
3. **Honest stopping** — admits when it can't decide
4. **Kong integration** — LLM calls routed through Kong with guardrails
5. **Demo quality** — end-to-end vendor evaluation walkthrough

See also: [[Domain/Agents]], [[Domain/Truths]], [[Architecture/Convergence Loop]]
