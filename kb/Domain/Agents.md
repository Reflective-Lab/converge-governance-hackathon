---
tags: [domain, agents]
---
# Agents

Five agents for the [[Domain/Vendor Selection|vendor selection]] challenge. Each evaluates vendors from a different angle.

## 1. ComplianceScreenerAgent (rule-based)
- **Input:** vendor documentation, policy rules
- **Checks:** GDPR compliance, AI Act requirements, data residency, certifications
- **Output:** `compliance:screen:{vendor}` facts with pass/fail per policy
- **Kong:** fetch vendor compliance docs via Kong-routed API

## 2. CostAnalyticsAgent (analytics)
- **Input:** vendor pricing models, usage projections
- **Analyzes:** token costs, volume discounts, billing models, total cost of ownership
- **Output:** `cost:estimate:{vendor}` facts with monthly projections
- **Stretch:** Polars via converge-analytics for temporal cost projections

## 3. CapabilityMatcherAgent (LLM-backed)
- **Input:** requirements document, vendor feature lists
- **Evaluates:** model quality, latency, context window, fine-tuning support
- **Output:** `capability:match:{vendor}` facts with scores per requirement
- **Kong:** LLM call routed through [[Integrations/Kong Gateway|Kong AI Gateway]]

## 4. RiskScorerAgent (optimization)
- **Input:** all compliance and capability facts
- **Scores:** vendor lock-in risk, financial stability, compliance risk, operational risk
- **Output:** `risk:score:{vendor}` facts with weighted risk scores
- **Stretch:** OR-Tools via converge-optimization for multi-criteria optimization

## 5. DecisionSynthesisAgent (LLM-backed)
- **Input:** all facts from all agents
- **Synthesizes:** overall recommendation with rationale
- **Output:** `decision:recommendation` fact
- If confidence < 7000 bps → set `needs_human_review: true`
- **Kong:** final synthesis LLM call through Kong

## Suggestor Patterns

Suggestors in this repo follow one of:
- **Rule-based** — Rust logic, no external calls
- **Analytics/scoring** — Rust computation
- **LLM-backed** — calls model through [[Integrations/Kong Gateway|Kong]]
- **Service-backed** — gets business context through Kong APIs or [[Integrations/MCP Tools|MCP tools]]

If a real service is missing, mock it behind the same interface rather than embedding data directly.

See also: [[Development/Writing Agents]], [[Architecture/Convergence Loop]]
