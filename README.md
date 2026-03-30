# AI Governance Hackathon — Converge

Build enterprise AI governance infrastructure using multi-agent convergence. Compliance, security, LLM monitoring, and automated guardrails that protect sensitive data without killing innovation.

**Sponsors:** Kong · Vivicta
**Challenge contributors:** Ericsson, H&M, Mölnlycke Health Care, Azets, Green Cargo, SBAB, DFDS, Burnt Oak Partners

## Getting Started

```bash
# Prerequisites: Rust 1.80+, just
git clone <this-repo>
cd converge-governance-hackathon

just test    # Should pass — 6+ tests out of the box
just server  # Starts HTTP server on :8080
```

Try the reference truth:

```bash
curl -X POST http://localhost:8080/v1/truths/evaluate-vendor/execute \
  -H 'Content-Type: application/json' \
  -d '{"inputs": {"vendors": "Acme AI, Beta ML, Gamma LLM"}}'
```

You should see a converged decision with compliance screening, cost analysis, and a recommendation.

## The Challenge: Vendor Selection

You're evaluating AI vendors for enterprise deployment. The decision must be **traceable, compliant, and defensible to auditors**.

Build a multi-agent system where:
- A **compliance screener** checks vendors against GDPR, AI Act, and internal policy
- A **cost analyst** evaluates pricing, token costs, and volume models
- A **capability matcher** scores vendor features against requirements (LLM-backed, via Kong)
- A **risk scorer** quantifies vendor risk across dimensions
- A **decision synthesizer** reads all evidence and produces a recommendation

Every agent proposes facts. Converge's promotion gate validates and promotes them. The criterion evaluator checks if the decision criteria are met. If confidence is too low, the system honestly says "needs human review" instead of guessing.

## How It Works

```
Input (vendor list) → Seed Context
    → ComplianceScreenerAgent proposes compliance:screen:* facts
    → CostAnalysisAgent proposes cost:estimate:* facts
    → DecisionSynthesisAgent reads all facts, proposes decision:recommendation
    → CriterionEvaluator checks: all screened? recommendation produced?
    → ConvergeResult: converged/blocked/budget-exhausted
    → Project decision into GovernanceKernel (audit trail, decision record)
```

## Repo Structure

```
crates/
  governance-kernel/     Domain model + in-memory store (Vendor, PolicyRule, etc.)
  governance-truths/     Truth catalog + converge bindings + evaluators
  governance-server/     HTTP server + truth executors (evaluate_vendor.rs is the reference)
  governance-app/        Shared app layer for desktop

challenges/              Challenge briefs and ideas
docs/                    Architecture docs, agent patterns, Kong integration
```

## What to Build

The reference truth (`evaluate-vendor`) works end-to-end with 3 placeholder agents. Your job:

1. **Make the agents real.** Replace placeholder logic with actual LLM calls (via Kong), analytics, and optimization.
2. **Add more agents.** Risk scoring, capability matching, contract analysis.
3. **Handle edge cases.** What if a vendor fails compliance? What if the LLM is uncertain? Use `CriterionResult::Blocked` and `StopReason::HumanInterventionRequired`.
4. **Route through Kong.** All LLM API calls go through Kong AI Gateway for rate limiting, PII detection, and cost tracking.
5. **Build the UX.** The desktop app shell is ready for Svelte components.

## Key Concepts

| Concept | What it means |
|---|---|
| **Truth** | A job the system can perform (vendor evaluation, cost audit, compliance check) |
| **Agent** | A unit of work that reads context and proposes facts |
| **Fact** | A validated piece of evidence (promoted from a proposal through the governance gate) |
| **Pack** | A group of agents that work together (compliance-pack, cost-pack) |
| **Criterion** | A success condition (all vendors screened, recommendation produced) |
| **Convergence** | The engine runs agents in cycles until criteria are met or budget is exhausted |

## API

```
GET  /health                        Health check
GET  /v1/truths                     List available truths
POST /v1/truths/{key}/execute       Execute a truth with inputs
GET  /v1/decisions                  List recent decisions
GET  /v1/vendors                    List registered vendors
GET  /v1/audit                      Audit trail
```

## Dependencies

- [Converge](https://converge.zone) — multi-agent convergence runtime
- [Axum](https://github.com/tokio-rs/axum) — HTTP server
- [Kong AI Gateway](https://konghq.com) — API gateway for LLM routing (hackathon sponsor)
