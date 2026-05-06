# Budget Approval Truth — Threshold-Based Governance with HITL Gates

## Overview

The **Budget Approval Truth** demonstrates a 4-tier threshold-based approval workflow with Human-in-the-Loop (HITL) gates. This is a production-pattern example of threshold-based governance that scales across departments, geographies, and approval authorities.

### What It Does

- **Tier 1** (`< $5,000`): Auto-approve (no human review required)
- **Tier 2** (`$5k–$25k`): Requires supervisory authority + human approval
- **Tier 3** (`$25k–$100k`): Requires sovereign authority + human approval
- **Tier 4** (`> $100k`): Forbid unless sovereign authority present

Each request flows through:
1. **Validation** — Check request fields, amounts, authority
2. **Policy Evaluation** — Cedar policy gates the decision based on tier + authority
3. **HITL Gate** — For Tiers 2–3, emit a gate request and wait for human decision
4. **Audit Logging** — Record the entire decision with provenance

## Files

- **`budget-approval-policy.cedar`** — Cedar policy with 4-tier rules
- **`seed-requests.json`** — 6 sample requests spanning all tiers
- **Tests** — `crates/governance-server/tests/budget_approval_test.rs` (12 test cases)
- **Executor** — `crates/governance-server/src/truth_runtime/budget_approval.rs` (652+ lines)

## Running the Truth

### Via API

Start the server:

```bash
cd /Users/kpernyer/dev/apps/vendor-selection
just server
```

In another terminal, run a budget approval request:

```bash
curl -X POST http://localhost:8080/v1/truths/budget-approval/execute \
  -H "Content-Type: application/json" \
  -d '{
    "request_id": "office-supplies-001",
    "requester_id": "alice@example.com",
    "amount_minor": 120000,
    "currency_code": "USD",
    "authority": "advisory",
    "description": "Office supplies and equipment",
    "human_approval_present": false
  }'
```

### Test Cases

Run all tests:

```bash
cd /Users/kpernyer/dev/apps/vendor-selection
just test budget_approval_test
```

Key test scenarios:
- **Tier 1 auto-approval** — `test_tier_1_auto_approval_no_hitl()`
- **Tier 2 escalation** — `test_tier_2_escalation_requires_hitl()`
- **Tier 2 with human approval** — `test_tier_2_with_human_approval_promotes()`
- **Tier 3 escalation** — `test_tier_3_escalation_with_high_amount()`
- **Tier 4 rejection** — `test_tier_4_rejection_without_sovereign_authority()`
- **Tier 4 approval** — `test_tier_4_approval_with_sovereign_authority()`
- **Boundary tests** — Just under/over $5k thresholds
- **Audit recording** — `test_audit_entry_recorded()`
- **Validation errors** — Missing fields, invalid authority, negative amounts
- **All criteria met** — End-to-end happy path

## The 4 Tiers Explained

### Tier 1: Auto-Approve (< $5,000)

**Amount Range:** $0–$4,999
**Authority:** Any (advisory, supervisory, participatory, sovereign)
**Human Approval:** Not required
**Policy Rule:** Automatic permit

Use for routine, low-risk expenses: office supplies, subscriptions under $1k, travel under $3k.

```cedar
permit(principal, action == Action::"approve_budget", resource)
when {
  context.amount < 500000  // 500,000 cents = $5,000
};
```

### Tier 2: Supervisory Gate ($5k–$25k)

**Amount Range:** $5,000–$24,999
**Authority:** Must be supervisory or higher
**Human Approval:** Required
**Policy Rule:** Permit only if supervisory + human_approval_present

Use for team-level budgets: software licenses, contractor work under $15k, equipment purchases.

```cedar
permit(principal, action == Action::"approve_budget", resource)
when {
  principal.authority == "supervisory" &&
  context.amount >= 500000 &&
  context.amount < 2500000 &&
  context.human_approval_present == true
};
```

### Tier 3: Sovereign Gate ($25k–$100k)

**Amount Range:** $25,000–$99,999
**Authority:** Must be sovereign
**Human Approval:** Required
**Policy Rule:** Permit only if sovereign + human_approval_present

Use for department-level budgets: annual licenses, major system upgrades, vendor partnerships.

```cedar
permit(principal, action == Action::"approve_budget", resource)
when {
  principal.authority == "sovereign" &&
  context.amount >= 2500000 &&
  context.amount < 10000000 &&
  context.human_approval_present == true
};
```

### Tier 4: Sovereign-Only (> $100k)

**Amount Range:** $100,000+
**Authority:** Must be sovereign
**Human Approval:** Any (not strictly required by policy, but recommend always yes)
**Policy Rule:** Permit only for sovereign authority; forbid all others

Use for strategic investments: new initiatives, major vendor commitments, infrastructure projects.

```cedar
permit(principal, action == Action::"approve_budget", resource)
when {
  principal.authority == "sovereign" &&
  context.amount >= 10000000
};

forbid(principal, action == Action::"approve_budget", resource)
when {
  context.amount >= 10000000 &&
  principal.authority != "sovereign"
};
```

## Modifying Thresholds

### Change Tier 1 limit (currently $5k)

Edit `budget-approval-policy.cedar`:

```cedar
// OLD: Tier 1 < $5,000
permit(principal, action == Action::"approve_budget", resource)
when {
  context.amount < 500000
};

// NEW: Tier 1 < $10,000
permit(principal, action == Action::"approve_budget", resource)
when {
  context.amount < 1000000  // 1,000,000 cents = $10,000
};
```

Then update the executor's `determine_tier()` function:

```rust
fn determine_tier(amount_minor: i64) -> u8 {
    if amount_minor < 1000000 {  // Changed from 500_000
        1
    } else if amount_minor < 2_500_000 {
        2
    } else if amount_minor < 10_000_000 {
        3
    } else {
        4
    }
}
```

### Add a 5th Tier (> $1M requires board approval)

1. Add new permit rule in Cedar:

```cedar
permit(principal, action == Action::"approve_budget", resource)
when {
  principal.authority == "sovereign" &&
  context.amount >= 100000000 &&  // $1M
  context.has_board_approval == true
};
```

2. Extend `BudgetApprovalRequest` struct to include `has_board_approval` field
3. Add new `determine_tier()` logic for Tier 5
4. Update all tests and seed data
5. Rebuild and re-test

### Add department-specific rules

Example: Finance dept gets special $50k auto-approval (no HITL):

```cedar
// Finance department gets special tier
permit(principal, action == Action::"approve_budget", resource)
when {
  context.department == "finance" &&
  context.amount >= 500000 &&
  context.amount < 5000000  // $50k
};
```

Then extend `BudgetApprovalRequest` with a `department` field.

## HITL Approval Flow

When a Tier 2 or 3 request escalates:

1. **Request arrives** → Executor evaluates Cedar policy
2. **Policy says "Escalate"** → Executor returns `CriterionResult::Blocked`
3. **Engine pauses** → Emits `GateRequest` with `approval_ref: "approval:budget:{request_id}"`
4. **Human reviews** → Via external UI/system, approves or rejects
5. **GateDecision arrives** → Engine re-evaluates criteria
6. **Policy re-runs** → Now `human_approval_present == true`
7. **Decision promoted** → Criterion becomes `Met` (if policy still permits)

## Seed Data

Six requests in `seed-requests.json`:

1. **office-supplies-001** — $1,200 (Tier 1, auto-approve)
2. **software-licenses-002** — $22,000 (Tier 2, supervisory, human-approved)
3. **contractor-services-003** — $75,000 (Tier 3, sovereign, human-approved)
4. **infrastructure-004** — $150,000 (Tier 4, advisory authority, should reject)
5. **partner-api-005** — $4,990 (Tier 1 boundary, just under $5k)
6. **training-program-006** — $51,000 (Tier 2 boundary, just over $5k)

Load and run:

```bash
jq '.[]' examples/budget-approval/seed-requests.json | while read req; do
  curl -X POST http://localhost:8080/v1/truths/budget-approval/execute \
    -H "Content-Type: application/json" \
    -d "$req"
done
```

Each request will show:
- Tier determination
- Policy evaluation outcome
- HITL gate status (if Tier 2–3)
- Audit record created

## Design Patterns

This truth demonstrates:

- **Threshold-based gates** — Amount triggers approval tier
- **Authority hierarchy** — Different roles can approve different amounts
- **HITL integration** — Policy can escalate to humans for final decision
- **Audit trail** — Every decision recorded with full provenance
- **Idempotency** — Suggestors check if facts exist before creating
- **Cedar policy reuse** — Same policy file for all API calls
- **Amount precision** — ISO 4217: amounts stored in cents (amount_minor)

## Future Extensions

- **Recurring budgets** — Track approval for monthly/annual allocations
- **Delegation** — Approve on behalf of another authority via time-scoped token
- **Risk scoring** — Adjust tier based on vendor, department, or category risk
- **Cost center tracking** — Route approvals by cost center owner
- **Multi-level approval** — Require multiple signers for Tier 3+
- **Budget variance** — Alert if actual vs. approved differs >10%

## Troubleshooting

### "policy setup failed"

Check that `budget-approval-policy.cedar` is in the correct path:
```
examples/budget-approval/budget-approval-policy.cedar
```

### "no executor for truth: budget-approval"

Ensure the dispatcher in `crates/governance-server/src/truth_runtime/mod.rs` includes:
```rust
"budget-approval" => budget_approval::execute(store, &inputs, persist).await,
```

### Request stuck in "Blocked" state

A Tier 2–3 request is waiting for human approval. In production, implement a UI endpoint that:
1. Fetches pending gate requests
2. Shows request details to human
3. Posts approval/rejection decision
4. Engine re-evaluates and converges

### Tests fail with "trust not found"

Rebuild the truth definitions:
```bash
cd crates/governance-truths
cargo build
cd ../..
cargo test budget_approval_test
```

## Further Reading

- **Converge Engine** — See `converge-kernel` for criterion evaluation + HITL gates
- **Cedar Policy** — See `converge-policy` for policy as code fundamentals
- **Vendor Selection** — See `crates/governance-server/src/truth_runtime/vendor_selection.rs` for a full-stack example with multi-agent evaluation
- **Governance Kernel** — See `governance-kernel` for decision recording + audit trails

---

**Teaching Goal:** This truth teaches students how to implement real-world threshold-based approval workflows where policy gates escalate decisions to humans when needed, and how to audit those decisions for compliance.
