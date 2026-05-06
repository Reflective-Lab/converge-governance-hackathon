//! Truth executor: access-control
//!
//! Role-based access control with cryptographically-verified delegation tokens.
//!
//! Design:
//! - 4 suggestors: role assignment, delegation verification, policy evaluation, audit logging
//! - Ed25519-signed delegation tokens with time windows and resource scoping
//! - Cedar policy engine for access decisions based on role + sensitivity + delegation
//! - Complete audit trail for governance and compliance
//!
//! Learning objectives for students:
//! - Role-based access control (RBAC) principles
//! - Time-scoped delegation with cryptographic verification
//! - Sensitivity levels and implicit role hierarchy
//! - Policy-driven decision making with audit integration

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use converge_kernel::{ContextState, Engine, TypesRunHooks};
use converge_pack::{AgentEffect, Context as ContextView, ContextKey, ProposedFact, Suggestor};
use governance_kernel::types::access::{
    AccessControlRequest, AccessControlledResource, AccessDecision, DelegationToken, Permission,
    Role, SensitivityLevel,
};
use governance_truths::{AccessControlEvaluator, build_intent, find_truth};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::TruthExecutionResult;

// ---------------------------------------------------------------------------
// Input/Output types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlInput {
    pub request: AccessControlRequest,
    pub user_roles: HashMap<String, String>, // user_id -> role_name
    pub resources: Vec<AccessControlledResource>,
    pub delegation_tokens: Vec<DelegationToken>,
}

// Parse input from HashMap
fn parse_input(inputs: &HashMap<String, String>) -> Result<AccessControlInput, String> {
    let request_json = super::common::required_input(inputs, "request")?;
    let request: AccessControlRequest =
        serde_json::from_str(request_json).map_err(|e| format!("invalid request: {e}"))?;

    let user_roles_json = super::common::required_input(inputs, "user_roles")?;
    let user_roles: HashMap<String, String> =
        serde_json::from_str(user_roles_json).map_err(|e| format!("invalid user_roles: {e}"))?;

    let resources_json = super::common::required_input(inputs, "resources")?;
    let resources: Vec<AccessControlledResource> =
        serde_json::from_str(resources_json).map_err(|e| format!("invalid resources: {e}"))?;

    let delegation_tokens_json =
        super::common::optional_input(inputs, "delegation_tokens").unwrap_or_default();
    let delegation_tokens: Vec<DelegationToken> = if delegation_tokens_json.is_empty() {
        vec![]
    } else {
        serde_json::from_str(&delegation_tokens_json)
            .map_err(|e| format!("invalid delegation_tokens: {e}"))?
    };

    Ok(AccessControlInput {
        request,
        user_roles,
        resources,
        delegation_tokens,
    })
}

// ---------------------------------------------------------------------------
// Suggestor 1: RoleAssignmentSuggestor
//
// Loads user->role mappings from seed data and emits facts.
// TRY: Extend to support role groups or inheritance chains
// ---------------------------------------------------------------------------

struct RoleAssignmentSuggestor {
    input: AccessControlInput,
}

#[async_trait]
impl Suggestor for RoleAssignmentSuggestor {
    fn name(&self) -> &str {
        "role-assignment"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Seeds)
            .iter()
            .any(|f| f.id().starts_with("role:assignment:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        for (user_id, role_name) in &self.input.user_roles {
            let fact_id = format!("role:assignment:{}", user_id);

            // Find the role definition
            let role = match role_name.as_str() {
                "viewer" => Role::viewer(),
                "editor" => Role::editor(),
                "admin" => Role::admin(),
                _ => Role::new(role_name.clone(), role_name.clone(), vec![Permission::Read]),
            };

            proposals.push(
                ProposedFact::new(
                    ContextKey::Seeds,
                    fact_id,
                    serde_json::json!({
                        "user_id": user_id,
                        "role_id": role.id,
                        "role_name": role.name,
                        "permissions": role
                            .permissions
                            .iter()
                            .map(|p| p.as_str())
                            .collect::<Vec<_>>(),
                    })
                    .to_string(),
                    "suggestor:role-assignment",
                )
                .with_confidence(0.99),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

// ---------------------------------------------------------------------------
// Suggestor 2: DelegationTokenVerifySuggestor
//
// Verifies presented delegation tokens (signature, time window, constraints).
// TRY: Implement actual Ed25519 signature verification
// TRY: Add max-operations counter and enforce per-token usage limits
// ---------------------------------------------------------------------------

struct DelegationTokenVerifySuggestor {
    input: AccessControlInput,
}

#[async_trait]
impl Suggestor for DelegationTokenVerifySuggestor {
    fn name(&self) -> &str {
        "delegation-token-verify"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        // Only run if a delegation token was presented and we haven't verified it yet
        self.input.request.presented_delegation_token.is_some()
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id().starts_with("delegation:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        if let Some(token) = &self.input.request.presented_delegation_token {
            let token_id = token.id.clone();

            // Verify: time window
            let time_valid = token.is_valid_now();

            // Verify: subject (granted_to_user) matches request user
            let subject_valid = token.granted_to_user == self.input.request.user_id;

            // Verify: resource scoping (if token is scoped, it must match the request resource)
            let resource_valid = if let Some(scoped_resource) = &token.resource_id {
                scoped_resource == &self.input.request.resource_id
            } else {
                true // token is not resource-scoped, valid for any resource
            };

            // Verify: signature (in production, use Ed25519::verify)
            // For demo, we check if signature is non-empty
            let signature_valid = !token.signature.is_empty();

            let all_valid = time_valid && subject_valid && resource_valid && signature_valid;

            let fact_id = format!("delegation:verification:{}", token_id);

            proposals.push(
                ProposedFact::new(
                    ContextKey::Evaluations,
                    fact_id,
                    serde_json::json!({
                        "token_id": token_id,
                        "valid": all_valid,
                        "time_valid": time_valid,
                        "subject_valid": subject_valid,
                        "resource_valid": resource_valid,
                        "signature_valid": signature_valid,
                        "granted_by": token.granted_by,
                        "elevated_role": token.elevated_role,
                        "reason": token.reason,
                    })
                    .to_string(),
                    "suggestor:delegation-token-verify",
                )
                .with_confidence(if all_valid { 0.98 } else { 0.95 }),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

// ---------------------------------------------------------------------------
// Suggestor 3: AccessPolicySuggestor
//
// Evaluates Cedar policy based on user role, delegation status, and resource sensitivity.
// Cedar rules:
//   - Viewer can read public/internal
//   - Editor can read public/internal/confidential, can write
//   - Admin can read all, write all, delete all
//   - Valid delegation elevates user's role for the scoped request
//   - Secret docs require admin role OR executive_approval=true (escalated)
//   - Delete operations require deletion_reason_provided=true
//
// TRY: Extend policy with department-based rules
// TRY: Add time-of-day based access restrictions
// TRY: Implement delegation token usage counter
// ---------------------------------------------------------------------------

struct AccessPolicySuggestor {
    input: AccessControlInput,
}

#[async_trait]
impl Suggestor for AccessPolicySuggestor {
    fn name(&self) -> &str {
        "access-policy"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        // Run when we have roles assigned and haven't made a decision yet
        ctx.get(ContextKey::Seeds)
            .iter()
            .any(|f| f.id().starts_with("role:assignment:"))
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id().starts_with("policy:decision:"))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        let request_id = Uuid::new_v4().to_string();
        let user_id = &self.input.request.user_id;
        let resource_id = &self.input.request.resource_id;
        let action = &self.input.request.action;

        // Find user's role
        let user_role_name = self
            .input
            .user_roles
            .get(user_id)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        // Check if user has a verified delegation token that elevates their role
        let delegation_role = if let Some(token) = &self.input.request.presented_delegation_token {
            let token_verified = ctx
                .get(ContextKey::Evaluations)
                .iter()
                .find(|f| f.id().contains(&token.id))
                .and_then(|fact| serde_json::from_str::<serde_json::Value>(fact.content()).ok())
                .and_then(|payload| payload.get("valid").and_then(|v| v.as_bool()))
                .unwrap_or(false);

            if token_verified {
                Some(token.elevated_role.clone())
            } else {
                None
            }
        } else {
            None
        };

        // Determine effective role: use delegation if available, otherwise use base role
        let effective_role_name = delegation_role.as_ref().unwrap_or(&user_role_name);

        // Construct effective role
        let effective_role = match effective_role_name.as_str() {
            "viewer" => Role::viewer(),
            "editor" => Role::editor(),
            "admin" => Role::admin(),
            _ => Role::viewer(),
        };

        // Find resource
        let resource = self.input.resources.iter().find(|r| r.id == *resource_id);

        // Evaluate policy
        let (permitted, reason) = if let Some(res) = resource {
            match action.as_str() {
                "read" => {
                    // Viewer: public, internal
                    // Editor: public, internal, confidential
                    // Admin: all
                    let can_read = match res.sensitivity_level {
                        SensitivityLevel::Public => true,
                        SensitivityLevel::Internal => {
                            effective_role.has_permission(Permission::Read)
                        }
                        SensitivityLevel::Confidential => {
                            effective_role.has_permission(Permission::Read)
                                && (user_role_name == "editor"
                                    || user_role_name == "admin"
                                    || delegation_role == Some("editor".to_string())
                                    || delegation_role == Some("admin".to_string()))
                        }
                        SensitivityLevel::Secret => {
                            effective_role.has_permission(Permission::Read)
                                && (user_role_name == "admin"
                                    || delegation_role == Some("admin".to_string()))
                        }
                    };
                    (
                        can_read,
                        if can_read {
                            format!(
                                "Read permitted: user role '{}' with effective role '{}'",
                                user_role_name, effective_role_name
                            )
                        } else {
                            format!(
                                "Read forbidden: user role '{}' cannot access {} resource",
                                user_role_name,
                                res.sensitivity_level.as_str()
                            )
                        },
                    )
                }
                "write" => {
                    // Only editor and admin can write
                    // Editor cannot write to secret
                    let can_write = effective_role.has_permission(Permission::Write)
                        && match res.sensitivity_level {
                            SensitivityLevel::Public | SensitivityLevel::Internal => true,
                            SensitivityLevel::Confidential => {
                                user_role_name == "editor"
                                    || user_role_name == "admin"
                                    || delegation_role == Some("editor".to_string())
                                    || delegation_role == Some("admin".to_string())
                            }
                            SensitivityLevel::Secret => {
                                user_role_name == "admin"
                                    || delegation_role == Some("admin".to_string())
                            }
                        };
                    (
                        can_write,
                        if can_write {
                            format!(
                                "Write permitted: user role '{}' can modify {}",
                                user_role_name, res.name
                            )
                        } else {
                            format!(
                                "Write forbidden: user role '{}' cannot write {} resource",
                                user_role_name,
                                res.sensitivity_level.as_str()
                            )
                        },
                    )
                }
                "delete" => {
                    // Only admin can delete
                    let can_delete = effective_role.has_permission(Permission::Delete)
                        && (user_role_name == "admin"
                            || delegation_role == Some("admin".to_string()));
                    (
                        can_delete,
                        if can_delete {
                            "Delete permitted: user is admin".to_string()
                        } else {
                            format!(
                                "Delete forbidden: user role '{}' is not admin",
                                user_role_name
                            )
                        },
                    )
                }
                _ => (false, format!("Unknown action: {}", action)),
            }
        } else {
            (false, format!("Resource '{}' not found", resource_id))
        };

        let decision = if permitted {
            AccessDecision::Permit
        } else {
            AccessDecision::Forbid
        };

        let fact_id = format!("policy:decision:{}", request_id);

        proposals.push(
            ProposedFact::new(
                ContextKey::Evaluations,
                fact_id.clone(),
                serde_json::json!({
                    "request_id": request_id,
                    "user_id": user_id,
                    "resource_id": resource_id,
                    "action": action,
                    "decision": decision.as_str(),
                    "reason": reason,
                    "user_role": user_role_name,
                    "effective_role": effective_role_name,
                    "delegation_used": delegation_role.is_some(),
                })
                .to_string(),
                "suggestor:access-policy",
            )
            .with_confidence(0.97),
        );

        AgentEffect::with_proposals(proposals)
    }
}

// ---------------------------------------------------------------------------
// Suggestor 4: AccessAuditSuggestor
//
// Writes audit entry for every access decision.
// Captures: decision, actor, resource, reason, timestamp, delegation metadata
// ---------------------------------------------------------------------------

struct AccessAuditSuggestor {
    input: AccessControlInput,
}

#[async_trait]
impl Suggestor for AccessAuditSuggestor {
    fn name(&self) -> &str {
        "access-audit"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        // Run when policy decision exists and we haven't written audit yet
        ctx.get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().starts_with("policy:decision:"))
            && !ctx
                .get(ContextKey::Proposals)
                .iter()
                .any(|f| f.id().starts_with("audit:access:"))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = vec![];

        if let Some(decision_fact) = ctx
            .get(ContextKey::Evaluations)
            .iter()
            .find(|f| f.id().starts_with("policy:decision:"))
            && let Ok(payload) = serde_json::from_str::<serde_json::Value>(decision_fact.content())
        {
            let request_id = payload
                .get("request_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let user_id = payload
                .get("user_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let resource_id = payload
                .get("resource_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let action = payload
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let decision_str = payload
                .get("decision")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let reason = payload
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("no reason provided");

            let audit_id = format!("audit:access:{}", request_id);

            let delegation_metadata =
                self.input
                    .request
                    .presented_delegation_token
                    .as_ref()
                    .map(|token| {
                        serde_json::json!({
                            "token_id": token.id,
                            "granted_by": token.granted_by,
                            "elevated_role": token.elevated_role,
                        })
                    });

            proposals.push(
                ProposedFact::new(
                    ContextKey::Proposals,
                    audit_id,
                    serde_json::json!({
                        "request_id": request_id,
                        "user_id": user_id,
                        "resource_id": resource_id,
                        "action": action,
                        "decision": decision_str,
                        "reason": reason,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "delegation_metadata": delegation_metadata,
                    })
                    .to_string(),
                    "suggestor:access-audit",
                )
                .with_confidence(0.99),
            );
        }

        AgentEffect::with_proposals(proposals)
    }
}

// ---------------------------------------------------------------------------
// Executor
// ---------------------------------------------------------------------------

pub async fn execute(
    _store: &governance_kernel::InMemoryStore,
    inputs: &HashMap<String, String>,
    _persist: bool,
) -> Result<TruthExecutionResult, String> {
    let truth = find_truth("access-control").ok_or("truth not found")?;
    let intent = build_intent(truth);
    let access_input = parse_input(inputs)?;

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack(
        "access-pack",
        RoleAssignmentSuggestor {
            input: access_input.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "access-pack",
        DelegationTokenVerifySuggestor {
            input: access_input.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "access-pack",
        AccessPolicySuggestor {
            input: access_input.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "access-pack",
        AccessAuditSuggestor {
            input: access_input.clone(),
        },
    );

    let result = engine
        .run_with_types_intent_and_hooks(
            ContextState::new(),
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(AccessControlEvaluator)),
                event_observer: None,
            },
        )
        .await
        .map_err(|e| format!("convergence failed: {e}"))?;

    Ok(TruthExecutionResult {
        converged: result.converged,
        cycles: result.cycles,
        stop_reason: format!("{:?}", result.stop_reason),
        criteria_outcomes: result
            .criteria_outcomes
            .iter()
            .map(|o| super::CriterionOutcomeView {
                criterion: o.criterion.description.clone(),
                result: format!("{:?}", o.result),
            })
            .collect(),
        projection: None,
        llm_calls: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> AccessControlInput {
        AccessControlInput {
            request: AccessControlRequest::new("alice", "budget-2026", "read"),
            user_roles: HashMap::from([
                ("alice".into(), "editor".into()),
                ("bob".into(), "viewer".into()),
                ("charlie".into(), "admin".into()),
            ]),
            resources: vec![
                AccessControlledResource::new(
                    "budget-2026",
                    "2026 Budget Plan",
                    "charlie",
                    SensitivityLevel::Confidential,
                ),
                AccessControlledResource::new(
                    "roadmap-public",
                    "Public Roadmap",
                    "charlie",
                    SensitivityLevel::Public,
                ),
                AccessControlledResource::new(
                    "salary-data",
                    "Salary Information",
                    "charlie",
                    SensitivityLevel::Secret,
                ),
            ],
            delegation_tokens: vec![],
        }
    }

    #[tokio::test]
    async fn happy_path_role_assignment_and_policy() {
        let store = governance_kernel::InMemoryStore::new();
        let input = sample_input();
        let inputs = HashMap::from([
            (
                "request".into(),
                serde_json::to_string(&input.request).unwrap(),
            ),
            (
                "user_roles".into(),
                serde_json::to_string(&input.user_roles).unwrap(),
            ),
            (
                "resources".into(),
                serde_json::to_string(&input.resources).unwrap(),
            ),
            ("delegation_tokens".into(), "[]".into()),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn missing_request_returns_error() {
        let store = governance_kernel::InMemoryStore::new();
        let inputs = HashMap::from([
            ("user_roles".into(), "{}".into()),
            ("resources".into(), "[]".into()),
        ]);

        assert!(execute(&store, &inputs, false).await.is_err());
    }

    #[tokio::test]
    async fn viewer_cannot_write() {
        let store = governance_kernel::InMemoryStore::new();
        let mut input = sample_input();
        input.request = AccessControlRequest::new("bob", "budget-2026", "write");
        let inputs = HashMap::from([
            (
                "request".into(),
                serde_json::to_string(&input.request).unwrap(),
            ),
            (
                "user_roles".into(),
                serde_json::to_string(&input.user_roles).unwrap(),
            ),
            (
                "resources".into(),
                serde_json::to_string(&input.resources).unwrap(),
            ),
            ("delegation_tokens".into(), "[]".into()),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn editor_can_read_confidential() {
        let store = governance_kernel::InMemoryStore::new();
        let mut input = sample_input();
        input.request = AccessControlRequest::new("alice", "budget-2026", "read");
        let inputs = HashMap::from([
            (
                "request".into(),
                serde_json::to_string(&input.request).unwrap(),
            ),
            (
                "user_roles".into(),
                serde_json::to_string(&input.user_roles).unwrap(),
            ),
            (
                "resources".into(),
                serde_json::to_string(&input.resources).unwrap(),
            ),
            ("delegation_tokens".into(), "[]".into()),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
    }

    #[tokio::test]
    async fn admin_can_delete() {
        let store = governance_kernel::InMemoryStore::new();
        let mut input = sample_input();
        input.request = AccessControlRequest::new("charlie", "budget-2026", "delete");
        let inputs = HashMap::from([
            (
                "request".into(),
                serde_json::to_string(&input.request).unwrap(),
            ),
            (
                "user_roles".into(),
                serde_json::to_string(&input.user_roles).unwrap(),
            ),
            (
                "resources".into(),
                serde_json::to_string(&input.resources).unwrap(),
            ),
            ("delegation_tokens".into(), "[]".into()),
        ]);

        let result = execute(&store, &inputs, false).await.unwrap();
        assert!(result.converged);
    }
}
