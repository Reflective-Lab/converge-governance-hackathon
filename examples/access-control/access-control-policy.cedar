// Role-Based Access Control Policy with Delegation Support
//
// This policy demonstrates:
// - Role-based permit rules (viewer, editor, admin)
// - Implicit role hierarchy (admin > editor > viewer)
// - Resource sensitivity levels (public, internal, confidential, secret)
// - Time-scoped delegation token elevation
// - Audit-friendly decision reasoning
//
// Learning objectives:
// - Cedar policy syntax for RBAC
// - Sensitivity-aware access control
// - Delegation token integration
// - Rule ordering and specificity

// =========================================================================
// PERMIT RULES — Allow access for granted roles/delegations
// =========================================================================

// Rule 1: Everyone can read public resources
permit(
    principal,
    action == Action::"read",
    resource
) when {
    resource.sensitivity_level == "public"
};

// Rule 2: Viewer can read internal resources (and public, by Rule 1)
permit(
    principal,
    action == Action::"read",
    resource
) when {
    principal has role &&
    principal.role == "viewer" &&
    (resource.sensitivity_level == "public" || resource.sensitivity_level == "internal")
};

// Rule 3: Editor can read public, internal, and confidential
permit(
    principal,
    action == Action::"read",
    resource
) when {
    principal has role &&
    principal.role == "editor" &&
    (
        resource.sensitivity_level == "public" ||
        resource.sensitivity_level == "internal" ||
        resource.sensitivity_level == "confidential"
    )
};

// Rule 4: Editor can write to public and internal (read+write permission)
permit(
    principal,
    action == Action::"write",
    resource
) when {
    principal has role &&
    principal.role == "editor" &&
    (resource.sensitivity_level == "public" || resource.sensitivity_level == "internal")
};

// Rule 5: Editor can write confidential (elevated permission)
permit(
    principal,
    action == Action::"write",
    resource
) when {
    principal has role &&
    principal.role == "editor" &&
    resource.sensitivity_level == "confidential"
};

// Rule 6: Admin can read anything
permit(
    principal,
    action == Action::"read",
    resource
) when {
    principal has role &&
    principal.role == "admin"
};

// Rule 7: Admin can write anything
permit(
    principal,
    action == Action::"write",
    resource
) when {
    principal has role &&
    principal.role == "admin"
};

// Rule 8: Admin can delete anything
permit(
    principal,
    action == Action::"delete",
    resource
) when {
    principal has role &&
    principal.role == "admin"
};

// Rule 9: Delegation elevation — valid token grants elevated role temporarily
// TRY: Add max_operations constraint to limit token usage
// TRY: Add department-based delegation rules
permit(
    principal,
    action == Action::"read",
    resource
) when {
    principal has delegation_active &&
    principal.delegation_active == true &&
    principal has delegation_role &&
    context has delegation_time_valid &&
    context.delegation_time_valid == true &&
    (resource.sensitivity_level != "secret" || principal.delegation_role == "admin")
};

// Rule 10: Delegation elevation for write
permit(
    principal,
    action == Action::"write",
    resource
) when {
    principal has delegation_active &&
    principal.delegation_active == true &&
    principal has delegation_role &&
    principal.delegation_role == "editor" &&
    context has delegation_time_valid &&
    context.delegation_time_valid == true
};

// =========================================================================
// FORBID RULES — Explicitly deny certain access patterns
// =========================================================================

// Forbid 1: Secret documents require admin role (no delegation elevation allowed)
forbid(
    principal,
    action == Action::"read",
    resource
) when {
    resource.sensitivity_level == "secret" &&
    principal has role &&
    principal.role != "admin"
};

// Forbid 2: Delete requires admin role
forbid(
    principal,
    action == Action::"delete",
    resource
) when {
    principal has role &&
    principal.role != "admin"
};

// Forbid 3: Non-editors cannot write
forbid(
    principal,
    action == Action::"write",
    resource
) when {
    principal has role &&
    !(principal.role == "editor" || principal.role == "admin")
};

// Forbid 4: Viewer cannot write even with delegation
forbid(
    principal,
    action == Action::"write",
    resource
) when {
    principal has delegation_active &&
    principal.delegation_active == true &&
    principal has delegation_role &&
    principal.delegation_role == "viewer"
};

// =========================================================================
// POLICY COMMENTS FOR STUDENTS
// =========================================================================

// Q: Why is Rule 1 checked first?
// A: Cedar evaluates rules in order. Public access (Rule 1) is unrestricted,
//    so it's listed first for clarity and performance.

// Q: How does delegation override the base role?
// A: When a delegation_token is presented and verified, the context sets
//    delegation_active=true and delegation_role=elevated_role. Rules 9-10
//    check these fields to grant temporary elevated access.

// Q: Why forbid Rule 4 (viewer with delegation)?
// A: Even with a delegation token, a viewer role shouldn't be elevated to
//    write access (only to read). This demonstrates explicit constraints.

// Q: Can I add department-based rules?
// A: Yes! Add principal has department && context has required_department,
//    then check if principal.department == required_department.
//
//    Example:
//    permit(principal, action == Action::"read", resource) when {
//        principal has department &&
//        resource has required_department &&
//        principal.department == resource.required_department
//    };

// Q: How do I enforce approval workflows?
// A: Add context has approvals and check approvals list:
//
//    permit(...) when {
//        context has approvals &&
//        context.approvals.contains(principal)
//    };
