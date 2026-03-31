# Challenge: AI Vendor Selection as Multi-Agent Governance

## The Problem

Your enterprise needs to select AI vendors (LLM providers, embedding services, inference platforms). The decision involves compliance, cost, capability, and risk — four dimensions that different specialists evaluate independently. Today this is done in spreadsheets and meetings. You're going to automate it with governed multi-agent convergence.

## What You're Building

A self-contained desktop app where specialized agents evaluate vendors from different angles, propose their findings as facts, and converge on a defensible recommendation. Every step is traceable. If the system can't decide with confidence, it says so.

The primary operator flow should be:

1. Load a local vendor-selection source file.
2. Preview the parsed vendor list and truth execution intent.
3. Run the `evaluate-vendor` truth locally in the Rust core.
4. Call out to Kong only when an agent needs LLM or business-service access.

The starter repo now includes example inputs for this flow:

- [vendor-selection.feature](/Users/kpernyer/dev/work/converge-governance-hackathon/examples/vendor-selection/vendor-selection.feature)
- [vendor-selection.truths.json](/Users/kpernyer/dev/work/converge-governance-hackathon/examples/vendor-selection/vendor-selection.truths.json)

## Agents to Build

### 1. ComplianceScreenerAgent (rule-based)
- Input: vendor documentation, policy rules
- Checks: GDPR compliance, AI Act requirements, data residency, certifications
- Output: `compliance:screen:{vendor}` facts with pass/fail per policy
- **Kong integration:** Fetch vendor compliance docs via Kong-routed API

### 2. CostAnalyticsAgent (analytics)
- Input: vendor pricing models, usage projections
- Analyzes: token costs, volume discounts, billing models, total cost of ownership
- Output: `cost:estimate:{vendor}` facts with monthly projections
- **Stretch:** Use Polars (via converge-analytics) for temporal cost projections

### 3. CapabilityMatcherAgent (LLM-backed)
- Input: requirements document, vendor feature lists
- Evaluates: model quality, latency, context window, fine-tuning support
- Output: `capability:match:{vendor}` facts with scores per requirement
- **Kong integration:** LLM call routed through Kong AI Gateway

### 4. RiskScorerAgent (optimization)
- Input: all compliance and capability facts
- Scores: vendor lock-in risk, financial stability, compliance risk, operational risk
- Output: `risk:score:{vendor}` facts with weighted risk scores
- **Stretch:** Use OR-Tools (via converge-optimization) for multi-criteria optimization

### 5. DecisionSynthesisAgent (LLM-backed)
- Input: all facts from all agents
- Synthesizes: overall recommendation with rationale
- Output: `decision:recommendation` fact
- If confidence < 7000 bps → set `needs_human_review: true`
- **Kong integration:** Final synthesis LLM call through Kong

## Success Criteria

```rust
"all-vendors-screened"     // Every vendor has compliance:screen:* facts
"costs-analyzed"           // Every vendor has cost:estimate:* facts
"recommendation-produced"  // decision:recommendation fact exists
"confidence-above-threshold" // Confidence > 7000 bps OR honestly blocked
```

## What's Already Built

The reference executor in [evaluate_vendor.rs](/Users/kpernyer/dev/work/converge-governance-hackathon/crates/governance-server/src/truth_runtime/evaluate_vendor.rs) has 3 placeholder agents that produce hardcoded facts. The shared app layer can also preview and execute local Gherkin or truth-spec inputs for the `evaluate-vendor` flow. Your job is to make the rest real:

- Replace hardcoded compliance results with actual policy evaluation
- Replace hardcoded cost estimates with real pricing analysis
- Add the CapabilityMatcherAgent and RiskScorerAgent
- Make DecisionSynthesisAgent call an LLM via Kong
- Handle the case where a vendor fails compliance (should block, not recommend)

## Judging Criteria

1. **Governance quality:** Are decisions traceable? Can an auditor understand why a vendor was recommended?
2. **Agent sophistication:** Do agents use real analysis (LLM, analytics, optimization) or just hardcoded values?
3. **Honest stopping:** Does the system admit when it can't decide?
4. **Kong integration:** Are LLM calls routed through Kong with proper guardrails?
5. **Demo quality:** Can you walk through a vendor evaluation end-to-end?
