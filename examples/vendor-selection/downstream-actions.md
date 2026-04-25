# Downstream Actions

The demo does not end with a purchase order.

It creates a decision package that can move into the normal procurement path.

## Decision Package

The package should contain:

- recommended vendor or router strategy
- ranked alternatives and rejected candidates
- compliance, price, and risk evidence
- assumptions and hard constraints
- HITL/Cedar gate outcome
- open issues for legal, security, finance, or implementation
- audit trail showing which facts were promoted and why

## Real Procurement Follow-Through

1. Management reviews the recommendation and alternatives.
2. Procurement runs clarification or best-and-final-offer if needed.
3. Legal and security validate contract terms, data handling, SLAs, and regulatory exposure.
4. Finance validates budget and commercial impact.
5. A PoC or pilot may be required before final award.
6. Contracting formalizes the vendor selection.
7. Onboarding turns the selected option into an implementation plan.
8. Outcomes feed back into the learning registry for the next decision.

## Open Questions For The Team

- Which inputs should Helm create, validate, or request from humans?
- Which gates should remain HITL for the demo?
- Which gates can be represented as Cedar delegation candidates?
- What evidence is needed before the router/provider-mix recommendation becomes trustworthy?
- How do we show changed data producing a changed but still explainable outcome?
