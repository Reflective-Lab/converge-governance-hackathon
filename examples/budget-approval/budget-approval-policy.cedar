// Budget approval policy — 4-tier threshold-based workflow with HITL gates.
//
// Tier 1 (<$5,000): Auto-approve — no human approval required.
// Tier 2 ($5k-$25k): Supervisory authority required + human approval required.
// Tier 3 ($25k-$100k): Sovereign authority required + human approval required.
// Tier 4 (>$100k): Sovereign authority only (no human approval required, but often given).
//
// TRY: Change 500000 to 1000000 to raise Tier 1 threshold to $10k.
// TRY: Change 2500000 to 5000000 to raise Tier 2/3 boundary to $50k.

// --- Tier 1: Auto-approve small budgets (<$5k, 500,000 cents) ---
// No conditions needed beyond amount; no authority, no human approval required.

permit(principal, action == Action::"approve_budget", resource)
when {
  context.amount < 500000
};

// --- Tier 2: Supervisory approval ($5k-$25k with human approval) ---

permit(principal, action == Action::"approve_budget", resource)
when {
  principal.authority == "supervisory" &&
  context.amount >= 500000 &&
  context.amount < 2500000 &&
  context.human_approval_present == true
};

// --- Tier 3: Sovereign approval ($25k-$100k with human approval) ---

permit(principal, action == Action::"approve_budget", resource)
when {
  principal.authority == "sovereign" &&
  context.amount >= 2500000 &&
  context.amount < 10000000 &&
  context.human_approval_present == true
};

// --- Tier 4: Sovereign only (>$100k) ---

permit(principal, action == Action::"approve_budget", resource)
when {
  principal.authority == "sovereign" &&
  context.amount >= 10000000
};

// --- Forbid rules (deny takes precedence if any forbid matches) ---

// Forbid: Tier 2 ($5k-$25k) without supervisory authority
forbid(principal, action == Action::"approve_budget", resource)
when {
  context.amount >= 500000 &&
  context.amount < 2500000 &&
  principal.authority != "supervisory" &&
  principal.authority != "sovereign"
};

// Forbid: Tier 2 ($5k-$25k) without human approval
forbid(principal, action == Action::"approve_budget", resource)
when {
  context.amount >= 500000 &&
  context.amount < 2500000 &&
  context.human_approval_present == false
};

// Forbid: Tier 3 ($25k-$100k) without sovereign authority
forbid(principal, action == Action::"approve_budget", resource)
when {
  context.amount >= 2500000 &&
  context.amount < 10000000 &&
  principal.authority != "sovereign"
};

// Forbid: Tier 3 ($25k-$100k) without human approval
forbid(principal, action == Action::"approve_budget", resource)
when {
  context.amount >= 2500000 &&
  context.amount < 10000000 &&
  context.human_approval_present == false
};

// Forbid: Tier 4 (>$100k) without sovereign authority
forbid(principal, action == Action::"approve_budget", resource)
when {
  context.amount >= 10000000 &&
  principal.authority != "sovereign"
};
