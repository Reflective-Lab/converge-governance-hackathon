---
tags: [development, policies, cedar]
---
# Cedar Policies for Participants

A practical guide to reading, understanding, and modifying Cedar policies in Converge governance. You don't need to be a policy expert — just follow the patterns.

## What is Cedar?

Cedar is Amazon's open-source language for authorization policies. Think of it as a SQL WHERE clause, but for access control: given a principal (who), an action (what), and context (circumstances), does the policy allow it?

Learn more at https://cedarpolicy.com.

In Converge, Cedar policies sit between your agents' recommendations and the final decision. A policy answers: "Should this vendor move to commitment? Should this spend be approved? Should this fact be promoted?" Cedar ensures human oversight rules are enforced consistently across all decisions.

## The Three Outcomes

Every Cedar policy evaluation has one of three outcomes:

| Outcome | Meaning |
|---------|---------|
| **permit** | Action is allowed — proceeds normally |
| **forbid** | Action is explicitly denied — blocked |
| **silence** | No matching rule — default deny (action blocked) |

Key insight: **permit wins, unless forbid matches first**. If you have both a permit clause and a forbid clause that match the same request, forbid wins.

```cedar
// Permit example: allow supervisors to propose
permit(principal, action == Action::"propose", resource)
when {
  principal.authority == "supervisory"
};

// Forbid example: block high-spend proposals without approval
forbid(principal, action == Action::"propose", resource)
when {
  context.amount > 100000 &&
  context.human_approval == false
};
```

## How Converge Uses Cedar

When you run a governed truth (like vendor selection), here's the flow:

```
Converge Truth Definition
         ↓
  [Agents propose facts]
         ↓
  [Cedar policy evaluates]
         ↓
Permit → Fact promoted to truth
Forbid → Fact rejected
Silence → Fact rejected (default deny)
         ↓
  [Audit trail records decision]
```

For this hackathon, policies are compiled into the governance server at startup. That means:
- Read policies in `examples/policy-{name}/`
- Modify a policy file
- Restart the server
- Re-run the demo to see the new outcome

The policy is part of the truth definition—it's how you define what "authorize vendor commitment" actually means.

## Reading the Vendor Selection Policy

Let's walk through [`examples/vendor-selection/vendor-selection-policy.cedar`](../examples/vendor-selection/vendor-selection-policy.cedar) step by step.

### Parts of a policy clause

Every Cedar clause has this structure:

```cedar
permit(principal, action == Action::"commit", resource)
when {
  principal.authority == "supervisory" &&
  context.human_approval_present == true
};
```

Breaking it down:
- **principal** — who is taking the action (a user or agent)
- **action** — what they're trying to do (propose, validate, commit, etc.)
- **resource** — what they're acting on (the vendor, the commitment decision)
- **context** — circumstances: amounts, approval flags, gates met, etc.

The `when` block is a condition: all conditions must be true (AND logic) for the clause to match.

### Authority levels

The hackathon uses four authority levels (you can add more!):

| Level | Can propose | Can validate | Can commit |
|-------|------------|-------------|-----------|
| **advisory** | ✓ | ✗ | ✗ |
| **supervisory** | ✓ | ✓ | ✓ (with approval + gates) |
| **participatory** | ✓ | ✓ | ✗ |
| **sovereign** | ✓ | ✓ | ✓ (always) |

From the policy:

```cedar
// Supervisory can commit only if gates are met AND human approved
permit(principal, action == Action::"commit", resource)
when {
  principal.authority == "supervisory" &&
  context.human_approval_present == true &&
  context.required_gates_met == true
};

// Sovereign can always commit
permit(principal, action == Action::"commit", resource)
when {
  principal.authority == "sovereign"
};
```

Translation: Supervisors need human sign-off and compliance gates passed. Sovereigns don't—they have final authority.

### Context attributes

The vendor selection policy checks three context flags:

- `context.amount` — the dollar amount of the commitment (number)
- `context.human_approval_present` — did a human explicitly approve? (true/false)
- `context.required_gates_met` — did the commitment pass compliance checks? (true/false)
- `context.commitment_type` — what type: "contract" or "spend"

These come from Converge's suggestors. If a suggestor doesn't populate a context field, the policy can't check it.

### Risk thresholds

```cedar
// Block commitment when amount exceeds $50,000 without human approval.
forbid(principal, action == Action::"commit", resource)
when {
  context.amount > 50000 &&
  context.human_approval_present == false
};
```

Translation: If the deal is big AND no human has signed off, block it. This is a safety gate.

**TRY THIS:** Change `50000` to `100000`. Now deals up to $100K don't require human approval—only bigger ones do. Restart the server and re-run the demo to see the difference.

### Compliance gates

```cedar
// Block commitment on contracts where required gates are not met.
forbid(principal, action == Action::"commit", resource)
when {
  context.required_gates_met == false &&
  context.commitment_type == "contract"
};
```

Translation: Contracts must pass compliance checks. Spend-type commitments don't have this requirement (only contracts do).

This pattern is useful for differentiating how policies apply to different deal types.

## Three Common Patterns

### Pattern 1: Authority Gates

**What it does:** Only certain roles can do certain actions.

```cedar
permit(principal, action == Action::"validate", resource)
when {
  principal.authority == "supervisory" ||
  principal.authority == "sovereign"
};
```

**When to use:** Restricting actions to senior roles (CTO reviews security, CFO reviews spend, etc.).

**In vendor-selection-policy.cedar:** Lines 14–18 (validate action).

**Modify it:** Add another authority level. Change `||` (OR) to include "participatory":
```cedar
permit(principal, action == Action::"validate", resource)
when {
  principal.authority == "supervisory" ||
  principal.authority == "participatory" ||
  principal.authority == "sovereign"
};
```

Now participatory users can also validate. Restart and test.

### Pattern 2: Amount Thresholds

**What it does:** Different rules for big vs. small amounts.

```cedar
forbid(principal, action == Action::"commit", resource)
when {
  context.amount > 50000 &&
  context.human_approval_present == false
};
```

**When to use:** "Deals under $10K don't need approval. Deals under $100K need one approver. Deals over $100K need two."

**In vendor-selection-policy.cedar:** Lines 36–42.

**Modify it:** Raise the threshold. Change `50000` to `150000`. Now vendors up to $150K don't need human approval (only bigger deals do). The impact: more deals skip HITL, faster decision velocity, more risk.

### Pattern 3: Compliance Requirements

**What it does:** Gate actions on whether prerequisites are met.

```cedar
forbid(principal, action == Action::"commit", resource)
when {
  context.required_gates_met == false &&
  context.commitment_type == "contract"
};
```

**When to use:** "Contracts must pass legal review before commitment." "No spend without budget sign-off." "Hiring needs manager approval."

**In vendor-selection-policy.cedar:** Lines 46–50.

**Modify it:** Add another type. Change the condition to also forbid "spend" type:
```cedar
forbid(principal, action == Action::"commit", resource)
when {
  context.required_gates_met == false &&
  (context.commitment_type == "contract" ||
   context.commitment_type == "spend")
};
```

Now both contracts AND spend commitments require gates to be met.

## Modifying Policies Safely

Here's a step-by-step walkthrough: let's raise the approval threshold from $50K to $75K.

### Step 1: Locate the policy

Open [`examples/vendor-selection/vendor-selection-policy.cedar`](../examples/vendor-selection/vendor-selection-policy.cedar).

### Step 2: Find the line to change

Look for the forbid clause with the amount check:

```cedar
forbid(principal, action == Action::"commit", resource)
when {
  context.amount > 50000 &&
  context.human_approval_present == false
};
```

### Step 3: Make the change

Change `50000` to `75000`:

```cedar
forbid(principal, action == Action::"commit", resource)
when {
  context.amount > 75000 &&
  context.human_approval_present == false
};
```

### Step 4: Save and restart

```bash
# Stop the running server (Ctrl+C if running locally)
# Restart:
just run-governance-server
```

### Step 5: Test with the demo

Re-run the vendor selection demo. Try committing vendors at different price points:
- $50,000 without approval — now **allowed** (was forbidden)
- $75,000 without approval — now **allowed** (was forbidden)
- $100,000 without approval — still **forbidden** (as before)

### Step 6: Observe the outcome

Look at the audit trail. The decision record will show which clause matched and why. If the outcome is unexpected, check:
- Did the server restart successfully?
- Are the context values being populated (check logs)?
- Is the syntax correct (missing semicolon, typo)?

## Debugging Policy Evaluation

If a policy decision is unexpected, follow this checklist:

### 1. Check the audit trail

Every decision is logged. Find the decision record for the action that surprised you. It will show:
- Which clause matched (permit/forbid)
- Why it matched
- The values of principal, action, context

```
Decision: forbid
Reason: context.amount > 50000 && context.human_approval_present == false
context.amount = 75000
context.human_approval_present = false
```

This tells you exactly why the action was blocked.

### 2. Verify context attributes are populated

If a context field isn't set by the suggestor, Cedar can't use it. Check:
- Is `context.human_approval_present` actually true/false? (Or missing?)
- Is `context.amount` a number? (Or a string?)
- Are attribute names spelled correctly in the policy?

Typos are the #1 debugging issue. Compare your policy to the reference:
```cedar
// Correct:
context.human_approval_present == true

// Wrong (won't match):
context.human_approval == true          // typo: missing "_present"
context.Human_Approval_Present == true  // typo: capitalization
```

### 3. Check rule ordering

In Cedar, the **first matching rule wins**. If you have:

```cedar
permit(principal, action == Action::"commit", resource)
when { true };

forbid(principal, action == Action::"commit", resource)
when { true };
```

The permit matches first, so the action is allowed. The forbid never gets a chance to block. Reorder to forbid first if you want to block by default:

```cedar
forbid(principal, action == Action::"commit", resource)
when { true };

permit(principal, action == Action::"commit", resource)
when { /* specific conditions */ };
```

### 4. Common syntax errors

| Error | Fix |
|-------|-----|
| Missing semicolon at end of clause | Add `;` after the closing `}` |
| Mismatched parens or braces | Count: `{` should equal `}` |
| Action name typo | Use exact action name (e.g., `Action::"commit"` not `Action::"COMMIT"`) |
| Boolean comparison wrong | Use `== true` or `== false`, not `== "true"` (string) |
| Unclosed string | Strings must be in quotes: `"value"` not `value` |

**Quick fix:** Open the reference policy [`examples/vendor-selection/vendor-selection-policy.cedar`](../examples/vendor-selection/vendor-selection-policy.cedar) and compare your syntax line by line.

## Safety Defaults

### Converge uses deny-by-default

If no permit rule matches, the action is **denied**. This is the secure default.

```cedar
permit(principal, action == Action::"propose", resource)
when { false };  // Never matches — all proposes are denied
```

Running this policy blocks all proposals. That's **safe**: nothing is accidentally allowed.

### What happens if you delete all permit rules?

Safe. Nothing is allowed, everything is denied. Users will see errors, and you'll debug quickly. Better than accidentally allowing something dangerous.

### The dangerous pattern: overly broad permit

```cedar
// Dangerous: permits anything
permit(principal, action == Action::"commit", resource)
when { true };

// Trying to block with forbid... but forbid loses
forbid(principal, action == Action::"commit", resource)
when { context.amount > 1000000 };
```

The permit matches first. The forbid never blocks anything. Don't structure it this way. Instead:

```cedar
// Permit only under specific conditions
permit(principal, action == Action::"commit", resource)
when {
  context.amount < 1000000 &&
  context.human_approval_present == true
};
```

Now high-amount commits are denied by default, and you don't rely on forbid losing.

### Best practice

1. Narrow permits — describe exactly who can do what
2. Use forbid for exceptions — "everyone except X"
3. Test incrementally — change one thing at a time, restart, verify

## Quick Reference: Cedar Syntax

This is a quick cheat sheet. For comprehensive Cedar syntax, see https://cedarpolicy.com/learn.

### Operators

| Operator | Example | Meaning |
|----------|---------|---------|
| `==` | `principal.authority == "supervisory"` | Equals |
| `!=` | `context.amount != 0` | Not equals |
| `<` | `context.amount < 50000` | Less than |
| `>` | `context.amount > 50000` | Greater than |
| `<=` | `context.amount <= 100000` | Less than or equal |
| `>=` | `context.amount >= 10000` | Greater than or equal |
| `&&` | `a == true && b == true` | AND (both must be true) |
| `\|\|` | `a == true \|\| b == true` | OR (one or both must be true) |
| `!` | `!context.flag` | NOT (true becomes false) |

### Types

| Type | Example |
|------|---------|
| String | `"supervisory"`, `"contract"` (in quotes) |
| Number | `50000`, `100` (integers only) |
| Boolean | `true`, `false` |
| List | `["advisory", "supervisory", "sovereign"]` (for `in` checks) |

### Common checks

```cedar
// String equality
principal.authority == "supervisory"

// Number comparison
context.amount > 50000

// Boolean flag
context.human_approval_present == true

// List membership
principal.authority in ["supervisory", "sovereign"]

// String contains
context.commitment_type contains "contract"

// Compound condition (AND)
principal.authority == "supervisory" &&
context.amount > 100000

// Compound condition (OR)
principal.authority == "supervisory" ||
principal.authority == "sovereign"

// Negation
!context.required_gates_met
```

## What NOT to Do

### Don't rely on forbid alone

```cedar
// Wrong: forbid doesn't prevent a matching permit
forbid(principal, action == Action::"commit", resource)
when { context.amount > 1000000 };

permit(principal, action == Action::"commit", resource)
when { true };  // This matches first — forbid is ignored
```

Instead, make permit specific:

```cedar
// Right: permit only safe cases
permit(principal, action == Action::"commit", resource)
when {
  context.amount <= 1000000
};
```

### Don't hardcode resource IDs

```cedar
// Wrong: only works for one vendor
permit(principal, action == Action::"commit", resource)
when {
  resource.id == "vendor-123"
};
```

Cedar policies should be generic. Let the truth definition route requests to the right policy based on action/authority/context, not hardcoded IDs.

### Don't change policy without restarting

The policy is compiled into the governance server at startup. If you edit a policy file and don't restart:

```bash
# Edit the policy file
vim examples/vendor-selection/vendor-selection-policy.cedar

# Old policy is still running — your edit does nothing!
```

Always restart after editing:

```bash
# Ctrl+C to stop
just run-governance-server
```

### Don't forget: context must be populated by Converge

If a suggestor doesn't set `context.human_approval_present`, Cedar can't check it:

```cedar
permit(principal, action == Action::"commit", resource)
when {
  context.human_approval_present == true  // Does nothing if not set
};
```

The suggestor must populate every context field the policy needs. Check the suggestor code (in `governance-server/src/truth_runtime/`) to see what context it provides.

## Next Steps

- **Edit a policy:** Try the "Modify Policies Safely" walkthrough above.
- **Add a new action:** Look at the `authorize-vendor-commitment` truth in `governance-server/src/truth_runtime/` and the policy in `examples/vendor-selection/`.
- **Understand context:** Read the suggestor code that populates context. See [[Development/Writing Suggestors]].
- **Learn Cedar deeply:** https://cedarpolicy.com/learn

Good luck with the hackathon!
