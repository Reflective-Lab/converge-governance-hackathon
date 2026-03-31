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
        key: "evaluate-vendor",
        display_name: "Evaluate AI Vendor",
        summary: "Multi-agent vendor evaluation: compliance, risk, cost, decision",
        packs: &["compliance-pack", "risk-pack", "cost-pack"],
        criteria: &[
            ("all-vendors-screened", "All vendors have compliance screening facts"),
            ("recommendation-produced", "A decision recommendation fact exists"),
        ],
    },
    TruthDef {
        key: "audit-vendor-decision",
        display_name: "Audit Vendor Decision",
        summary: "Trust pack: audit trail, provenance, and compliance scan for a vendor decision",
        packs: &["trust-pack"],
        criteria: &[
            ("audit-entries-written", "All access decisions have audit entries"),
            ("compliance-scanned", "Compliance scan has been performed"),
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
    fn intent_builds_with_packs() {
        let truth = find_truth("evaluate-vendor").unwrap();
        let intent = build_intent(truth);
        assert!(intent.active_packs.contains(&"compliance-pack".to_string()));
    }
}
