---
tags: [integrations, kong, demo]
---
# Kong Demo Story

This is the end-to-end demonstration walkthrough for the **two-layer AI governance** story. It shows how Converge (internal governance) and Kong (external governance) work together to make AI systems auditable.

## The Thesis

You can't govern AI systems with only internal rules. You can't govern them with only external controls. You need both.

| Layer | What it governs | The question it answers |
|---|---|---|
| **Converge** (internal) | Decision-making, proposal promotion, policy gates | "How did you reach this decision?" |
| **Kong** (external) | External access, LLM calls, cost, PII | "What did you access, at what cost, and did you leak data?" |

## Demo Walkthrough

### Scene 1: Load the Spec

**Operator action:** Open the desktop app (Helm), load `vendor-selection.feature` with 3 vendors: Anthropic, OpenAI, Mistral.

**Preview shows:**
- Truth intent: `evaluate-vendor`
- 5 agents: ComplianceScreenerAgent, CostAnalyticsAgent, CapabilityMatcherAgent, RiskScorerAgent, DecisionSynthesisAgent
- 4 criteria: compliance, cost, capability, risk

**What the operator sees:**
```
Truth: evaluate-vendor
Vendors: 3 (Anthropic, OpenAI, Mistral)
Agents: 5
Criteria: 4
Estimated cost: $0.02-0.05 (3 LLM calls via Kong)
```

---

### Scene 2: Run the Evaluation

**Operator action:** Click "Run Evaluation"

**The convergence loop begins:**

#### Cycle 1: ComplianceScreenerAgent (rule-based, local)

- No LLM calls — pure Rust logic checking GDPR/AI Act policies
- Kong dashboard: **quiet**. No external requests.
- Output: 3 compliance facts promoted, all vendors pass

```
[Converge] ComplianceScreenerAgent: 3 facts promoted
[Kong] 0 requests, $0.00
```

#### Cycle 2 (parallel): CostAnalyticsAgent + CapabilityMatcherAgent

Both call LLMs through Kong:

**CostAnalyticsAgent:**
- POST to Kong `/llm/v1/chat` → routes to upstream (configured in Konnect)
- Kong response: 847 tokens, $0.003
- Kong headers: `X-Kong-Proxy-Latency: 45ms`, `X-RateLimit-Remaining-Minute: 999`

**CapabilityMatcherAgent:**
- Separate call through Kong
- Kong response: 1,200 tokens, $0.004
- Kong PII Sanitizer strips vendor names before they reach the model (configured in Konnect)

```
[Converge] CostAnalyticsAgent: 3 cost facts promoted
[Converge] CapabilityMatcherAgent: 3 capability facts promoted
[Kong] 2 requests, 2,047 tokens, $0.007
```

#### Cycle 3: RiskScorerAgent (computation, local)

- No LLM calls — mathematical scoring of lock-in, financial stability, compliance risk
- Output: 3 risk score facts promoted

```
[Converge] RiskScorerAgent: 3 risk facts promoted
[Kong] 2 requests, $0.007 (unchanged)
```

#### Cycle 4: DecisionSynthesisAgent

- Calls LLM through Kong to synthesize all facts into a recommendation
- Kong response: 1,800 tokens, $0.006
- Rate limiter checks team budget (configured in Konnect) — request proceeds
- If confidence < 70% → HITL gate fires, human review required

```
[Converge] DecisionSynthesisAgent: recommendation promoted
[Converge] Confidence: 82% — no human review needed
[Kong] 3 requests, 3,847 tokens, $0.013
```

#### Cycle 5: No changes → fixed point → converged

---

### Scene 3: The Governance Dashboard (Split Screen)

**Left panel — Converge (internal governance):**

```
Criteria Results:
  ✓ compliance: all 3 vendors pass
  ✓ cost: Anthropic cheapest at $0.003/vendor
  ✓ capability: Anthropic, OpenAI meet requirements
  ✓ risk: all vendors acceptable

Convergence:
  5 cycles
  11 facts promoted
  3 promotion gates passed (Cedar policies)
  
Audit:
  ✓ Full provenance chain
  ✓ All proposals traced to agents
```

**Right panel — Kong (external governance):**

```
AI Gateway:
  3 LLM calls total
  3,847 tokens consumed
  $0.013 total cost
  
  Call 1: CostAnalyticsAgent → 847 tokens → $0.003
  Call 2: CapabilityMatcherAgent → 1,200 tokens → $0.004  
  Call 3: DecisionSynthesisAgent → 1,800 tokens → $0.006
  
Rate Limits:
  ✓ No violations (team budget: 100k tokens/day)
  
PII Redaction:
  ✓ 2 instances redacted (vendor names in capability request)
  
Latency:
  Avg proxy: 45ms
  Avg upstream: 120ms
```

**Summary banner:**
```
"This recommendation cost $0.013, took 5 cycles,
consulted 3 external models, redacted 2 PII instances,
and every step is auditable."
```

---

### Scene 4: "Change a Policy, See a Different Outcome"

**Operator action:** Modify Cedar policy in the desktop app:
```
permit(
  Authority::Agent(CostAnalyticsAgent),
  Action::CallLlm,
  Resource::Model("gpt-4")
) when { context.team_budget < 5000 };
```
Translation: "CostAnalyticsAgent cannot call GPT-4 when team budget is below 5000 tokens."

**Re-run evaluation:**

- CostAnalyticsAgent tries to call model → Kong rate limiter blocks
- Agent falls back to cheaper model (Claude 3.5 Haiku) via Kong load balancing
- Cost drops: $0.013 → $0.004
- Convergence still succeeds — same governance, different cost

```
[Converge] Policy enforcement: fallback to cheaper model
[Kong] Rate limit: gpt-4 blocked for CostAnalyticsAgent
[Result] Cost: $0.004 (down from $0.013)
```

---

### Scene 5: "Kong Stops Something"

**Operator action:** Enable Kong AI Prompt Guard in Konnect dashboard:
```
Block prompts containing: "competitor pricing", "acquisition target"
```

**Re-run evaluation:**

- CostAnalyticsAgent prompt contains "competitive pricing analysis"
- Kong AI Prompt Guard blocks the request → returns 403
- Agent honestly reports: "cost analysis unavailable — external access denied"
- Converge: criterion "costs-analyzed" → Blocked
- System stops honestly. No hallucinated cost data.

```
[Kong] AI Prompt Guard: blocked prompt containing "competitive pricing"
[Converge] Criterion "costs-analyzed": BLOCKED
[Result] Cost analysis unavailable — honest stop, no hallucination
```

---

## The Sponsor Pitch

**Kong isn't just routing API calls.** Kong is the **external governance layer** — it decides what AI agents can access, at what cost, with what data protections.

Converge decides what agents can **conclude**. Kong decides what agents can **consult**.

Together, they answer every audit question:

1. "How did you reach this decision?" → **Converge**: Full convergence trail, criteria evaluation, Cedar policy gates, every fact's provenance
2. "What external resources did you consult?" → **Kong**: API audit log, token counts, cost, PII redaction receipts
3. "Did you leak sensitive data?" → **Kong**: PII Sanitizer logs show what was redacted
4. "Did you stay within budget?" → **Kong**: Rate limiter enforcement, token budget tracking

**For the hackathon:** Participants build on Converge (internal governance). Kong is pre-configured and transparent — they see its effects in the desktop app and Kong dashboard. They don't need to configure Kong themselves, but they can explore its effects and understand the two-layer model.

See also: [[Integrations/Kong Gateway]], [[Converge/Core Concepts]], [[Converge/Governed Artifacts]]