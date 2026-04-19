---
tags: [converge, organism, blueprints]
source: mixed
---
# Organism Blueprints

Organism maintains a library of ~22 production-shaped business cases modeled as converging truths. These are not toy examples — they represent the full quote-to-cash lifecycle of a SaaS business, from lead scoring through payment enforcement.

Any of these can be brought into the hackathon as additional example truths participants can study, modify, or use as templates for their own governance flows.

## Blueprint Categories

| Category | Truths | Notes |
|---|---|---|
| Sales Pipeline | score_inbound_fit, qualify_inbound_lead, plan_outbound_campaign, schedule_strategic_meetings | Lead scoring through meeting scheduling |
| Subscription Lifecycle | activate_subscription, upgrade_subscription_plan, renew_contract | Commercial commitment through expansion |
| Revenue & Metering | refill_prepaid_ai_credits, reconcile_model_usage_against_customer_ledger, detect_abnormal_token_burn, suspend_service_on_payment_failure | Credits, reconciliation, protection |
| Operations | create_customer_workspace | Provisioning linked to entitlements |
| Support | Incident resolution with escalation logic | |
| Expense Management | Expense report submission with receipt evidence and approval workflow | |
| Content & Marketing | Visual/tagline matching, brand signal monitoring | Design-phase, not yet executable |
| Demo/PoV | Converge browser extension demo, receipt OCR extraction | Partnership scenarios |

## Quote-to-Cash as Converging Truths

The core insight: quote-to-cash is not a pipeline — it is a directed graph of truths with policy constraints. No single truth owns the process end-to-end. The outcome emerges from all of them firing correctly with policies enforcing invariants at every boundary.

### The Flow

```
1. Discover & Qualify
   score_inbound_fit → qualify_inbound_lead
   Inbound signal scored against account context → fit assessment → qualified state with evidence + next owner

2. Engage
   plan_outbound_campaign → schedule_strategic_meetings
   Qualified leads → campaign planning within budget guardrails → ranked meeting slate scored by strategy alignment (human confirms)

3. Close & Activate
   activate_subscription → create_customer_workspace
   Accepted commitment → subscription activation (must resolve to valid catalog plan) → workspace provisioned with entitlements

4. Grow
   upgrade_subscription_plan · renew_contract
   Two parallel paths. Upgrades move entitlements with effective date. Renewals resolve terms against catalog. Both require approval for non-standard pricing.

5. Meter & Reconcile
   refill_prepaid_ai_credits → reconcile_model_usage_against_customer_ledger
   Credits consumed via metered usage. Reconciliation compares metered totals against ledger and flags unreconciled deltas above threshold.

6. Protect
   detect_abnormal_token_burn · suspend_service_on_payment_failure
   Continuous. Abnormal burn cites telemetry evidence and opens mitigation (hard-limit requires approval). Payment failure triggers graceful suspension after grace-period checks.
```

### Cross-Cutting Policies

Two policies act as invariants across the entire chain:

- **top_up_requires_confirmed_payment** — no credit grant without confirmed payment; manual override requires approval + audit
- **overdue_balance_blocks_entitlement_increase** — overdue accounts cannot expand; temporary relief is time-bound and approval-gated

### Module Truths

Two module truths enforce structural invariants:

- **active_subscription_requires_plan** — activation must resolve to a valid catalog plan with explicit entitlements
- **ledger_entry_is_immutable** — corrections are always new adjusting entries, never mutations

### Convergence Graph

```
score_inbound_fit ──→ qualify_inbound_lead ──→ plan_outbound_campaign
                                                      │
                                            schedule_strategic_meetings
                                                      │
                                            activate_subscription
                                              │ (requires valid plan)
                                       create_customer_workspace
                                          │              │
                              upgrade_subscription   renew_contract
                                          │              │
                              refill_prepaid_credits ←───┘
                                          │
                              reconcile_usage_vs_ledger
                                    (immutable ledger)

  ── always active ──────────────────────────
  detect_abnormal_token_burn
  suspend_service_on_payment_failure
  policy: top_up_requires_confirmed_payment
  policy: overdue_balance_blocks_entitlement_increase
```

## Bringing Blueprints into the Hackathon

Each blueprint is a self-contained truth definition with inputs, outputs, evidence requirements, and HITL gates. To port one into the hackathon repo:

1. Pick a blueprint that fits the team's challenge (e.g., expense approval for a finance governance demo)
2. Define the truth in `governance-truths/src/lib.rs` — key, packs, criteria
3. Create the executor in `governance-server/src/truth_runtime/`
4. Write suggestors implementing `converge_pack::Suggestor`
5. Add domain types to `governance-kernel` if needed
6. Wire in `truth_runtime/mod.rs`

The vendor selection truth in this repo is one node in this larger graph. Participants who want to go further can model multi-truth chains — for example, connecting vendor evaluation → subscription activation → workspace provisioning to show how governance decisions flow through a business.

### Good Candidates for Participant Projects

- **Expense report submission** — tangible, relatable, clear evidence requirements (receipts), approval workflow with thresholds
- **Subscription activation** — shows module truth enforcement (must have valid plan), connects naturally to the vendor selection outcome
- **Abnormal token burn detection** — continuous monitoring truth, good for teams interested in observability and protection
- **Inbound lead qualification** — scoring + evidence + human handoff, familiar domain for business students

## What Makes This Different

Traditional orchestration (Airflow, Step Functions, Temporal) runs a DAG of tasks. Converge runs a graph of truths where:

- Each truth declares what evidence it needs, not what steps to run
- Policies enforce invariants across truth boundaries
- The runtime stops honestly when confidence is too low
- Every decision has provenance — who proposed it, what evidence supported it, what policy gates it passed

The blueprints demonstrate this at business scale: 11 jobs, 2 policies, 2 modules, all converging into a coherent quote-to-cash outcome.

See also: [[Core Concepts]], [[Domain Packs]], [[../Domain/Truths]], [[../Development/Writing Truths]]
