// Vendor Selection Policy (Simple Edition) — gates the shortlist recommendation
//
// This policy controls whether a vendor recommendation can be promoted to
// commitment. Scores below thresholds trigger escalation or rejection.
//
// Modify thresholds below to experiment with different approval flows.
// TRY: Change 70 to 75 to raise quality threshold

// Promote recommendation if top vendor score >= 70
permit(principal, action == Action::"shortlist", resource)
when {
  context.top_score >= 70.0
};

// Escalate for human review if top score is 50-70 (needs review)
forbid(principal, action == Action::"shortlist", resource)
when {
  context.top_score >= 50.0 &&
  context.top_score < 70.0
};

// Block recommendation if top score < 50 (fails minimum threshold)
forbid(principal, action == Action::"shortlist", resource)
when {
  context.top_score < 50.0
};
