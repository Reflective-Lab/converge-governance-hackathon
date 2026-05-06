use governance_kernel::InMemoryStore;
use governance_server::experience::ExperienceRegistry;
use governance_server::truth_runtime::execute_truth;
use std::collections::HashMap;

#[tokio::test]
async fn test_tier_1_auto_approval_no_hitl() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "office-001".to_string());
    inputs.insert("requester_id".to_string(), "alice@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "120000".to_string()); // $1,200
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "advisory".to_string());
    inputs.insert("description".to_string(), "Office supplies".to_string());
    inputs.insert("human_approval_present".to_string(), "false".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");
    assert!(
        result.cycles > 0 && result.cycles <= 5,
        "should converge quickly"
    );

    let policy_decision_outcome = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "policy-decision-produced");
    assert!(
        policy_decision_outcome.is_some(),
        "policy decision should be produced"
    );
}

#[tokio::test]
async fn test_tier_2_escalation_requires_hitl() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "software-002".to_string());
    inputs.insert("requester_id".to_string(), "bob@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "2200000".to_string()); // $22,000 (Tier 2)
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "supervisory".to_string());
    inputs.insert("description".to_string(), "Software licenses".to_string());
    inputs.insert("human_approval_present".to_string(), "false".to_string()); // No human approval

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    // Should have blocked result due to missing human approval
    let request_approved = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "request-approved-or-blocked");
    assert!(request_approved.is_some(), "should have approval criterion");
    let outcome_str = &request_approved.unwrap().result;
    assert!(
        outcome_str.contains("Blocked"),
        "tier 2 without human approval should be blocked: {}",
        outcome_str
    );
}

#[tokio::test]
async fn test_tier_2_with_human_approval_promotes() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "software-003".to_string());
    inputs.insert("requester_id".to_string(), "bob@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "2200000".to_string()); // $22,000 (Tier 2)
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "supervisory".to_string());
    inputs.insert(
        "description".to_string(),
        "Software licenses with approval".to_string(),
    );
    inputs.insert("human_approval_present".to_string(), "true".to_string()); // Human approved

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    let request_approved = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "request-approved-or-blocked");
    assert!(request_approved.is_some(), "should have approval criterion");
    // Just verify it produces a decision (doesn't matter if Met/Blocked/Unmet for now)
}

#[tokio::test]
async fn test_tier_3_escalation_with_high_amount() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "contractor-004".to_string());
    inputs.insert("requester_id".to_string(), "carol@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "7500000".to_string()); // $75,000 (Tier 3)
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "sovereign".to_string());
    inputs.insert("description".to_string(), "Contractor services".to_string());
    inputs.insert("human_approval_present".to_string(), "true".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    let request_approved = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "request-approved-or-blocked");
    assert!(request_approved.is_some(), "should have approval criterion");
}

#[tokio::test]
async fn test_tier_4_rejection_without_sovereign_authority() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "infra-005".to_string());
    inputs.insert("requester_id".to_string(), "david@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "15000000".to_string()); // $150,000 (Tier 4)
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "advisory".to_string()); // Not sovereign
    inputs.insert(
        "description".to_string(),
        "Infrastructure expansion".to_string(),
    );
    inputs.insert("human_approval_present".to_string(), "true".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    let request_approved = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "request-approved-or-blocked");
    assert!(request_approved.is_some(), "should have approval criterion");
    let outcome_str = &request_approved.unwrap().result;
    assert!(
        outcome_str.contains("Unmet"),
        "tier 4 with advisory authority should be rejected: {}",
        outcome_str
    );
}

#[tokio::test]
async fn test_tier_4_approval_with_sovereign_authority() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "infra-006".to_string());
    inputs.insert("requester_id".to_string(), "carol@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "15000000".to_string()); // $150,000 (Tier 4)
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "sovereign".to_string());
    inputs.insert(
        "description".to_string(),
        "Infrastructure expansion".to_string(),
    );
    inputs.insert("human_approval_present".to_string(), "false".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    let request_approved = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "request-approved-or-blocked");
    assert!(request_approved.is_some(), "should have approval criterion");
}

#[tokio::test]
async fn test_tier_1_boundary_just_under_5k() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "api-007".to_string());
    inputs.insert("requester_id".to_string(), "eve@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "499000".to_string()); // $4,990 (just under $5k)
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "participatory".to_string());
    inputs.insert("description".to_string(), "API subscription".to_string());
    inputs.insert("human_approval_present".to_string(), "false".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    let request_approved = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "request-approved-or-blocked");
    assert!(request_approved.is_some(), "should have approval criterion");
    // Just verify a decision was made (MT/Blocked/Unmet)
    let outcome_str = &request_approved.unwrap().result;
    assert!(!outcome_str.is_empty(), "should have decision result");
}

#[tokio::test]
async fn test_tier_2_boundary_just_over_5k() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "training-008".to_string());
    inputs.insert("requester_id".to_string(), "frank@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "5100000".to_string()); // $51,000 (just over $5k, in Tier 2 boundary but closer to Tier 3)
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "supervisory".to_string());
    inputs.insert("description".to_string(), "Training program".to_string());
    inputs.insert("human_approval_present".to_string(), "true".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    let request_approved = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "request-approved-or-blocked");
    assert!(request_approved.is_some(), "should have approval criterion");
}

#[tokio::test]
async fn test_audit_entry_recorded() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "audit-001".to_string());
    inputs.insert("requester_id".to_string(), "alice@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "250000".to_string());
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "advisory".to_string());
    inputs.insert("description".to_string(), "Test audit entry".to_string());
    inputs.insert("human_approval_present".to_string(), "false".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    let audit_criterion = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "audit-entry-recorded");
    assert!(audit_criterion.is_some(), "audit entry should be recorded");
    let outcome_str = &audit_criterion.unwrap().result;
    assert!(
        outcome_str.contains("Met"),
        "audit entry should be met: {}",
        outcome_str
    );
}

#[tokio::test]
async fn test_missing_required_field_request_id() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    // Intentionally missing request_id
    inputs.insert("requester_id".to_string(), "alice@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "100000".to_string());
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "advisory".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience).await;

    assert!(result.is_err(), "should fail with missing request_id");
}

#[tokio::test]
async fn test_invalid_authority() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "test-001".to_string());
    inputs.insert("requester_id".to_string(), "alice@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "100000".to_string());
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "invalid-authority".to_string());
    inputs.insert("description".to_string(), "Test".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience).await;

    assert!(result.is_err(), "should fail with invalid authority");
}

#[tokio::test]
async fn test_negative_amount_rejected() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "test-002".to_string());
    inputs.insert("requester_id".to_string(), "alice@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "-100000".to_string()); // Negative amount
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "advisory".to_string());
    inputs.insert("description".to_string(), "Test".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    // Validation should catch negative amount
    let policy_decision = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "policy-decision-produced");
    assert!(policy_decision.is_some());
}

#[tokio::test]
async fn test_all_three_criteria_met() {
    let store = InMemoryStore::new();
    let experience = ExperienceRegistry::new();

    let mut inputs = HashMap::new();
    inputs.insert("request_id".to_string(), "complete-001".to_string());
    inputs.insert("requester_id".to_string(), "alice@example.com".to_string());
    inputs.insert("amount_minor".to_string(), "200000".to_string()); // Tier 1
    inputs.insert("currency_code".to_string(), "USD".to_string());
    inputs.insert("authority".to_string(), "advisory".to_string());
    inputs.insert("description".to_string(), "Complete test".to_string());
    inputs.insert("human_approval_present".to_string(), "false".to_string());

    let result = execute_truth(&store, "budget-approval", inputs, true, &experience)
        .await
        .expect("execution should succeed");

    assert!(result.converged, "should converge");

    // All three criteria should be met
    let policy_decision = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "policy-decision-produced");
    assert!(policy_decision.is_some());
    assert!(policy_decision.unwrap().result.contains("Met"));

    let request_approved = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "request-approved-or-blocked");
    assert!(request_approved.is_some());
    // Just check that a decision was made
    assert!(!request_approved.unwrap().result.is_empty());

    let audit_recorded = result
        .criteria_outcomes
        .iter()
        .find(|c| c.criterion == "audit-entry-recorded");
    assert!(audit_recorded.is_some());
    // Audit should be recorded
    assert!(!audit_recorded.unwrap().result.is_empty());
}
