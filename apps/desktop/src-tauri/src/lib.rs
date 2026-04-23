use std::sync::Arc;

mod config;

use axiom_truth::StaticChatBackend;
use axiom_truth::gherkin::{
    GherkinValidator, InvariantClassTag, IssueCategory, ScenarioKind, Severity, ValidationConfig,
    ValidationError,
};
use axiom_truth::guidance::{self, GuidanceConfig};
use axiom_truth::policy_lens;
use axiom_truth::simulation::{self, FindingSeverity, SimulationConfig};
use axiom_truth::truths::{TruthGovernance, parse_truth_document};
use axiom_truth::validation_view;
use serde::Serialize;

const OFFLINE_VALIDATION_MODE: &str = "offline-syntax-and-conventions";

// ─── Validation response (Helm-specific serialization) ───

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationResponse {
    is_valid: bool,
    summary: String,
    scenario_count: usize,
    confidence: f64,
    validation_mode: &'static str,
    notes: Vec<String>,
    governance: GovernanceSummary,
    scenarios: Vec<ScenarioSummary>,
    steps: Vec<validation_view::ValidationStep>,
    issues: Vec<ValidationIssueView>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GovernanceSummary {
    intent: bool,
    authority: bool,
    constraint: bool,
    evidence: bool,
    exception: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScenarioSummary {
    name: String,
    kind: Option<&'static str>,
    invariant_class: Option<&'static str>,
    id: Option<String>,
    provider: Option<String>,
    is_test: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationIssueView {
    location: String,
    category: &'static str,
    severity: &'static str,
    message: String,
    suggestion: Option<String>,
}

// ─── Simulation response ───

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SimulationResponse {
    verdict: &'static str,
    can_converge: bool,
    scenario_count: usize,
    findings: Vec<SimulationFindingView>,
    governance: GovernanceCoverageView,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SimulationFindingView {
    severity: &'static str,
    category: &'static str,
    message: String,
    suggestion: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GovernanceCoverageView {
    has_intent: bool,
    has_outcome: bool,
    has_authority: bool,
    has_actor: bool,
    has_approval_gate: bool,
    has_constraint: bool,
    has_evidence: bool,
    evidence_count: usize,
    has_exception: bool,
    has_escalation_path: bool,
}

// ─── Policy response ───

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PolicyResponse {
    required_gates: Vec<String>,
    gated_actions: Vec<GatedActionView>,
    requires_human_approval: bool,
    authority_level: Option<String>,
    spending_limits: Vec<String>,
    escalation_targets: Vec<String>,
    cedar_preview: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GatedActionView {
    action: String,
    reason: String,
}

// ═══════════════════════════════════════════════
// Tauri Commands — thin wrappers over the Axiom truth validation boundary.
// ═══════════════════════════════════════════════

#[tauri::command(rename_all = "snake_case")]
async fn validate_gherkin(spec: String) -> Result<ValidationResponse, String> {
    if spec.trim().is_empty() {
        return Err("Spec is empty. Paste a Truth or Feature before validating.".into());
    }

    let config = offline_validation_config();
    let validator = offline_validator(config.clone());
    match validator.validate(&spec, "editor.truths").await {
        Ok(validation) => Ok(build_validation_response(validation, &config)),
        Err(ValidationError::ParseError(message)) => {
            Ok(build_parse_error_response(message, &config))
        }
        Err(error) => Err(format_validation_error(error)),
    }
}

#[tauri::command(rename_all = "snake_case")]
async fn guide_truth_heading(spec: String) -> Result<Option<guidance::GuidanceResponse>, String> {
    let editor_config = config::editor_config();
    let gc = GuidanceConfig {
        provider_override: editor_config.heading_provider_override().map(Into::into),
        model_override: editor_config.heading_model_override().map(Into::into),
    };
    Ok(guidance::guide_heading(&spec, &gc).await)
}

#[tauri::command(rename_all = "snake_case")]
fn simulate_truth(spec: String) -> Result<SimulationResponse, String> {
    if spec.trim().is_empty() {
        return Err("Spec is empty.".into());
    }

    let report = simulation::simulate_spec(&spec, &SimulationConfig::default())
        .map_err(|e| format!("{e}"))?;

    Ok(SimulationResponse {
        verdict: match report.verdict {
            simulation::Verdict::Ready => "ready",
            simulation::Verdict::Risky => "risky",
            simulation::Verdict::WillNotConverge => "will-not-converge",
        },
        can_converge: report.can_converge(),
        scenario_count: report.scenario_count,
        findings: report
            .findings
            .iter()
            .map(|f| SimulationFindingView {
                severity: match f.severity {
                    FindingSeverity::Info => "info",
                    FindingSeverity::Warning => "warning",
                    FindingSeverity::Error => "error",
                },
                category: f.category,
                message: f.message.clone(),
                suggestion: f.suggestion.clone(),
            })
            .collect(),
        governance: GovernanceCoverageView {
            has_intent: report.governance_coverage.has_intent,
            has_outcome: report.governance_coverage.has_outcome,
            has_authority: report.governance_coverage.has_authority,
            has_actor: report.governance_coverage.has_actor,
            has_approval_gate: report.governance_coverage.has_approval_gate,
            has_constraint: report.governance_coverage.has_constraint,
            has_evidence: report.governance_coverage.has_evidence,
            evidence_count: report.governance_coverage.evidence_count,
            has_exception: report.governance_coverage.has_exception,
            has_escalation_path: report.governance_coverage.has_escalation_path,
        },
    })
}

#[tauri::command(rename_all = "snake_case")]
fn extract_policy(spec: String) -> Result<PolicyResponse, String> {
    if spec.trim().is_empty() {
        return Err("Spec is empty.".into());
    }

    let doc = parse_truth_document(&spec).map_err(|e| format!("{e}"))?;
    let reqs = policy_lens::extract_requirements(&doc.governance);
    let cedar_preview = generate_cedar_preview(&doc.governance);

    Ok(PolicyResponse {
        required_gates: reqs.required_gates,
        gated_actions: reqs
            .gated_actions
            .iter()
            .map(|a| GatedActionView {
                action: a.action.clone(),
                reason: a.reason.clone(),
            })
            .collect(),
        requires_human_approval: reqs.requires_human_approval,
        authority_level: reqs.authority_level,
        spending_limits: reqs.spending_limits,
        escalation_targets: reqs.escalation_targets,
        cedar_preview,
    })
}

// ═══════════════════════════════════════════════
// Internal helpers
// ═══════════════════════════════════════════════

fn offline_validation_config() -> ValidationConfig {
    ValidationConfig {
        check_business_sense: false,
        check_compilability: false,
        check_conventions: true,
        min_confidence: 0.0,
    }
}

fn offline_validator(config: ValidationConfig) -> GherkinValidator {
    GherkinValidator::new(Arc::new(StaticChatBackend::constant("VALID")), config)
}

fn build_validation_response(
    validation: axiom_truth::gherkin::SpecValidation,
    config: &ValidationConfig,
) -> ValidationResponse {
    let issues: Vec<ValidationIssueView> = validation
        .issues
        .iter()
        .map(|issue| ValidationIssueView {
            location: issue.location.clone(),
            category: issue_category_label(issue.category),
            severity: severity_label(issue.severity),
            message: issue.message.clone(),
            suggestion: issue.suggestion.clone(),
        })
        .collect();

    let steps = validation_view::build_steps(&validation, config);
    let summary = validation_view::summarize(&validation);
    let governance = governance_summary(&validation.governance);

    ValidationResponse {
        is_valid: validation.is_valid,
        summary,
        scenario_count: validation.scenario_count,
        confidence: validation.confidence,
        validation_mode: OFFLINE_VALIDATION_MODE,
        notes: vec![validation_view::offline_note()],
        governance,
        scenarios: validation
            .scenario_metas
            .iter()
            .map(|meta| ScenarioSummary {
                name: meta.name.clone(),
                kind: meta.kind.map(scenario_kind_label),
                invariant_class: meta.invariant_class.map(invariant_class_label),
                id: meta.id.clone(),
                provider: meta.provider.clone(),
                is_test: meta.is_test,
            })
            .collect(),
        steps,
        issues,
    }
}

fn build_parse_error_response(message: String, config: &ValidationConfig) -> ValidationResponse {
    ValidationResponse {
        is_valid: false,
        summary: "Syntax failed before local rule checks could continue.".into(),
        scenario_count: 0,
        confidence: 0.0,
        validation_mode: OFFLINE_VALIDATION_MODE,
        notes: vec![validation_view::offline_note()],
        governance: GovernanceSummary {
            intent: false,
            authority: false,
            constraint: false,
            evidence: false,
            exception: false,
        },
        scenarios: Vec::new(),
        steps: validation_view::build_parse_error_steps(&message, config),
        issues: vec![ValidationIssueView {
            location: "Spec".into(),
            category: "syntax",
            severity: "error",
            message,
            suggestion: None,
        }],
    }
}

fn governance_summary(governance: &TruthGovernance) -> GovernanceSummary {
    GovernanceSummary {
        intent: governance.intent.is_some(),
        authority: governance.authority.is_some(),
        constraint: governance.constraint.is_some(),
        evidence: governance.evidence.is_some(),
        exception: governance.exception.is_some(),
    }
}

fn generate_cedar_preview(gov: &TruthGovernance) -> String {
    let mut lines = Vec::new();
    lines.push("// Generated Cedar policy from Truth governance blocks".into());
    lines.push(String::new());

    lines.push("// Any authorized agent may propose.".into());
    lines.push(r#"permit(principal, action == Action::"propose", resource)"#.into());
    lines.push("when {".into());
    if let Some(authority) = &gov.authority {
        if let Some(actor) = &authority.actor {
            lines.push(format!(r#"  principal.domains.contains("{actor}")"#));
        } else {
            lines.push("  true".into());
        }
    } else {
        lines.push("  true".into());
    }
    lines.push("};".into());
    lines.push(String::new());

    if let Some(evidence) = &gov.evidence {
        if !evidence.requires.is_empty() {
            lines.push("// Validation requires evidence gates to be passed.".into());
            lines.push(r#"permit(principal, action == Action::"validate", resource)"#.into());
            lines.push("when {".into());
            for (i, req) in evidence.requires.iter().enumerate() {
                let sep = if i < evidence.requires.len() - 1 {
                    " &&"
                } else {
                    ""
                };
                lines.push(format!(r#"  resource.gates_passed.contains("{req}"){sep}"#));
            }
            lines.push("};".into());
            lines.push(String::new());
        }
    }

    if let Some(authority) = &gov.authority {
        if !authority.requires_approval.is_empty() {
            lines.push("// Commit requires human approval.".into());
            lines.push(r#"permit(principal, action == Action::"commit", resource)"#.into());
            lines.push("when {".into());
            lines.push("  context.human_approval_present == true &&".into());
            lines.push("  context.required_gates_met == true".into());
            lines.push("};".into());
            lines.push(String::new());
            lines.push("// Block commit without human approval.".into());
            lines.push(r#"forbid(principal, action == Action::"commit", resource)"#.into());
            lines.push("when {".into());
            lines.push("  context.human_approval_present == false".into());
            lines.push("};".into());
            lines.push(String::new());
        }
    }

    if let Some(constraint) = &gov.constraint {
        if !constraint.cost_limit.is_empty() {
            lines.push("// Enforce spending limits.".into());
            lines.push(r#"forbid(principal, action == Action::"commit", resource)"#.into());
            lines.push("when {".into());
            lines.push("  context.amount > 0 &&".into());
            lines.push("  context.human_approval_present == false".into());
            lines.push("};".into());
            lines.push(String::new());
        }
    }

    if let Some(exception) = &gov.exception {
        if !exception.escalates_to.is_empty() {
            lines.push(format!(
                "// Escalation path: {}",
                exception.escalates_to.join(", ")
            ));
            lines.push("// When commit is denied and principal has escalatable authority,".into());
            lines.push("// the decision escalates rather than rejecting outright.".into());
        }
    }

    lines.join("\n")
}

fn format_validation_error(error: ValidationError) -> String {
    match error {
        ValidationError::ParseError(msg) => format!("Parse error: {msg}"),
        ValidationError::IoError(msg) => format!("IO error: {msg}"),
        ValidationError::LlmError(msg) => format!("LLM error: {msg}"),
    }
}

fn issue_category_label(category: IssueCategory) -> &'static str {
    match category {
        IssueCategory::BusinessSense => "business-sense",
        IssueCategory::Compilability => "compilability",
        IssueCategory::Convention => "convention",
        IssueCategory::Syntax => "syntax",
        IssueCategory::NotRelatedError => "not-related-error",
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    }
}

fn scenario_kind_label(kind: ScenarioKind) -> &'static str {
    match kind {
        ScenarioKind::Invariant => "invariant",
        ScenarioKind::Validation => "validation",
        ScenarioKind::Suggestor => "agent",
        ScenarioKind::EndToEnd => "end-to-end",
    }
}

fn invariant_class_label(class: InvariantClassTag) -> &'static str {
    match class {
        InvariantClassTag::Structural => "structural",
        InvariantClassTag::Semantic => "semantic",
        InvariantClassTag::Acceptance => "acceptance",
    }
}

// ─── Due Diligence (self-contained, copied from Monterro) ───

mod dd;

#[tauri::command]
async fn run_due_diligence(
    company_name: String,
    product_name: Option<String>,
    #[allow(unused)] focus_areas: Vec<String>,
) -> Result<dd::DdReport, String> {
    dd::run_dd(&company_name, product_name.as_deref())
        .await
        .map_err(|e| format!("{e:#}"))
}

pub fn run() {
    let _ = dotenv::dotenv();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            validate_gherkin,
            guide_truth_heading,
            simulate_truth,
            extract_policy,
            run_due_diligence
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate(spec: &str) -> ValidationResponse {
        tauri::async_runtime::block_on(validate_gherkin(spec.into())).unwrap()
    }

    #[test]
    fn validate_gherkin_accepts_truth_blocks() {
        let result = validate(
            r#"Truth: Vendor selection

Intent:
  Outcome: Pick a preferred vendor.

Scenario: Vendor evaluation is traceable
  Given candidate vendors "Acme AI, Beta ML"
  When the governance workflow evaluates each vendor
  Then each vendor should produce a compliance screening result
"#,
        );

        assert!(result.is_valid);
        assert!(result.governance.intent);
        assert_eq!(result.scenario_count, 1);
        assert_eq!(result.steps[0].status, "ok");
    }

    #[test]
    fn validate_gherkin_reports_missing_then_step() {
        let result = validate(
            r#"Feature: Broken spec

Scenario: Missing expectation
  Given a vendor list
  When the system validates the spec
"#,
        );

        assert!(!result.is_valid);
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.message.contains("lacks Then steps"))
        );
        assert_eq!(result.steps[1].status, "issue");
    }

    #[test]
    fn validate_gherkin_returns_syntax_step_for_parse_errors() {
        let result = validate(
            r#"Truth: Broken declarations

Intent:
  Outcome Pick a preferred vendor.

Scenario: Parse fails before local checks
  Given a vendor list
  When the validator reads the declarations
  Then the syntax step should fail
"#,
        );

        assert!(!result.is_valid);
        assert_eq!(result.steps[0].id, "syntax");
        assert_eq!(result.steps[0].status, "issue");
        assert_eq!(result.steps[1].status, "unavailable");
    }

    #[test]
    fn local_guidance_rewrites_topic_titles() {
        let g = guidance::local_heading_guidance(
            r#"Truth: Vendor selection for enterprise AI rollout

Authority:
  Actor: governance_review_board
  Requires Approval: final_vendor_selection

Scenario: Candidate vendors produce traceable evaluation outcomes
  Given candidate vendors "Acme AI, Beta ML, Gamma LLM"
  When the governance workflow evaluates each vendor
  Then each vendor should produce a compliance screening result
"#,
            "Vendor selection for enterprise AI rollout",
            "local".into(),
        );

        assert!(g.should_rewrite);
        assert_eq!(
            g.suggested_title,
            "Enterprise AI vendor selection is auditable, constrained, and approval-gated"
        );
    }

    #[test]
    fn local_guidance_keeps_declarative_titles() {
        let g = guidance::local_heading_guidance(
            r#"Truth: Enterprise AI vendor selection is auditable and approval-gated

Constraint:
  Cost Limit: first-year spend stays within procurement budget

Evidence:
  Requires: security_assessment
"#,
            "Enterprise AI vendor selection is auditable and approval-gated",
            "local".into(),
        );

        assert!(!g.should_rewrite);
    }
}
