# Evaluation Model

## Hard Gates

These conditions can stop a candidate even if it scores well:

- `compliance_status` must be `compliant`.
- `risk_score` must be less than or equal to the configured maximum risk.
- A commitment needs the right authority.
- A commitment over the HITL threshold needs human approval unless delegated by policy.

## Objective Function

The governed shortlist uses this transparent objective:

```text
0.35 * capability
+ 0.25 * risk_adjusted
+ 0.20 * cost_efficiency
+ 0.20 * certification_coverage
```

This is intentionally adjustable. The point of the demo is not that these weights are universal. The point is that the weights are visible, auditable, and easy to challenge.

## Current Story

The stable governed demo uses `demo-ai-vendors.json`.

Mistral can beat Anthropic and OpenAI because the buyer values cost efficiency and certification coverage alongside capability and risk. Qwen is blocked because compliance is pending and risk exceeds the threshold.

## Competition Story

The breakout demo uses:

- `competition-matrix.json`
- `demo-competition-vendors.json`

The matrix illustrates that no single model dominates all roles. A provider basket can be better than forcing one winner:

- strong model for high-ambiguity reasoning
- fast model for structured synthesis
- broad search provider for discovery
- focused retrieval provider for canonical evidence
- gateway for policy, audit, rate limits, PII, and cost controls

## Safe Tuning

To make the demo more price-sensitive, increase the cost weight in the runtime or raise costs in the candidate data.

To make the demo more risk-sensitive, lower `--max-risk` or increase candidate `risk_score`.

To make the demo more compliance-sensitive, remove certifications or set a candidate to `pending`.

To test honest stopping, run with advisory authority or without human approval.
