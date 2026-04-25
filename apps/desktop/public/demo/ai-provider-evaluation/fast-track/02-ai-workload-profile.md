# AI Workload Profile

## Workloads

| Workload | Share | Latency Need | Quality Need | Notes |
|---|---:|---|---|---|
| Discussion and synthesis | 45% | Medium | High | Used in huddles, summaries, and trade-off analysis |
| Coding-agent execution | 25% | Medium | High | Must handle repo context and structured patches |
| Compliance/risk extraction | 15% | Low | High | Must produce structured evidence and cite sources |
| Stakeholder polishing | 10% | Medium | Medium | Executive-ready wording and concise summaries |
| Escalation reasoning | 5% | Low | Very high | Used only for high ambiguity or high-risk decisions |

## Operating Assumptions

- Most tasks should not require a premium frontier model.
- Structured JSON output must be recoverable, not brittle.
- Model choice should be role-based, not hard-coded at the top level.
- Search should combine wide discovery and deep verification.

## Preferred Role Mapping

- Primary: Arcee Trinity or Mistral Small for most discussion and synthesis.
- Secondary: Writer Palmyra for stakeholder-ready summaries.
- Escalation: Claude or GPT Pro for high-risk ambiguity.
- Routing: Kong or OpenRouter where governance, fallback, and observability matter.

## Non-Functional Requirements

- Provider failures must not collapse the whole decision flow.
- Output parsing errors must be handled close to the provider boundary.
- Every promoted fact must include source, agent, and rationale.

