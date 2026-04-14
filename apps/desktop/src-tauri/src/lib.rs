use std::sync::Arc;

mod config;

use converge_core::traits::{ChatMessage, ChatRequest, ChatRole, DynChatBackend, ResponseFormat};
use converge_provider::{ChatBackendSelectionConfig, select_chat_backend};
use converge_tool::StaticChatBackend;
use converge_tool::gherkin::{
    GherkinValidator, InvariantClassTag, IssueCategory, ScenarioKind, Severity, ValidationConfig,
    ValidationError,
};
use converge_tool::truths::{TruthGovernance, parse_truth_document};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct TruthGuidanceResponse {
    current_title: String,
    suggested_title: String,
    should_rewrite: bool,
    source: &'static str,
    rationale: Vec<String>,
    description_hints: Vec<String>,
    note: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TruthDraftContext {
    title: String,
    description_line_count: usize,
    scenario_count: usize,
    has_intent: bool,
    has_authority: bool,
    has_constraint: bool,
    has_evidence: bool,
    has_exception: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LlmTruthGuidance {
    #[serde(default)]
    should_rewrite: bool,
    #[serde(default)]
    suggested_title: String,
    #[serde(default)]
    rationale: Vec<String>,
    #[serde(default)]
    description_hints: Vec<String>,
}

#[tauri::command(rename_all = "snake_case")]
fn validate_gherkin(spec: String) -> Result<ValidationResponse, String> {
    if spec.trim().is_empty() {
        return Err("Spec is empty. Paste a Truth or Feature before validating.".into());
    }

    let validator = offline_validator();
    let validation = validator
        .validate(&spec, "editor.truths")
        .map_err(format_validation_error)?;

    Ok(ValidationResponse {
        is_valid: validation.is_valid,
        summary: validation.summary(),
        scenario_count: validation.scenario_count,
        confidence: validation.confidence,
        validation_mode: "offline-syntax-and-conventions",
        notes: vec![String::from(
            "Local validation checks Converge Truth parsing, governance blocks, and Gherkin conventions. Business-sense and compilability checks stay disabled until a live ChatBackend validator is configured.",
        )],
        governance: summarize_governance(&validation.governance),
        scenarios: validation
            .scenario_metas
            .into_iter()
            .map(|meta| ScenarioSummary {
                name: meta.name,
                kind: meta.kind.map(scenario_kind_label),
                invariant_class: meta.invariant_class.map(invariant_class_label),
                id: meta.id,
                provider: meta.provider,
                is_test: meta.is_test,
            })
            .collect(),
        issues: validation
            .issues
            .into_iter()
            .map(|issue| ValidationIssueView {
                location: issue.location,
                category: issue_category_label(issue.category),
                severity: severity_label(issue.severity),
                message: issue.message,
                suggestion: issue.suggestion,
            })
            .collect(),
    })
}

#[tauri::command(rename_all = "snake_case")]
async fn guide_truth_heading(spec: String) -> Result<Option<TruthGuidanceResponse>, String> {
    let Some(current_title) = extract_truth_title(&spec) else {
        return Ok(None);
    };

    Ok(Some(guided_heading(&spec, &current_title).await))
}

fn offline_validator() -> GherkinValidator {
    GherkinValidator::new(
        Arc::new(StaticChatBackend::constant("VALID")),
        ValidationConfig {
            check_business_sense: false,
            check_compilability: false,
            check_conventions: true,
            min_confidence: 0.0,
        },
    )
}

async fn guided_heading(spec: &str, current_title: &str) -> TruthGuidanceResponse {
    match request_live_heading_guidance(spec, current_title).await {
        Ok(response) => response,
        Err(error) => local_heading_guidance(
            spec,
            current_title,
            format!(
                "Live ChatBackend guidance failed, so the editor is showing a local rewrite instead: {error}"
            ),
        ),
    }
}

async fn request_live_heading_guidance(
    spec: &str,
    current_title: &str,
) -> Result<TruthGuidanceResponse, String> {
    let editor_config = config::editor_config();
    let draft_context = draft_context(spec, current_title);
    let selected = select_heading_backend(editor_config)?;
    let prompt = format!(
        r#"You are improving a Converge Truth heading inside a desktop editor.

A strong heading states a durable business truth, decision rule, or governed outcome.
A weak heading reads like a topic, workstream, or initiative label.

Current heading:
{current_title}

Parsed draft context:
{draft_context}

Spec excerpt:
{spec_excerpt}

Return ONLY a JSON object with this exact schema:
{{
  "shouldRewrite": true,
  "suggestedTitle": "Enterprise AI vendor selection is auditable, constrained, and approval-gated",
  "rationale": [
    "Current heading reads like a topic, not a governed truth."
  ],
  "descriptionHints": [
    "Vendor choice must be reproducible from explicit evidence.",
    "Final selection must stay within policy, budget, and approval boundaries."
  ]
}}

Rules:
- Do not include `Truth:` in suggestedTitle.
- Keep suggestedTitle concise.
- Prefer declarative language such as `is`, `must`, `requires`, `remains`, or `produces`.
- Align the rewrite with the governance, evidence, and approval context in the spec.
- If the current heading is already strong, set shouldRewrite to false and keep suggestedTitle equal to the current heading.
- descriptionHints should be 0-2 concise lines suitable immediately below the Truth header."#,
        draft_context = serde_json::to_string_pretty(&draft_context)
            .map_err(|error| format!("Failed to serialize draft context: {error}"))?,
        spec_excerpt = truncated_spec_excerpt(spec)
    );
    let response = selected
        .backend
        .chat(ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: prompt,
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: Some("You are a strict Converge Truth editor. Respond with JSON only.".into()),
            tools: Vec::new(),
            response_format: ResponseFormat::Json,
            max_tokens: Some(300),
            temperature: Some(0.2),
            stop_sequences: Vec::new(),
            model: editor_config
                .heading_model_override()
                .map(ToString::to_string),
        })
        .await
        .map_err(|error| format!("ChatBackend request failed: {error}"))?;
    let parsed = parse_llm_truth_guidance(&response.content)?;
    let suggested_title = sanitize_suggested_title(&parsed.suggested_title, current_title);

    Ok(TruthGuidanceResponse {
        current_title: current_title.to_string(),
        suggested_title,
        should_rewrite: parsed.should_rewrite,
        source: "live-chat-backend",
        rationale: normalize_rationale(
            parsed.rationale,
            "The current heading was evaluated against the full Truth context.".to_string(),
        ),
        description_hints: normalize_description_hints(parsed.description_hints),
        note: format!(
            "Live guidance is active through ChatBackend provider `{}`.",
            selected.provider
        ),
    })
}

struct SelectedHeadingBackend {
    backend: Arc<dyn DynChatBackend>,
    provider: String,
}

fn select_heading_backend(config: &config::EditorConfig) -> Result<SelectedHeadingBackend, String> {
    let mut selection = ChatBackendSelectionConfig::from_env()
        .map_err(|error| format!("ChatBackend selection configuration failed: {error}"))?;
    if let Some(provider) = config.heading_provider_override() {
        selection = selection.with_provider_override(provider.to_string());
    }
    let selected = select_chat_backend(&selection)
        .map_err(|error| format!("No live chat backend is available: {error}"))?;
    let provider = selected.provider().to_string();
    Ok(SelectedHeadingBackend {
        backend: selected.backend,
        provider,
    })
}

fn parse_llm_truth_guidance(content: &str) -> Result<LlmTruthGuidance, String> {
    let payload = content.trim();
    let json = match (payload.find('{'), payload.rfind('}')) {
        (Some(start), Some(end)) if start <= end => &payload[start..=end],
        _ => payload,
    };

    serde_json::from_str(json)
        .map_err(|error| format!("LLM returned invalid Truth guidance JSON: {error}"))
}

fn local_heading_guidance(spec: &str, current_title: &str, note: String) -> TruthGuidanceResponse {
    let draft_context = draft_context(spec, current_title);
    let current_title_trimmed = current_title.trim();
    let title_lower = current_title_trimmed.to_ascii_lowercase();
    let spec_lower = spec.to_ascii_lowercase();
    let has_authority = draft_context.has_authority;
    let has_constraint = draft_context.has_constraint;
    let has_evidence = draft_context.has_evidence;
    let mentions_approval = spec_lower.contains("approval");
    let mentions_traceability = spec_lower.contains("traceable")
        || spec_lower.contains("audit")
        || spec_lower.contains("provenance")
        || spec_lower.contains("compliance");
    let mentions_policy = spec_lower.contains("governance")
        || spec_lower.contains("policy")
        || spec_lower.contains("budget")
        || spec_lower.contains("cost")
        || has_constraint;
    let has_assertive_verb = truth_title_is_declarative(&title_lower);
    let sounds_like_topic = title_lower.contains(" for ")
        || title_lower.contains(" workflow")
        || title_lower.contains(" rollout")
        || title_lower.contains(" process")
        || !has_assertive_verb;
    let subject = normalize_subject(current_title_trimmed);
    let predicate = quality_predicate(
        has_authority || mentions_approval,
        has_constraint || mentions_policy,
        has_evidence || mentions_traceability,
    );
    let suggested_title = if sounds_like_topic {
        format!("{subject} {predicate}")
    } else {
        current_title_trimmed.to_string()
    };
    let mut rationale = Vec::new();

    if title_lower.contains(" for ") {
        rationale.push(
            "The current heading reads like a topic scoped to an initiative, not a governed truth."
                .to_string(),
        );
    }
    if !has_assertive_verb {
        rationale.push("A Converge Truth heading should state a claim or rule, usually with language like `is`, `must`, or `requires`.".to_string());
    }
    if !has_constraint {
        rationale.push("Vendor-selection truths are stronger when the title is backed by explicit constraints.".to_string());
    }
    if !has_evidence {
        rationale.push("Vendor-selection truths should usually imply what evidence makes the decision auditable.".to_string());
    }

    let description_hints = build_description_hints(
        &draft_context,
        spec,
        has_authority,
        has_constraint,
        has_evidence,
    );

    TruthGuidanceResponse {
        current_title: current_title_trimmed.to_string(),
        suggested_title,
        should_rewrite: sounds_like_topic,
        source: "local-heuristic",
        rationale: normalize_rationale(
            rationale,
            "The editor is checking whether the heading is written as a stable truth instead of a topic label.".to_string(),
        ),
        description_hints,
        note,
    }
}

fn build_description_hints(
    draft_context: &TruthDraftContext,
    spec: &str,
    has_authority: bool,
    has_constraint: bool,
    has_evidence: bool,
) -> Vec<String> {
    let mut hints = Vec::new();

    if draft_context.description_line_count == 0 {
        hints.push("Vendor choice must be reproducible from explicit evidence.".to_string());
    }
    if has_authority || spec.to_ascii_lowercase().contains("approval") {
        hints.push("Final selection must stay within accountable approval boundaries.".to_string());
    } else if !has_constraint {
        hints.push("Selection must stay within policy, cost, and risk boundaries.".to_string());
    }
    if !has_evidence && hints.len() < 2 {
        hints.push(
            "The recommended vendor must be justified by traceable review artifacts.".to_string(),
        );
    }

    normalize_description_hints(hints)
}

fn draft_context(spec: &str, current_title: &str) -> TruthDraftContext {
    if let Ok(document) = parse_truth_document(spec) {
        return TruthDraftContext {
            title: current_title.trim().to_string(),
            description_line_count: description_line_count(spec),
            scenario_count: document
                .gherkin
                .lines()
                .filter(|line| line.trim_start().starts_with("Scenario:"))
                .count(),
            has_intent: document.governance.intent.is_some(),
            has_authority: document.governance.authority.is_some(),
            has_constraint: document.governance.constraint.is_some(),
            has_evidence: document.governance.evidence.is_some(),
            has_exception: document.governance.exception.is_some(),
        };
    }

    TruthDraftContext {
        title: current_title.trim().to_string(),
        description_line_count: description_line_count(spec),
        scenario_count: spec
            .lines()
            .filter(|line| line.trim_start().starts_with("Scenario:"))
            .count(),
        has_intent: spec.contains("\nIntent:"),
        has_authority: spec.contains("\nAuthority:"),
        has_constraint: spec.contains("\nConstraint:"),
        has_evidence: spec.contains("\nEvidence:"),
        has_exception: spec.contains("\nException:"),
    }
}

fn normalize_description_hints(mut hints: Vec<String>) -> Vec<String> {
    hints.retain(|hint| !hint.trim().is_empty());
    hints.truncate(2);
    hints
}

fn normalize_rationale(mut rationale: Vec<String>, fallback: String) -> Vec<String> {
    rationale.retain(|item| !item.trim().is_empty());
    if rationale.is_empty() {
        rationale.push(fallback);
    }
    rationale
}

fn quality_predicate(has_authority: bool, has_constraint: bool, has_evidence: bool) -> String {
    let mut qualities = Vec::new();

    if has_evidence {
        qualities.push("auditable");
    }
    if has_constraint {
        qualities.push("constrained");
    }
    if has_authority {
        qualities.push("approval-gated");
    }
    if qualities.is_empty() {
        qualities.push("explicit");
        qualities.push("reviewable");
    }

    format!("is {}", join_phrases(&qualities))
}

fn join_phrases(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_string(),
        [first, second] => format!("{first} and {second}"),
        [first, middle @ .., last] => format!("{first}, {}, and {last}", middle.join(", ")),
    }
}

fn truth_title_is_declarative(title_lower: &str) -> bool {
    [
        " is ",
        " must ",
        " requires ",
        " remains ",
        " produces ",
        " blocks ",
        " allows ",
    ]
    .iter()
    .any(|needle| title_lower.contains(needle))
}

fn normalize_subject(title: &str) -> String {
    let trimmed = title.trim().trim_end_matches('.');

    if let Some((left, right)) = trimmed.split_once(" for ") {
        let left = left.trim();
        if reorderable_subject(left) {
            let right = strip_context_suffix(right.trim());
            return uppercase_first(&format!("{} {}", right, left.to_ascii_lowercase()));
        }
    }

    uppercase_first(trimmed)
}

fn reorderable_subject(left: &str) -> bool {
    let left = left.to_ascii_lowercase();
    [
        "selection",
        "evaluation",
        "approval",
        "review",
        "screening",
        "comparison",
    ]
    .iter()
    .any(|suffix| left.ends_with(suffix))
}

fn strip_context_suffix(value: &str) -> String {
    for suffix in [" rollout", " workflow", " process", " program"] {
        if let Some(stripped) = value.strip_suffix(suffix) {
            return stripped.trim().to_string();
        }
    }

    value.trim().to_string()
}

fn uppercase_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    format!("{}{}", first.to_uppercase(), chars.as_str())
}

fn description_line_count(spec: &str) -> usize {
    let lines: Vec<&str> = spec.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("Truth:") || trimmed.starts_with("Feature:") {
            let mut count = 0;

            for next in lines.iter().skip(idx + 1) {
                let trimmed_next = next.trim();
                if trimmed_next.is_empty() {
                    if count > 0 {
                        break;
                    }
                    continue;
                }
                if is_heading_boundary(trimmed_next) {
                    break;
                }
                count += 1;
            }

            return count;
        }
    }

    0
}

fn is_heading_boundary(line: &str) -> bool {
    matches!(
        line,
        "Intent:" | "Authority:" | "Constraint:" | "Evidence:" | "Exception:"
    ) || line.starts_with('@')
        || line.starts_with("Background:")
        || line.starts_with("Scenario:")
        || line.starts_with("Rule:")
        || line.starts_with("Example:")
        || line.starts_with("Examples:")
}

fn extract_truth_title(spec: &str) -> Option<String> {
    spec.lines().find_map(|line| {
        let trimmed = line.trim_start();
        trimmed
            .strip_prefix("Truth:")
            .or_else(|| trimmed.strip_prefix("Feature:"))
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn sanitize_suggested_title(suggested_title: &str, fallback: &str) -> String {
    let trimmed = suggested_title.trim();
    let stripped = trimmed
        .strip_prefix("Truth:")
        .or_else(|| trimmed.strip_prefix("Feature:"))
        .map(str::trim)
        .unwrap_or(trimmed);

    if stripped.is_empty() {
        fallback.to_string()
    } else {
        stripped.to_string()
    }
}

fn truncated_spec_excerpt(spec: &str) -> String {
    const MAX_LINES: usize = 20;
    const MAX_CHARS: usize = 2200;

    let mut excerpt = spec.lines().take(MAX_LINES).collect::<Vec<_>>().join("\n");
    if excerpt.chars().count() > MAX_CHARS {
        excerpt = excerpt.chars().take(MAX_CHARS).collect::<String>();
        excerpt.push_str("\n...");
    }
    excerpt
}

fn summarize_governance(governance: &TruthGovernance) -> GovernanceSummary {
    GovernanceSummary {
        intent: governance.intent.is_some(),
        authority: governance.authority.is_some(),
        constraint: governance.constraint.is_some(),
        evidence: governance.evidence.is_some(),
        exception: governance.exception.is_some(),
    }
}

fn format_validation_error(error: ValidationError) -> String {
    match error {
        ValidationError::ParseError(message) => format!("Parse error: {message}"),
        ValidationError::IoError(message) => format!("IO error: {message}"),
        ValidationError::LlmError(message) => format!("LLM error: {message}"),
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

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            validate_gherkin,
            guide_truth_heading
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_gherkin_accepts_truth_blocks() {
        let result = validate_gherkin(
            r#"Truth: Vendor selection

Intent:
  Outcome: Pick a preferred vendor.

Scenario: Vendor evaluation is traceable
  Given candidate vendors "Acme AI, Beta ML"
  When the governance workflow evaluates each vendor
  Then each vendor should produce a compliance screening result
"#
            .into(),
        )
        .unwrap();

        assert!(result.is_valid);
        assert!(result.governance.intent);
        assert_eq!(result.scenario_count, 1);
    }

    #[test]
    fn validate_gherkin_reports_missing_then_step() {
        let result = validate_gherkin(
            r#"Feature: Broken spec

Scenario: Missing expectation
  Given a vendor list
  When the system validates the spec
"#
            .into(),
        )
        .unwrap();

        assert!(!result.is_valid);
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.message.contains("lacks Then steps"))
        );
    }

    #[test]
    fn local_guidance_rewrites_topic_titles() {
        let guidance = local_heading_guidance(
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
            "local".to_string(),
        );

        assert!(guidance.should_rewrite);
        assert_eq!(
            guidance.suggested_title,
            "Enterprise AI vendor selection is auditable, constrained, and approval-gated"
        );
    }

    #[test]
    fn local_guidance_keeps_declarative_titles() {
        let guidance = local_heading_guidance(
            r#"Truth: Enterprise AI vendor selection is auditable and approval-gated

Constraint:
  Cost Limit: first-year spend stays within procurement budget

Evidence:
  Requires: security_assessment
"#,
            "Enterprise AI vendor selection is auditable and approval-gated",
            "local".to_string(),
        );

        assert!(!guidance.should_rewrite);
        assert_eq!(
            guidance.suggested_title,
            "Enterprise AI vendor selection is auditable and approval-gated"
        );
    }
}
