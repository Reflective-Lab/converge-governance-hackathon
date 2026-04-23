// Budget approval policy — tiered approval thresholds for vendor commitments.
//
// TRY: Adjust the dollar thresholds to see how HITL gates change.

// Auto-approve: under $10,000
permit(principal, action == Action::"commit", resource)
when {
  context.amount <= 10000
};

// Manager approval: $10,001 – $50,000
permit(principal, action == Action::"commit", resource)
when {
  context.amount > 10000 &&
  context.amount <= 50000 &&
  context.human_approval_present == true &&
  principal.authority == "supervisory"
};

// VP approval: $50,001 – $100,000
permit(principal, action == Action::"commit", resource)
when {
  context.amount > 50000 &&
  context.amount <= 100000 &&
  context.human_approval_present == true &&
  principal.authority == "sovereign"
};

// CFO approval: over $100,000
forbid(principal, action == Action::"commit", resource)
when {
  context.amount > 100000 &&
  principal.authority != "sovereign"
};
