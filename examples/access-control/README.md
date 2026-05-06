# Access Control Truth — Role-Based Access with Delegation Tokens

This example demonstrates **role-based access control (RBAC)** integrated into the Converge governance platform. It teaches:

- **Identity + Role Binding** — Users have assigned roles (Viewer, Editor, Admin)
- **Resource Sensitivity Levels** — Resources have public/internal/confidential/secret classifications
- **Implicit Role Hierarchy** — Admin > Editor > Viewer (broader permissions at higher levels)
- **Time-Scoped Delegation Tokens** — Cryptographically signed, time-limited elevation of role
- **Policy-Driven Decisions** — Cedar policy engine enforces role-based permit/forbid rules
- **Complete Audit Trail** — Every access decision is logged with provenance and metadata

## Overview

### Resources

Four test resources representing realistic data governance scenarios:

| Resource ID | Name | Sensitivity | Owner | Use Case |
|------------|------|-------------|-------|----------|
| `budget-2026` | 2026 Budget Plan | Confidential | charlie | Financial planning (editors only) |
| `roadmap-public` | Public Roadmap | Public | charlie | Everyone can read |
| `salary-data` | Salary Information | Secret | charlie | Admin only |
| `api-keys` | API Keys and Credentials | Secret | charlie | Admin only |

### Users

Four test users with different base roles:

| User ID | Base Role | Permissions | Notes |
|---------|-----------|-------------|-------|
| alice | Editor | Read (pub/int/conf), Write (pub/int/conf) | Can handle confidential docs |
| bob | Viewer | Read (public only) | Read-only, limited access |
| charlie | Admin | Read/Write/Delete all | Full access |
| diana | Viewer | + Active delegation (editor, 4h) | Temporarily elevated to editor |

### Test Scenarios

Seven scenarios covering roles, delegations, and sensitivity levels:

1. **Viewer reads public** → Permit (baseline)
2. **Viewer cannot write** → Forbid (no write permission)
3. **Editor reads confidential** → Permit (role allows it)
4. **Editor writes confidential** → Permit (elevated permission)
5. **Admin deletes any** → Permit (admin privilege)
6. **Viewer with valid delegation writes** → Permit (delegation elevates role)
7. **Admin reads secret** → Permit (admin-only resource)

## How It Works

### Architecture

```
Request (user, resource, action)
    ↓
Role Assignment Suggestor
    ├─ Loads user→role mapping
    └─ Emits role:assignment:user-X facts
    ↓
Delegation Token Verify Suggestor (if token presented)
    ├─ Verify signature, time window, subject, resource scope
    └─ Emits delegation:verification:token-Y facts
    ↓
Access Policy Suggestor
    ├─ Cedar policy evaluation
    ├─ Base role + delegation elevation + resource sensitivity
    └─ Emits policy:decision:access-request-Z facts (permit/forbid)
    ↓
Access Audit Suggestor
    ├─ Record decision with full metadata
    └─ Emits audit:access:request-ID facts (immutable log)
    ↓
Decision Record (final output)
```

### Role Hierarchy

The role hierarchy is **implicit** in the Cedar policy rules:

```
Admin (highest)
├─ Can: read all, write all, delete all
├─ Can: read secret, confidential, internal, public
└─ Can: write confidential (unlike Editor)

Editor (middle)
├─ Can: read public, internal, confidential (NOT secret)
├─ Can: write public, internal, confidential
└─ Cannot: delete, read secret

Viewer (lowest)
├─ Can: read public, internal (if delegated to editor, can read confidential)
└─ Cannot: write, delete, read confidential/secret (without delegation)
```

### Delegation Token Lifecycle

1. **Issue** — Admin (charlie) creates a delegation token for diana
   - `granted_by: charlie`
   - `granted_to_user: diana`
   - `elevated_role: editor` (diana is viewer, token elevates to editor)
   - `valid_from/until` (e.g., 4 hours from now)
   - `signature` (Ed25519, hex-encoded)

2. **Present** — diana includes token in access request
   - Request: `(user: diana, resource: budget-2026, action: write)`
   - Token included as `presented_delegation_token` field

3. **Verify** — DelegationTokenVerifySuggestor checks:
   - ✓ Signature is non-empty (would verify Ed25519 in production)
   - ✓ Time window is valid (`now >= valid_from AND now <= valid_until`)
   - ✓ Subject matches (`granted_to_user == user_id`)
   - ✓ Resource scope (if token scoped, matches request resource)

4. **Evaluate** — AccessPolicySuggestor uses elevated role
   - Effective role becomes `editor` (from delegation)
   - Policy evaluates as if diana is editor
   - Decision: permit (editor can write confidential)

5. **Audit** — AccessAuditSuggestor logs with delegation metadata
   - Timestamp, decision, reason, token ID, granted_by, elevated_role

## Usage

### API Request Format

```bash
curl -X POST http://localhost:8080/v1/truths/access-control/execute \
  -H "Content-Type: application/json" \
  -d @request.json
```

### Request JSON

```json
{
  "request": {
    "user_id": "diana",
    "resource_id": "budget-2026",
    "action": "write",
    "presented_delegation_token": {
      "id": "token-diana-editor-temp",
      "granted_by": "charlie",
      "granted_to_user": "diana",
      "elevated_role": "editor",
      "valid_from": 1714320000,
      "valid_until": 1714334400,
      "max_operations": null,
      "reason": "Temporary editor access",
      "resource_id": null,
      "signature": "3045022100e8a3f2..."
    }
  },
  "user_roles": {
    "alice": "editor",
    "bob": "viewer",
    "charlie": "admin",
    "diana": "viewer"
  },
  "resources": [
    {
      "id": "budget-2026",
      "name": "2026 Budget Plan",
      "owner_id": "charlie",
      "sensitivity_level": "confidential"
    },
    ...
  ],
  "delegation_tokens": [
    { "id": "token-diana-editor-temp", ... }
  ]
}
```

### Test Scenario 1: Viewer Reads Public

```bash
curl -X POST http://localhost:8080/v1/truths/access-control/execute \
  -H "Content-Type: application/json" \
  -d '{
    "request": {
      "user_id": "bob",
      "resource_id": "roadmap-public",
      "action": "read",
      "presented_delegation_token": null
    },
    "user_roles": { "bob": "viewer", ... },
    "resources": [ { "id": "roadmap-public", "sensitivity_level": "public", ... }, ... ],
    "delegation_tokens": []
  }'
```

**Expected:** `converged: true`, decision = `Permit`

### Test Scenario 6: Viewer with Delegation Writes

```bash
curl -X POST http://localhost:8080/v1/truths/access-control/execute \
  -H "Content-Type: application/json" \
  -d '{
    "request": {
      "user_id": "diana",
      "resource_id": "budget-2026",
      "action": "write",
      "presented_delegation_token": { "id": "token-diana-editor-temp", ... }
    },
    "user_roles": { "diana": "viewer", ... },
    "resources": [ ... ],
    "delegation_tokens": [ { "id": "token-diana-editor-temp", ... } ]
  }'
```

**Expected:** `converged: true`, decision = `Permit` (delegation elevates diana to editor)

### Run Tests

```bash
cd /Users/kpernyer/dev/apps/vendor-selection
just test access_control_test
```

## How to Modify

### Add a New Role

1. Edit `/crates/governance-kernel/src/types/access.rs`:
   ```rust
   pub fn custom_role() -> Self {
       Role::new(
           "custom",
           "Custom Role",
           vec![Permission::Read, Permission::Write],
       )
   }
   ```

2. Update `AccessPolicySuggestor` in `/crates/governance-server/src/truth_runtime/access_control.rs`:
   ```rust
   "custom" => Role::custom(),
   ```

3. Add rules to `access-control-policy.cedar`:
   ```cedar
   permit(principal, action == Action::"read", resource) when {
       principal has role &&
       principal.role == "custom" &&
       resource.sensitivity_level == "internal"
   };
   ```

### Restrict Delegations

To prevent a role from being delegated:

**In Cedar policy:**
```cedar
forbid(principal, action == Action::"write", resource) when {
    principal has delegation_active &&
    principal.delegation_active == true &&
    principal has delegation_role &&
    principal.delegation_role == "viewer"  // prevent viewer delegation
};
```

### Add Department-Based Rules

1. Extend `AccessControlledResource`:
   ```rust
   pub department: Option<String>,
   ```

2. Add Cedar rule:
   ```cedar
   permit(principal, action == Action::"read", resource) when {
       principal has department &&
       resource has department &&
       principal.department == resource.department
   };
   ```

### Enforce Max Operations

Delegation tokens can limit usage:

```rust
token.max_operations = Some(5);  // valid for 5 read/write ops
```

Update `DelegationTokenVerifySuggestor` to track and enforce:

```rust
// Check operation counter in persistent store
let ops_used = token_usage_counter.get(&token_id).unwrap_or(0);
let ops_valid = ops_used < token.max_operations.unwrap_or(u32::MAX);
```

### Add Time-of-Day Restrictions

Extend Cedar policy with context time:

```cedar
permit(...) when {
    context has current_hour &&
    context.current_hour >= 9 &&    // business hours
    context.current_hour <= 17
};
```

## Learning Outcomes

After this example, students understand:

✅ **RBAC fundamentals** — Role-based access control models and role hierarchies  
✅ **Sensitivity levels** — Resource classification and implicit ordering  
✅ **Delegation patterns** — Time-scoped, cryptographically-verified role elevation  
✅ **Policy engines** — Cedar policy syntax and decision rules  
✅ **Audit integration** — Immutable logging of access decisions with provenance  
✅ **Governance-driven** — Access control as a convergence criterion, not an afterthought  

## Links

- **Truth definition:** `crates/governance-truths/src/lib.rs` (search `access-control`)
- **Executor:** `crates/governance-server/src/truth_runtime/access_control.rs` (700+ lines)
- **Domain types:** `crates/governance-kernel/src/types/access.rs`
- **Policy:** `examples/access-control/access-control-policy.cedar`
- **Seed data:** `examples/access-control/seed-users-and-resources.json`
- **HTTP API:** `crates/governance-server/src/http_api.rs` (routes `/v1/truths/access-control/execute`)

## Future Extensions

**For students or contributors:**

1. **Implement Ed25519 signature verification** — currently just checks non-empty signature
2. **Add delegation usage counter** — enforce max_operations by tracking in persistent store
3. **Department-based delegation** — restrict delegation to within department
4. **Resource ownership chain** — owner can grant permissions, not just admins
5. **Time-of-day access** — read/write only during business hours
6. **Approval workflows** — delegation requests go to approvers before granting
7. **Multi-factor delegation** — require multiple signatures for sensitive grants
8. **Delegation revocation** — explicit revoke before token expiry
9. **Context-based access** — IP-based, location-based, device-based rules
10. **Ephemeral roles** — one-time-use temporary roles for contractors

---

**This example is part of the Converge governance hackathon teaching suite. After this, students can apply delegation and role patterns to their own governance domains.**
