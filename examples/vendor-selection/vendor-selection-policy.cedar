// Vendor selection policy — governs the transition from recommendation to commitment.
//
// Modify thresholds below to experiment with different approval flows.

// --- Authority-based permits ---

// Anyone with supervisory authority can propose and validate vendors.
permit(principal, action == Action::"propose", resource)
when {
  principal.authority == "supervisory" ||
  principal.authority == "sovereign"
};

permit(principal, action == Action::"validate", resource)
when {
  principal.authority == "supervisory" ||
  principal.authority == "sovereign"
};

// Supervisory can commit if human approval is present and gates are met.
permit(principal, action == Action::"commit", resource)
when {
  principal.authority == "supervisory" &&
  context.human_approval_present == true &&
  context.required_gates_met == true
};

// Sovereign authority can always commit.
permit(principal, action == Action::"commit", resource)
when {
  principal.authority == "sovereign"
};

// --- Risk threshold forbids ---

// Block commitment when amount exceeds $50,000 without human approval.
// TRY: Change 50000 to 100000 to raise the HITL threshold.
forbid(principal, action == Action::"commit", resource)
when {
  context.amount > 50000 &&
  context.human_approval_present == false
};

// --- Compliance screening requirements ---

// Block commitment on contracts where required gates are not met.
forbid(principal, action == Action::"commit", resource)
when {
  context.required_gates_met == false &&
  context.commitment_type == "contract"
};
