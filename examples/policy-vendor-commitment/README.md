# Policy Vendor Commitment

This example shows `converge-policy` in a real business flow: a procurement or
governance actor attempts to commit a vendor recommendation, and the policy
engine returns one of three honest outcomes:

- `promote` — the commitment may proceed
- `escalate` — a human approval step is required
- `reject` — the commitment is not allowed

The example is wired into the local governance server as the
`authorize-vendor-commitment` truth.

## Run It

Start the local server:

```bash
just server
```

Then execute one of the sample requests:

```bash
curl -s http://127.0.0.1:8080/v1/truths/authorize-vendor-commitment/execute \
  -H 'content-type: application/json' \
  -d @examples/policy-vendor-commitment/commit-approved.request.json
```

Swap in:

- `commit-approved.request.json`
- `commit-escalate.request.json`
- `commit-reject.request.json`

## Business Flow

This models a realistic procurement boundary:

1. A vendor recommendation already exists.
2. A principal with explicit authority tries to act on that recommendation.
3. `converge-policy` evaluates authority, amount, gates, and human approval.
4. The result is projected into the audit trail as a `DecisionRecord`.

The policy source for this example lives in
`vendor-commitment-policy.cedar`.
