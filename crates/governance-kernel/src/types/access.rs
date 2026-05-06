use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SensitivityLevel {
    Public,
    Internal,
    Confidential,
    Secret,
}

impl SensitivityLevel {
    pub fn as_str(&self) -> &str {
        match self {
            SensitivityLevel::Public => "public",
            SensitivityLevel::Internal => "internal",
            SensitivityLevel::Confidential => "confidential",
            SensitivityLevel::Secret => "secret",
        }
    }
}

impl std::fmt::Display for SensitivityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Delete,
    ShareWithRole,
    GrantTemporaryAccess,
}

impl Permission {
    pub fn as_str(&self) -> &str {
        match self {
            Permission::Read => "read",
            Permission::Write => "write",
            Permission::Delete => "delete",
            Permission::ShareWithRole => "share_with_role",
            Permission::GrantTemporaryAccess => "grant_temporary_access",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub permissions: Vec<Permission>,
}

impl Role {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        permissions: Vec<Permission>,
    ) -> Self {
        Role {
            id: id.into(),
            name: name.into(),
            permissions,
        }
    }

    pub fn viewer() -> Self {
        Role::new("viewer", "Viewer", vec![Permission::Read])
    }

    pub fn editor() -> Self {
        Role::new(
            "editor",
            "Editor",
            vec![Permission::Read, Permission::Write],
        )
    }

    pub fn admin() -> Self {
        Role::new(
            "admin",
            "Admin",
            vec![
                Permission::Read,
                Permission::Write,
                Permission::Delete,
                Permission::ShareWithRole,
                Permission::GrantTemporaryAccess,
            ],
        )
    }

    pub fn has_permission(&self, perm: Permission) -> bool {
        self.permissions.contains(&perm)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlledResource {
    pub id: String,
    pub name: String,
    pub owner_id: String,
    pub sensitivity_level: SensitivityLevel,
}

impl AccessControlledResource {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        owner_id: impl Into<String>,
        sensitivity_level: SensitivityLevel,
    ) -> Self {
        AccessControlledResource {
            id: id.into(),
            name: name.into(),
            owner_id: owner_id.into(),
            sensitivity_level,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccess {
    pub user_id: String,
    pub resource_id: String,
    pub role: Role,
    pub granted_at: String,              // ISO 8601 timestamp
    pub active_delegations: Vec<String>, // delegation token IDs
}

impl UserAccess {
    pub fn new(
        user_id: impl Into<String>,
        resource_id: impl Into<String>,
        role: Role,
        granted_at: impl Into<String>,
    ) -> Self {
        UserAccess {
            user_id: user_id.into(),
            resource_id: resource_id.into(),
            role,
            granted_at: granted_at.into(),
            active_delegations: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationToken {
    pub id: String,
    pub granted_by: String,      // user ID
    pub granted_to_user: String, // user ID
    pub elevated_role: String,   // role name (e.g., "editor")
    pub valid_from: i64,         // epoch seconds
    pub valid_until: i64,        // epoch seconds
    pub max_operations: Option<u32>,
    pub reason: String,
    pub resource_id: Option<String>, // scoped to resource if present
    pub signature: String,           // Ed25519 signature (hex-encoded)
}

impl DelegationToken {
    pub fn new(
        id: impl Into<String>,
        granted_by: impl Into<String>,
        granted_to_user: impl Into<String>,
        elevated_role: impl Into<String>,
        valid_from: i64,
        valid_until: i64,
        reason: impl Into<String>,
    ) -> Self {
        DelegationToken {
            id: id.into(),
            granted_by: granted_by.into(),
            granted_to_user: granted_to_user.into(),
            elevated_role: elevated_role.into(),
            valid_from,
            valid_until,
            max_operations: None,
            reason: reason.into(),
            resource_id: None,
            signature: String::new(),
        }
    }

    pub fn with_max_operations(mut self, max_ops: u32) -> Self {
        self.max_operations = Some(max_ops);
        self
    }

    pub fn with_resource_scope(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = signature.into();
        self
    }

    pub fn is_valid_now(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.valid_from <= now && now <= self.valid_until
    }

    pub fn is_valid_at(&self, timestamp: i64) -> bool {
        timestamp >= self.valid_from && timestamp <= self.valid_until
    }

    pub fn is_resource_scoped(&self) -> bool {
        self.resource_id.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlRequest {
    pub user_id: String,
    pub resource_id: String,
    pub action: String, // "read", "write", "delete"
    pub presented_delegation_token: Option<DelegationToken>,
}

impl AccessControlRequest {
    pub fn new(
        user_id: impl Into<String>,
        resource_id: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        AccessControlRequest {
            user_id: user_id.into(),
            resource_id: resource_id.into(),
            action: action.into(),
            presented_delegation_token: None,
        }
    }

    pub fn with_delegation_token(mut self, token: DelegationToken) -> Self {
        self.presented_delegation_token = Some(token);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub access_request_id: String,
    pub user_id: String,
    pub resource_id: String,
    pub action: String,
    pub decision: AccessDecision,
    pub reason: String,
    pub decision_made_by: String, // "role-assignment", "delegation-token", "policy-engine"
    pub timestamp: String,        // ISO 8601
    pub delegation_metadata: Option<DelegationMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessDecision {
    Permit,
    Forbid,
}

impl AccessDecision {
    pub fn as_str(&self) -> &str {
        match self {
            AccessDecision::Permit => "permit",
            AccessDecision::Forbid => "forbid",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationMetadata {
    pub token_id: String,
    pub granted_by: String,
    pub elevated_role: String,
}

impl DecisionRecord {
    pub fn new(
        access_request_id: impl Into<String>,
        user_id: impl Into<String>,
        resource_id: impl Into<String>,
        action: impl Into<String>,
        decision: AccessDecision,
        reason: impl Into<String>,
        decision_made_by: impl Into<String>,
    ) -> Self {
        DecisionRecord {
            access_request_id: access_request_id.into(),
            user_id: user_id.into(),
            resource_id: resource_id.into(),
            action: action.into(),
            decision,
            reason: reason.into(),
            decision_made_by: decision_made_by.into(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            delegation_metadata: None,
        }
    }

    pub fn with_delegation_metadata(mut self, metadata: DelegationMetadata) -> Self {
        self.delegation_metadata = Some(metadata);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_hierarchy_permissions() {
        let viewer = Role::viewer();
        let editor = Role::editor();
        let admin = Role::admin();

        assert!(viewer.has_permission(Permission::Read));
        assert!(!viewer.has_permission(Permission::Write));

        assert!(editor.has_permission(Permission::Read));
        assert!(editor.has_permission(Permission::Write));
        assert!(!editor.has_permission(Permission::Delete));

        assert!(admin.has_permission(Permission::Read));
        assert!(admin.has_permission(Permission::Write));
        assert!(admin.has_permission(Permission::Delete));
        assert!(admin.has_permission(Permission::ShareWithRole));
        assert!(admin.has_permission(Permission::GrantTemporaryAccess));
    }

    #[test]
    fn delegation_token_time_validity() {
        let now = chrono::Utc::now().timestamp();
        let token = DelegationToken::new(
            "token-1",
            "alice",
            "bob",
            "editor",
            now - 60,
            now + 3600,
            "temporary edit access",
        );
        assert!(token.is_valid_now());
        assert!(token.is_valid_at(now));

        let expired_token = DelegationToken::new(
            "token-2",
            "alice",
            "bob",
            "editor",
            now - 3600,
            now - 60,
            "expired token",
        );
        assert!(!expired_token.is_valid_now());
    }

    #[test]
    fn sensitivity_levels_order() {
        assert!(SensitivityLevel::Public < SensitivityLevel::Internal);
        assert!(SensitivityLevel::Internal < SensitivityLevel::Confidential);
        assert!(SensitivityLevel::Confidential < SensitivityLevel::Secret);
    }

    #[test]
    fn delegation_token_resource_scope() {
        let token = DelegationToken::new(
            "token-1",
            "alice",
            "bob",
            "editor",
            0,
            1000,
            "scoped access",
        )
        .with_resource_scope("budget-2026");

        assert!(token.is_resource_scoped());
        assert_eq!(token.resource_id, Some("budget-2026".into()));
    }

    #[test]
    fn access_control_request_builder() {
        let req = AccessControlRequest::new("alice", "doc-1", "read");
        assert_eq!(req.user_id, "alice");
        assert_eq!(req.resource_id, "doc-1");
        assert_eq!(req.action, "read");
        assert!(req.presented_delegation_token.is_none());
    }

    #[test]
    fn decision_record_with_metadata() {
        let metadata = DelegationMetadata {
            token_id: "token-1".into(),
            granted_by: "alice".into(),
            elevated_role: "editor".into(),
        };

        let record = DecisionRecord::new(
            "req-1",
            "bob",
            "doc-1",
            "write",
            AccessDecision::Permit,
            "delegation token verified",
            "delegation-token",
        )
        .with_delegation_metadata(metadata.clone());

        assert!(record.delegation_metadata.is_some());
        assert_eq!(record.delegation_metadata.unwrap().token_id, "token-1");
    }
}
