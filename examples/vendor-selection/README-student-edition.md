# Vendor Selection Truth — Student Edition

> A modular, teachable implementation of governed vendor selection for the Converge Governance Hackathon.

## Overview

The **Vendor Selection Simple Truth** demonstrates how to evaluate vendors on multiple criteria, synthesize results into a ranked shortlist, and gate the final decision with policy rules. It's designed to be:

- **Modular**: Each of the 5 evaluators is self-contained and teachable
- **Extensible**: Students can add new criteria or modify weights
- **Policy-gated**: Cedar policy rules control the commitment decision
- **Traceable**: Full audit trail of all evaluation facts and scores

## The 5 Vendor Evaluators

### 1. Price Evaluator
**What it does**: Scores vendors based on monthly cost vs. budget.

**Scoring**:
- If vendor cost ≤ budget → score approaches 100
- If vendor cost > budget → score decreases proportionally
- Formula: `score = 100 - (vendor_cost / budget * 100)`

**Try modifying**:
- Adjust the budget input parameter
- Change the formula to penalize high cost more/less aggressively

### 2. Compliance Screener
**What it does**: Scores vendors on required certifications.

**Scoring**:
- Checks for SOC2, ISO27001, GDPR certifications
- Score = (matched_certs / 3) * 100
- A vendor with all 3 certs scores 100

**Try modifying**:
- Add or remove required certifications
- Weight different certifications differently (e.g., GDPR 40%, SOC2 30%, ISO27001 30%)

### 3. Reliability Evaluator
**What it does**: Scores vendors on SLA uptime percentage.

**Scoring**:
- Converts SLA uptime (e.g., 99.99%) into a 0-100 score
- Formula: `score = 100 - (100 - sla_uptime) * 10`
- 99.5% → ~85, 99.95% → ~95, 99.99% → ~99

**Try modifying**:
- Adjust the sensitivity to uptime differences
- Add penalties for downtime history (not just promised SLA)

### 4. Support Tier Evaluator
**What it does**: Scores vendors on support quality and response time.

**Scoring**:
- Response time score: `(100 - (response_hours / 48 * 100)).clamp(0, 100)`
- Tier bonus: premium=100, standard=75, basic=50
- Final score = average of both

**Try modifying**:
- Adjust max response SLA threshold (currently 48 hours)
- Weight tier and response time differently (e.g., 70% tier, 30% response)

### 5. Stability Evaluator
**What it does**: Scores vendors on financial health and organizational maturity.

**Scoring**:
- Funding stage: profitable=100, series-d=90, series-b=60, series-a=40, seed=20
- Headcount normalized: (headcount / 500) * 100 (capped at 100)
- Company age normalized: (age / 20) * 100 (capped at 100)
- Final score = 50% funding + 25% headcount + 25% age

**Try modifying**:
- Adjust the weight distribution (currently 50/25/25)
- Add more signals: revenue, burn rate, customer concentration
- Adjust the normalization thresholds (e.g., 500 headcount → "stable")

## Synthesis & Shortlist

After all 5 evaluators run, the **Shortlist Synthesis** agent:

1. Collects scores from each evaluator
2. Computes a **weighted average** of the 5 criteria
3. Ranks vendors by final score
4. Produces a recommendation:
   - **Score ≥ 70**: "Recommend [vendor]"
   - **Score 50-70**: "Escalate [vendor] for review"
   - **Score < 50**: "Reject [vendor]"

**Default weights** (you can modify these):
- Price: 30%
- Compliance: 20%
- Reliability: 20%
- Support: 15%
- Stability: 15%

## Cedar Policy Gate

The `vendor-selection-policy-simple.cedar` file gates the final decision:

```cedar
// Promote if top score >= 70
permit(principal, action == Action::"shortlist", resource)
when {
  context.top_score >= 70.0
};

// Escalate if score 50-70
forbid(...) when { context.top_score >= 50.0 && context.top_score < 70.0 };

// Block if score < 50
forbid(...) when { context.top_score < 50.0 };
```

**Try modifying**:
- Change 70 to 75 to raise the quality threshold
- Add additional gates (e.g., compliance_score >= 80)
- Require human approval below a certain score

## The 3 Seed Vendors

All realistic and modifiable:

### CloudTrust AI (Enterprise)
- **Cost**: $180,000/month → high cost, high quality
- **Compliance**: SOC2, ISO27001, GDPR (3/3 certs)
- **Reliability**: 99.99% SLA → best in class
- **Support**: Premium tier, 1-hour response
- **Stability**: Series D, founded 2016, 450 headcount → most stable

**Likely scores**: Price=44, Compliance=100, Reliability=99, Support=100, Stability=100 → ~85 overall

### QuickAI Solutions (Startup)
- **Cost**: $30,000/month → cheap, good for budget-conscious
- **Compliance**: SOC2 only (1/3 certs)
- **Reliability**: 99.5% SLA → basic reliability
- **Support**: Basic tier, 24-hour response
- **Stability**: Series A, founded 2023, 35 headcount → early stage

**Likely scores**: Price=100, Compliance=33, Reliability=85, Support=50, Stability=20 → ~57 overall (escalate)

### MidScale AI (Balanced)
- **Cost**: $90,000/month → middle ground
- **Compliance**: SOC2, ISO27001 (2/3 certs)
- **Reliability**: 99.95% SLA → very good
- **Support**: Standard tier, 4-hour response
- **Stability**: Series B, founded 2020, 180 headcount → growing

**Likely scores**: Price=72, Compliance=67, Reliability=95, Support=88, Stability=75 → ~79 overall (recommend)

## Running the API

### Setup

```bash
cd crates/governance-server
cargo build
just server
```

### Execute via curl

```bash
# Run with seed vendors
curl -X POST http://localhost:8080/v1/truths/vendor-selection-simple/execute \
  -H "Content-Type: application/json" \
  -d '{
    "vendors_json": "[{\"name\": \"CloudTrust AI\", \"cost_minor\": 18000000, \"certifications\": [\"SOC2\", \"ISO27001\", \"GDPR\"], \"sla_uptime\": 99.99, \"support_tier\": \"premium\", \"response_sla_hours\": 1.0, \"company_funding\": \"series-d\", \"founded_year\": 2016, \"headcount\": 450}]"
  }'

# Run with your own vendors (JSON array)
curl -X POST http://localhost:8080/v1/truths/vendor-selection-simple/execute \
  -H "Content-Type: application/json" \
  -d '{
    "vendors_json": "[vendor-data-array-here]",
    "budget": "10000000"
  }'
```

### Expected output

```json
{
  "converged": true,
  "cycles": 2,
  "stop_reason": "Converged",
  "criteria_outcomes": [
    {
      "criterion": "all-vendors-evaluated",
      "result": "Met { evidence: [] }"
    },
    {
      "criterion": "compliance-screened",
      "result": "Met { evidence: [] }"
    },
    ...
  ],
  "projection": {
    "events_emitted": 1,
    "details": null
  }
}
```

## Modifying the Truth

### Add a new criterion

1. **Create a new Suggestor** in `vendor_selection_simple.rs`:
   ```rust
   struct MyCriterionEvaluatorSuggestor {
       vendors: Vec<VendorData>,
   }

   #[async_trait]
   impl Suggestor for MyCriterionEvaluatorSuggestor {
       fn name(&self) -> &str { "my-criterion" }
       fn dependencies(&self) -> &[ContextKey] { &[] }
       fn accepts(&self, ctx: &dyn ContextView) -> bool {
           !ctx.get(ContextKey::Evaluations)
               .iter()
               .any(|f| f.id.starts_with("criterion:mycriterion:"))
       }
       async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
           // Evaluate each vendor and emit ProposedFact
       }
   }
   ```

2. **Register it in the executor**:
   ```rust
   engine.register_suggestor_in_pack(
       "evaluation-pack",
       MyCriterionEvaluatorSuggestor {
           vendors: vendors.clone(),
       },
   );
   ```

3. **Add it to the truth definition** in `governance-truths/src/lib.rs`:
   ```rust
   criteria: &[
       ("my-criterion-met", "My criterion has been evaluated"),
       // ... other criteria
   ],
   ```

4. **Update the criterion evaluator** in `governance-truths/src/lib.rs`:
   ```rust
   impl CriterionEvaluator for VendorSelectionSimpleEvaluator {
       fn evaluate(&self, criterion: &Criterion, context: &dyn Context) -> CriterionResult {
           match criterion.id.as_str() {
               "my-criterion-met" => {
                   if context.get(ContextKey::Evaluations)
                       .iter()
                       .any(|f| f.id.starts_with("criterion:mycriterion:"))
                   {
                       CriterionResult::Met { evidence: vec![] }
                   } else {
                       CriterionResult::Unmet { reason: "...".into() }
                   }
               }
               // ... other criteria
           }
       }
   }
   ```

### Adjust synthesis weights

In `ShortlistSynthesisSuggestor::execute()`, modify the weighting formula:

```rust
// Current: 30% price, 20% compliance, 20% reliability, 15% support, 15% stability
let weighted = vs.price * 0.30
    + vs.compliance * 0.20
    + vs.reliability * 0.20
    + vs.support * 0.15
    + vs.stability * 0.15;

// Try: emphasize compliance and stability
let weighted = vs.price * 0.20
    + vs.compliance * 0.30
    + vs.reliability * 0.15
    + vs.support * 0.15
    + vs.stability * 0.20;
```

### Change policy thresholds

Edit `vendor-selection-policy-simple.cedar`:

```cedar
// Raise the quality bar to 80
permit(principal, action == Action::"shortlist", resource)
when {
  context.top_score >= 80.0  // was 70.0
};

// Escalation band changes too
forbid(...) when { context.top_score >= 50.0 && context.top_score < 80.0 };
forbid(...) when { context.top_score < 50.0 };
```

## Running Tests

```bash
cargo test vendor_selection_simple
```

Tests cover:
- Happy path: all vendors evaluate and converge
- Edge case: low-score vendors get escalated
- Edge case: missing data handled gracefully
- Soak tests: repeated executions with many vendors

## Key Concepts for Learning

### Modular Architecture
Each evaluator is independent. They:
1. Check if their criterion fact exists (idempotency)
2. Emit scoring facts to the context
3. Don't depend on other evaluators

This mirrors real governance: specialists evaluate independently, then results synthesize.

### Multi-Criteria Synthesis
The shortlist synthesis shows how to:
1. Collect heterogeneous evidence (scores from different evaluators)
2. Normalize to a common scale (0-100)
3. Compute a weighted aggregate
4. Rank and recommend

This is the core of any multi-criteria decision framework.

### Policy-Gated Decisions
Cedar policies express business rules that can't be captured in code:
- Quality thresholds
- Escalation conditions
- Approval gates

Modifying the policy (without recompiling) is the core Reflective Labs value prop.

### Idempotency
Each suggestor checks `if criterion already exists, skip`. This ensures:
- Safe re-entry if the engine loops
- No duplicate work
- Convergence toward a stable state

## Next Steps for Students

1. **Run the happy path**: Execute the default 3 vendors, observe scores
2. **Modify weights**: Change the synthesis weights, re-run, observe impact
3. **Add a new vendor**: Create a JSON vendor object with different characteristics
4. **Change thresholds**: Modify the Cedar policy, see escalation trigger
5. **Add a criterion**: Implement a new evaluator (e.g., "geographic region")
6. **Stress test**: Add 20 vendors, observe convergence behavior

## Reference

- **Truth Definition**: `crates/governance-truths/src/lib.rs` (lines ~85-114)
- **Executor**: `crates/governance-server/src/truth_runtime/vendor_selection_simple.rs`
- **Policy**: `examples/vendor-selection/vendor-selection-policy-simple.cedar`
- **Seed Data**: `examples/vendor-selection/vendor-seed-simple.json`
- **Tests**: `crates/governance-server/src/truth_runtime/vendor_selection_simple.rs` (test module)

## Questions?

See `kb/Development/` for deeper dives into governance architecture. Ping the hackathon mentors for help extending the truth.
