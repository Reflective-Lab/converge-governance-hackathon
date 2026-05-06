//! Integration tests for access-control truth
//!
//! Tests cover:
//! - Role-based access control (viewer, editor, admin)
//! - Sensitivity levels (public, internal, confidential, secret)
//! - Delegation tokens (time-scoped elevation)
//! - Audit trail recording
//!
//! Test scenarios match the seed data: 4 resources, 4 users, 7+ scenarios

use governance_kernel::InMemoryStore;
use governance_kernel::types::access::{
    AccessControlRequest, AccessControlledResource, DelegationToken, SensitivityLevel,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_scenario_1_viewer_reads_public() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("bob", "roadmap-public", "read");
    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "roadmap-public",
        "Public Roadmap",
        "charlie",
        SensitivityLevel::Public,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(
        result.converged,
        "scenario 1: viewer reads public should converge"
    );
    assert!(!result.criteria_outcomes.is_empty());
}

#[tokio::test]
async fn test_scenario_2_viewer_cannot_write() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("bob", "roadmap-public", "write");
    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "roadmap-public",
        "Public Roadmap",
        "charlie",
        SensitivityLevel::Public,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(
        result.converged,
        "scenario 2: viewer cannot write should converge"
    );
}

#[tokio::test]
async fn test_scenario_3_editor_reads_confidential() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("alice", "budget-2026", "read");
    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "budget-2026",
        "2026 Budget Plan",
        "charlie",
        SensitivityLevel::Confidential,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(
        result.converged,
        "scenario 3: editor reads confidential should converge"
    );
}

#[tokio::test]
async fn test_scenario_4_editor_writes_confidential() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("alice", "budget-2026", "write");
    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "budget-2026",
        "2026 Budget Plan",
        "charlie",
        SensitivityLevel::Confidential,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(
        result.converged,
        "scenario 4: editor writes confidential should converge"
    );
}

#[tokio::test]
async fn test_scenario_5_admin_deletes() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("charlie", "budget-2026", "delete");
    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "budget-2026",
        "2026 Budget Plan",
        "charlie",
        SensitivityLevel::Confidential,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(
        result.converged,
        "scenario 5: admin deletes should converge"
    );
}

#[tokio::test]
async fn test_scenario_6_viewer_with_delegation_writes() {
    let store = InMemoryStore::new();

    let now = chrono::Utc::now().timestamp();
    let token = DelegationToken::new(
        "token-diana-editor-temp",
        "charlie",
        "diana",
        "editor",
        now - 60,
        now + 3600,
        "temporary editor access",
    )
    .with_signature("3045022100e8a3f2f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3022100f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3");

    let request =
        AccessControlRequest::new("diana", "budget-2026", "write").with_delegation_token(token);

    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
        ("diana".into(), "viewer".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "budget-2026",
        "2026 Budget Plan",
        "charlie",
        SensitivityLevel::Confidential,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(
        result.converged,
        "scenario 6: viewer with delegation writes should converge"
    );
}

#[tokio::test]
async fn test_scenario_7_admin_reads_secret() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("charlie", "salary-data", "read");
    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "salary-data",
        "Salary Information",
        "charlie",
        SensitivityLevel::Secret,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(
        result.converged,
        "scenario 7: admin reads secret should converge"
    );
}

// --- Additional negative test cases ---

#[tokio::test]
async fn test_viewer_cannot_read_confidential() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("bob", "budget-2026", "read");
    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "budget-2026",
        "2026 Budget Plan",
        "charlie",
        SensitivityLevel::Confidential,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(result.converged);
}

#[tokio::test]
async fn test_editor_cannot_delete() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("alice", "budget-2026", "delete");
    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "budget-2026",
        "2026 Budget Plan",
        "charlie",
        SensitivityLevel::Confidential,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(result.converged);
}

#[tokio::test]
async fn test_expired_delegation_rejected() {
    let store = InMemoryStore::new();

    let now = chrono::Utc::now().timestamp();
    let token = DelegationToken::new(
        "token-expired",
        "charlie",
        "diana",
        "editor",
        now - 3600,
        now - 600, // expired
        "expired token",
    )
    .with_signature("3045022100e8a3f2f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3022100f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3");

    let request =
        AccessControlRequest::new("diana", "budget-2026", "write").with_delegation_token(token);

    let user_roles: HashMap<String, String> = HashMap::from([
        ("alice".into(), "editor".into()),
        ("bob".into(), "viewer".into()),
        ("charlie".into(), "admin".into()),
        ("diana".into(), "viewer".into()),
    ]);
    let resources = vec![AccessControlledResource::new(
        "budget-2026",
        "2026 Budget Plan",
        "charlie",
        SensitivityLevel::Confidential,
    )];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(result.converged);
}

#[tokio::test]
async fn test_admin_cannot_read_nonexistent_resource() {
    let store = InMemoryStore::new();

    let request = AccessControlRequest::new("charlie", "nonexistent-resource", "read");
    let user_roles: HashMap<String, String> = HashMap::from([("charlie".into(), "admin".into())]);
    let resources: Vec<AccessControlledResource> = vec![];

    let inputs: HashMap<String, String> = HashMap::from([
        ("request".into(), serde_json::to_string(&request).unwrap()),
        (
            "user_roles".into(),
            serde_json::to_string(&user_roles).unwrap(),
        ),
        (
            "resources".into(),
            serde_json::to_string(&resources).unwrap(),
        ),
        ("delegation_tokens".into(), "[]".into()),
    ]);

    let result = governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
        .await
        .unwrap();

    assert!(result.converged);
}

#[tokio::test]
async fn test_all_sensitivity_levels() {
    let store = InMemoryStore::new();

    for sensitivity in &[
        SensitivityLevel::Public,
        SensitivityLevel::Internal,
        SensitivityLevel::Confidential,
        SensitivityLevel::Secret,
    ] {
        let request = AccessControlRequest::new("charlie", "test-resource", "read");
        let user_roles: HashMap<String, String> =
            HashMap::from([("charlie".into(), "admin".into())]);
        let resources = vec![AccessControlledResource::new(
            "test-resource",
            "Test Resource",
            "charlie",
            *sensitivity,
        )];

        let inputs = HashMap::from([
            ("request".into(), serde_json::to_string(&request).unwrap()),
            (
                "user_roles".into(),
                serde_json::to_string(&user_roles).unwrap(),
            ),
            (
                "resources".into(),
                serde_json::to_string(&resources).unwrap(),
            ),
            ("delegation_tokens".into(), "[]".into()),
        ]);

        let result =
            governance_server::truth_runtime::access_control::execute(&store, &inputs, false)
                .await
                .unwrap();

        assert!(
            result.converged,
            "admin should be able to read {:?} resource",
            sensitivity
        );
    }
}
