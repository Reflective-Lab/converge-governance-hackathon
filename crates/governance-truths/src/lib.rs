use converge_core::{
    Context, ContextKey, Criterion, CriterionEvaluator, CriterionResult, TypesBudgets,
    TypesIntentId, TypesIntentKind, TypesObjective, TypesRootIntent,
};

// ---------------------------------------------------------------------------
// Truth definitions
// ---------------------------------------------------------------------------

pub struct TruthDef {
    pub key: &'static str,
    pub display_name: &'static str,
    pub summary: &'static str,
    pub packs: &'static [&'static str],
    pub criteria: &'static [(&'static str, &'static str)], // (id, description)
}

pub const TRUTHS: &[TruthDef] = &[
    TruthDef {
        key: "dynamic-due-diligence",
        display_name: "Dynamic Due Diligence",
        summary: "Organism-planned, dynamic due-diligence loop: breadth research, depth research, contradictions, synthesis",
        packs: &[
            "planning-pack",
            "research-pack",
            "analysis-pack",
            "synthesis-pack",
        ],
        criteria: &[
            (
                "critical-evidence-collected",
                "Critical diligence categories have supporting evidence",
            ),
            ("final-brief-produced", "A final due-diligence brief exists"),
        ],
    },
    TruthDef {
        key: "evaluate-vendor",
        display_name: "Evaluate AI Vendor",
        summary: "Organism-planned, multi-agent vendor evaluation: compliance, risk, cost, decision",
        packs: &["planning-pack", "compliance-pack", "risk-pack", "cost-pack"],
        criteria: &[
            (
                "all-vendors-screened",
                "All vendors have compliance screening facts",
            ),
            (
                "recommendation-produced",
                "A decision recommendation fact exists",
            ),
        ],
    },
    TruthDef {
        key: "audit-vendor-decision",
        display_name: "Audit Vendor Decision",
        summary: "Trust pack: audit trail, provenance, and compliance scan for a vendor decision",
        packs: &["trust-pack"],
        criteria: &[
            (
                "audit-entries-written",
                "All access decisions have audit entries",
            ),
            ("compliance-scanned", "Compliance scan has been performed"),
        ],
    },
    TruthDef {
        key: "authorize-vendor-commitment",
        display_name: "Authorize Vendor Commitment",
        summary: "Policy decision for committing a vendor recommendation into a real procurement flow",
        packs: &["policy-pack"],
        criteria: &[
            (
                "policy-decision-produced",
                "A policy decision fact exists for the commitment",
            ),
            (
                "commitment-authorized",
                "The commitment is either authorized or blocked honestly for human review",
            ),
        ],
    },
    // Add your truths here.
];

pub fn find_truth(key: &str) -> Option<&'static TruthDef> {
    TRUTHS.iter().find(|t| t.key == key)
}

// ---------------------------------------------------------------------------
// Converge bindings
// ---------------------------------------------------------------------------

pub fn build_intent(truth: &TruthDef) -> TypesRootIntent {
    TypesRootIntent::builder()
        .id(TypesIntentId::new(format!("truth:{}", truth.key)))
        .kind(TypesIntentKind::Custom)
        .request(truth.summary.to_string())
        .objective(Some(TypesObjective::Custom(truth.display_name.to_string())))
        .active_packs(truth.packs.iter().map(|p| p.to_string()).collect())
        .success_criteria(
            truth
                .criteria
                .iter()
                .map(|(id, desc)| Criterion::required(*id, *desc))
                .collect(),
        )
        .budgets(TypesBudgets::with_cycles(10))
        .build()
}

// ---------------------------------------------------------------------------
// Evaluators
// ---------------------------------------------------------------------------

pub struct EvaluateVendorEvaluator;

pub struct DynamicDueDiligenceEvaluator;

impl CriterionEvaluator for DynamicDueDiligenceEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &Context) -> CriterionResult {
        match criterion.id.as_str() {
            "critical-evidence-collected" => {
                let categories = context
                    .get(ContextKey::Hypotheses)
                    .iter()
                    .filter_map(|fact| {
                        serde_json::from_str::<serde_json::Value>(&fact.content).ok()
                    })
                    .filter_map(|payload| {
                        payload
                            .get("category")
                            .and_then(|value| value.as_str())
                            .map(ToString::to_string)
                    })
                    .collect::<std::collections::HashSet<_>>();

                if [
                    "product",
                    "market",
                    "competition",
                    "technology",
                    "ownership",
                    "financials",
                    "compliance",
                ]
                .into_iter()
                .all(|category| categories.contains(category))
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "critical diligence categories are still missing".into(),
                    }
                }
            }
            "final-brief-produced" => {
                if context
                    .get(ContextKey::Proposals)
                    .iter()
                    .any(|fact| fact.id == "dd:final-brief")
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no due-diligence brief has been synthesized".into(),
                    }
                }
            }
            _ => CriterionResult::Indeterminate,
        }
    }
}

impl CriterionEvaluator for EvaluateVendorEvaluator {
    fn evaluate(&self, criterion: &Criterion, context: &Context) -> CriterionResult {
        match criterion.id.as_str() {
            "all-vendors-screened" => {
                if context
                    .get(ContextKey::Seeds)
                    .iter()
                    .any(|f| f.id.starts_with("compliance:screen:"))
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no vendors screened yet".into(),
                    }
                }
            }
            "recommendation-produced" => {
                if context
                    .get(ContextKey::Evaluations)
                    .iter()
                    .any(|f| f.id == "decision:recommendation")
                {
                    CriterionResult::Met { evidence: vec![] }
                } else {
                    CriterionResult::Unmet {
                        reason: "no recommendation produced".into(),
                    }
                }
            }
            _ => CriterionResult::Indeterminate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truth_catalog_has_evaluate_vendor() {
        assert!(find_truth("evaluate-vendor").is_some());
    }

    #[test]
    fn truth_catalog_has_dynamic_due_diligence() {
        assert!(find_truth("dynamic-due-diligence").is_some());
    }

    #[test]
    fn intent_builds_with_packs() {
        let truth = find_truth("evaluate-vendor").unwrap();
        let intent = build_intent(truth);
        assert!(intent.active_packs.contains(&"compliance-pack".to_string()));
    }
}
