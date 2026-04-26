# Demo AI Strategy Candidates

Use this pair when the demo needs a clear business choice rather than a long model leaderboard.

The source evidence comes from `competition-matrix.json`: 84 model-role runs, Artificial Analysis Intelligence Index context, OpenRouter pricing/usage context, and the governed provider-mix recommendation.

## Candidate 1: Premium Single-Model Strategy

**Name in demo data:** `Premium Single-Model Strategy (Claude Opus 4.7)`

**Business meaning:** Choose one premium frontier model as the default for most governed vendor-selection work.

**Why it is plausible:**

- Easy story for procurement and leadership: one strategic provider, one model family, fewer moving parts.
- Strong generic reasoning reputation and high Intelligence Index.
- Suitable as an escalation option when ambiguity or risk is genuinely high.

**Evidence from the matrix:**

- Claude Opus 4.7 has Intelligence Index `57`.
- It ranked `#8` in the governed business-decision competition, below lower-index models.
- It is roughly `33x` more expensive than Mistral Small in the matrix notes.
- Estimated cost for `1000` governed vendor-selection runs: `$40.77`.

**Demo interpretation:**

This is the intuitive executive choice: buy the smartest model and use it everywhere. The governed evaluation should expose why that is not necessarily the best fit for this workload.

## Candidate 2: Governed Multi-Model Router Strategy

**Name in demo data:** `Governed Multi-Model Router Strategy (Gemma + Mistral + Arcee)`

**Business meaning:** Use a governed provider mix behind a router, assigning each workload to the model or tool that fits it best.

**Provider mix:**

| Need | Route |
|---|---|
| Synthesis and decision-making | Gemma 4 31B |
| Compliance, cost, and risk work | Mistral Small 4 |
| Efficient secondary / token-optimized tasks | Arcee Trinity Large |
| High-risk escalation | Claude Opus 4.7 only when warranted |
| Broad web evidence | Brave Search API |
| Deep canonical evidence | Tavily Search API |
| Governance controls | Kong AI Gateway or equivalent router/gateway |

**Evidence from the matrix:**

- Gemma 4 31B ranked `#1` in the competition with Intelligence Index `39`.
- Mistral Small 4 ranked `#2`, was the fastest all-rounder, and has Intelligence Index `28`.
- Arcee Trinity ranked `#3` and used the fewest average tokens.
- The basket estimate for `1000` governed vendor-selection runs with `5%` escalation is `$2.66`.
- The non-escalation basket estimate is `$0.62`, compared with `$40.77` for single Opus and `$288.30` for GPT-5.4 Pro.

**Demo interpretation:**

This is the OpenClaw argument: do not optimize for generic intelligence too early. Optimize for the real need: concise evidence, governed promotion, cost control, fallback, and auditability.

## Slide Line

> "The surprising result is not that Gemma beats Opus on a generic benchmark. It does not. The result is that Gemma, Mistral, and Arcee are better at this governed business workflow when routed behind policy, evidence, and audit controls."

## JSON Input

The executable two-candidate input is `demo-ai-strategy-candidates.json`.

Run it directly:

```bash
just demo-strategy-candidates --mode=governed --business
```

Expected business outcome: the governed multi-model router strategy should win because it has better task fit, lower risk, broader governance controls, and much lower estimated cost for this workload.
