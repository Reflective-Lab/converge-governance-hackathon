# Security And Compliance Requirements

## Required Controls

- Audit logs for every LLM request used in a governed decision.
- Retention policy documented for prompts, responses, and derived facts.
- Data residency and cross-border transfer reviewed before production use.
- PII handling documented before sensitive workloads are routed.
- Human approval or Cedar delegation required before final provider promotion.

## Policy Gates

| Gate | Rule |
|---|---|
| Evidence coverage | At least 80% of required evidence categories must be present |
| Risk class | Medium risk or lower can use delegated approval |
| Budget delta | Budget impact must be within 15% of approved envelope |
| Escalation | High ambiguity or unresolved security gaps require human approval |

## Cedar Delegation Candidate

```cedar
permit(
  principal in Group::"procurement_review",
  action == Action::"promote_ai_provider_recommendation",
  resource
) when {
  resource.evaluation == "AI Provider Evaluation" &&
  resource.risk <= "medium" &&
  resource.budget_delta <= 0.15 &&
  resource.evidence.coverage >= 0.80
};
```

## Explicit Non-Goals

- Do not select a provider solely by benchmark rank.
- Do not promote unverifiable vendor claims.
- Do not require human review for repeated low-risk patterns once Cedar delegation is approved.

