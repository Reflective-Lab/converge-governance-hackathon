# Executable JTBD And Converging Truth

## Executable JTBD

Evaluate AI provider strategy for governed agentic work so the buyer can select a provider mix that satisfies compliance, cost, capability, and resilience requirements.

## Truth

```gherkin
Truth: AI Provider Evaluation converges on a governed provider strategy
  Provider strategy must be selected from promoted evidence, not model preference.
  The decision must satisfy budget, compliance, and risk constraints.

Intent:
  Outcome: Select a governed AI provider strategy for agentic software work.
  Goal: Balance task fit, governance, cost, resilience, and adoption speed.

Authority:
  Actor: procurement_review
  Requires Approval: promote_ai_provider_recommendation

Constraint:
  Cost Limit: budget impact must stay within 15 percent of approved forecast.
  Must Not: promote a provider setup without audit and fallback rationale.

Evidence:
  Requires: rfi_rfp_requirements
  Requires: workload_profile
  Requires: security_compliance_requirements
  Requires: budget_token_forecast
  Requires: gateway_architecture_context
  Audit: ai_provider_evaluation_record

Scenario: Provider strategy is promoted only after convergence
  Given the buyer document package contains at least three required evidence categories
  And the executable JTBD is declared
  And the Cedar delegation either approves the gate or requires human review
  When the formation evaluates the provider strategy
  Then the system should recommend a provider mix or honestly stop
  And the recommendation should include promoted evidence and governance rationale
```

## Fixed Point Definition

The run can stop when no active agent can produce a new promotable fact under the current policy, evidence coverage, budget envelope, and authority context.

