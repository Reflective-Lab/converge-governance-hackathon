//! Live LLM-backed suggestors for vendor selection.
//!
//! Tiered model strategy via OpenRouter:
//! - Fast (llama-3.1-8b): compliance screening
//! - Mid (gemini-2.0-flash): cost analysis, risk scoring
//! - Strong (claude-sonnet-4): decision synthesis

use super::vendor_selection::{VendorInput, slug};
use crate::llm_helpers::{SelectedLlm, call_llm_json};
use crate::search_helpers::{format_hits_for_prompt, search_brave, search_tavily};
use async_trait::async_trait;
use converge_pack::{AgentEffect, Context as ContextView, ContextKey, ProposedFact, Suggestor};
use governance_telemetry::{InMemoryLlmCallCollector, LlmCallSink};

fn record_fallback(collector: &InMemoryLlmCallCollector, context: &str, error: &str) {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("fallback".to_string(), "true".to_string());
    metadata.insert("error".to_string(), error.to_string());

    collector.record_llm_call(governance_telemetry::LlmCallTelemetry {
        context: context.to_string(),
        provider: "none".to_string(),
        model: "deterministic-fallback".to_string(),
        elapsed_ms: 0,
        finish_reason: Some("fallback".to_string()),
        usage: None,
        metadata,
    });
}

pub const MODEL_FAST: &str = "meta-llama/llama-3.1-8b-instruct";
pub const MODEL_MID: &str = "google/gemini-2.0-flash-001";
pub const MODEL_STRONG: &str = "anthropic/claude-sonnet-4";
pub const MODEL_FAST_FALLBACK: &str = "liquid/lfm-2.5-1.2b-instruct:free";
pub const MODEL_MID_FALLBACK: &str = "google/gemma-3-27b-it:free";
pub const MODEL_STRONG_FALLBACK: &str = "openai/gpt-oss-120b:free";

// ---------------------------------------------------------------------------
// Compliance screener — fast model + web search
// ---------------------------------------------------------------------------

pub(crate) struct LiveComplianceScreenerAgent {
    pub(crate) vendors: Vec<VendorInput>,
    pub(crate) llm: SelectedLlm,
    pub(crate) collector: InMemoryLlmCallCollector,
}

#[async_trait]
impl Suggestor for LiveComplianceScreenerAgent {
    fn name(&self) -> &str {
        "live-compliance-screener"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|f| f.id().as_str() == "strategy:vendor-sel:compliance")
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = Vec::new();

        for vendor in &self.vendors {
            let fact_id = format!("compliance:screen:{}", slug(&vendor.name));
            if ctx
                .get(ContextKey::Seeds)
                .iter()
                .any(|f| f.id().as_str() == fact_id)
            {
                continue;
            }

            let search_evidence = gather_compliance_evidence(&vendor.name).await;

            let prompt = format!(
                r#"Assess the regulatory compliance posture of vendor "{name}".

Declared certifications: {certs}
Declared compliance status: {status}

{evidence_section}

Respond with JSON only:
{{
  "status": "compliant" or "non-compliant" or "pending",
  "risks": ["risk1", "risk2"],
  "evidence_summary": "brief summary of findings",
  "confidence": 0.0 to 1.0
}}"#,
                name = vendor.name,
                certs = vendor.certifications.join(", "),
                status = vendor.compliance_status,
                evidence_section = if search_evidence.is_empty() {
                    "No web evidence available. Assess based on declared data only.".to_string()
                } else {
                    format!("Web research evidence:\n{search_evidence}")
                },
            );

            let result: serde_json::Value = match call_llm_json(
                &self.llm,
                "You are a regulatory compliance analyst. Respond with JSON only.",
                &prompt,
                800,
                &format!("compliance:screen:{}", slug(&vendor.name)),
                &self.collector,
            )
            .await
            {
                Ok(v) => v,
                Err(e) => {
                    let ctx_label = format!("compliance:screen:{}", slug(&vendor.name));
                    tracing::warn!(vendor = %vendor.name, error = %e, "LLM compliance check failed, using deterministic fallback");
                    record_fallback(&self.collector, &ctx_label, &e.to_string());
                    serde_json::json!({
                        "status": vendor.compliance_status,
                        "risks": [],
                        "evidence_summary": format!("deterministic fallback: {e}"),
                        "confidence": 0.7
                    })
                }
            };

            let mut normalized = result;
            if let Some(object) = normalized.as_object_mut() {
                object.insert(
                    "vendor_name".to_string(),
                    serde_json::Value::String(vendor.name.clone()),
                );
                object.insert(
                    "declared_compliance_status".to_string(),
                    serde_json::Value::String(vendor.compliance_status.clone()),
                );
                object
                    .entry("compliance_status")
                    .or_insert_with(|| serde_json::Value::String(vendor.compliance_status.clone()));
                object.insert(
                    "certifications".to_string(),
                    serde_json::Value::Array(
                        vendor
                            .certifications
                            .iter()
                            .cloned()
                            .map(serde_json::Value::String)
                            .collect(),
                    ),
                );
            } else {
                normalized = serde_json::json!({
                    "vendor_name": vendor.name.clone(),
                    "status": vendor.compliance_status.clone(),
                    "compliance_status": vendor.compliance_status.clone(),
                    "declared_compliance_status": vendor.compliance_status.clone(),
                    "certifications": vendor.certifications.clone(),
                    "risks": [],
                    "evidence_summary": "provider returned an unexpected compliance payload",
                    "confidence": 0.5,
                });
            }

            proposals.push(ProposedFact::new(
                ContextKey::Seeds,
                format!("compliance:screen:{}", slug(&vendor.name)),
                serde_json::to_string(&normalized).unwrap_or_default(),
                "live-compliance-screener",
            ));
        }

        AgentEffect::with_proposals(proposals)
    }
}

async fn gather_compliance_evidence(vendor_name: &str) -> String {
    let brave_query = format!("{vendor_name} compliance GDPR SOC2 ISO27001");
    let tavily_query = format!("{vendor_name} regulatory compliance data residency");

    let (brave_result, tavily_result) =
        tokio::join!(search_brave(&brave_query), search_tavily(&tavily_query),);

    let mut hits = Vec::new();
    if let Ok(brave_hits) = brave_result {
        hits.extend(brave_hits.into_iter().take(4));
    }
    if let Ok(tavily_hits) = tavily_result {
        hits.extend(tavily_hits.into_iter().take(4));
    }

    if hits.is_empty() {
        String::new()
    } else {
        format_hits_for_prompt(&hits)
    }
}

// ---------------------------------------------------------------------------
// Cost analysis — mid-tier model
// ---------------------------------------------------------------------------

pub(crate) struct LiveCostAnalysisAgent {
    pub(crate) vendors: Vec<VendorInput>,
    pub(crate) llm: SelectedLlm,
    pub(crate) collector: InMemoryLlmCallCollector,
}

#[async_trait]
impl Suggestor for LiveCostAnalysisAgent {
    fn name(&self) -> &str {
        "live-cost-analysis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Seeds, ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|f| f.id().as_str() == "strategy:vendor-sel:cost")
            && ctx
                .get(ContextKey::Seeds)
                .iter()
                .any(|f| f.id().starts_with("compliance:screen:"))
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id().starts_with("cost:estimate:"))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        // Skip if we already have cost estimates
        if ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().starts_with("cost:estimate:"))
        {
            return AgentEffect::with_proposals(vec![]);
        }

        let vendor_summary: Vec<String> = self
            .vendors
            .iter()
            .map(|v| {
                format!(
                    "- {} : {} {}/month, score={}, certs={}",
                    v.name,
                    v.monthly_cost_minor as f64 / 100.0,
                    v.currency_code,
                    v.score,
                    v.certifications.join("/")
                )
            })
            .collect();

        let prompt = format!(
            r#"Analyze the cost efficiency of the following AI vendors:

{vendors}

For each vendor, estimate total cost of ownership and assess value for money.
Respond with JSON only:
{{
  "estimates": [
    {{
      "vendor": "name",
      "monthly_cost_usd": 12345,
      "tco_rating": "excellent" | "good" | "average" | "poor",
      "value_score": 0.0 to 1.0,
      "notes": "brief assessment"
    }}
  ]
}}"#,
            vendors = vendor_summary.join("\n")
        );

        let result: serde_json::Value = match call_llm_json(
            &self.llm,
            "You are a procurement cost analyst. Respond with JSON only.",
            &prompt,
            1200,
            "cost:analysis",
            &self.collector,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "LLM cost analysis failed, using deterministic fallback");
                record_fallback(&self.collector, "cost:analysis", &e.to_string());
                return deterministic_cost_proposals(&self.vendors);
            }
        };

        let mut proposals = Vec::new();
        if let Some(estimates) = result.get("estimates").and_then(|e| e.as_array()) {
            for est in estimates {
                let vendor_name = est
                    .get("vendor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                proposals.push(ProposedFact::new(
                    ContextKey::Evaluations,
                    format!("cost:estimate:{}", slug(vendor_name)),
                    serde_json::to_string(est).unwrap_or_default(),
                    "live-cost-analysis",
                ));
            }
        }

        if proposals.is_empty() {
            return deterministic_cost_proposals(&self.vendors);
        }
        AgentEffect::with_proposals(proposals)
    }
}

fn deterministic_cost_proposals(vendors: &[VendorInput]) -> AgentEffect {
    let proposals: Vec<ProposedFact> = vendors
        .iter()
        .map(|v| {
            let amount_major = (v.monthly_cost_minor.max(0) + 99) / 100;
            ProposedFact::new(
                ContextKey::Evaluations,
                format!("cost:estimate:{}", slug(&v.name)),
                serde_json::json!({
                    "vendor": v.name,
                    "monthly_cost_usd": amount_major,
                    "tco_rating": if amount_major < 400 { "good" } else { "average" },
                    "value_score": 0.7,
                    "notes": "deterministic fallback"
                })
                .to_string(),
                "live-cost-analysis",
            )
        })
        .collect();
    AgentEffect::with_proposals(proposals)
}

// ---------------------------------------------------------------------------
// Vendor risk — mid-tier model
// ---------------------------------------------------------------------------

pub(crate) struct LiveVendorRiskAgent {
    pub(crate) vendors: Vec<VendorInput>,
    pub(crate) llm: SelectedLlm,
    pub(crate) collector: InMemoryLlmCallCollector,
}

#[async_trait]
impl Suggestor for LiveVendorRiskAgent {
    fn name(&self) -> &str {
        "live-vendor-risk"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[
            ContextKey::Seeds,
            ContextKey::Evaluations,
            ContextKey::Strategies,
        ]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|f| f.id().as_str() == "strategy:vendor-sel:risk")
            && ctx
                .get(ContextKey::Seeds)
                .iter()
                .any(|f| f.id().starts_with("compliance:screen:"))
            && ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id().starts_with("cost:estimate:"))
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id().starts_with("risk:score:"))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        if ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().starts_with("risk:score:"))
        {
            return AgentEffect::with_proposals(vec![]);
        }

        let vendor_summary: Vec<String> = self
            .vendors
            .iter()
            .map(|v| {
                format!(
                    "- {} : risk_score={}, compliance={}, certs={}",
                    v.name,
                    v.risk_score,
                    v.compliance_status,
                    v.certifications.join("/")
                )
            })
            .collect();

        let prompt = format!(
            r#"Assess multi-dimensional risk for the following AI vendors:

{vendors}

For each vendor, evaluate lock-in risk, operational risk, and compliance risk.
Respond with JSON only:
{{
  "assessments": [
    {{
      "vendor": "name",
      "overall_risk": "high" | "medium" | "low",
      "lock_in_risk": 0.0 to 1.0,
      "operational_risk": 0.0 to 1.0,
      "compliance_risk": 0.0 to 1.0,
      "rationale": "brief explanation"
    }}
  ]
}}"#,
            vendors = vendor_summary.join("\n")
        );

        let result: serde_json::Value = match call_llm_json(
            &self.llm,
            "You are a vendor risk analyst. Respond with JSON only.",
            &prompt,
            1200,
            "risk:assessment",
            &self.collector,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "LLM risk assessment failed, using deterministic fallback");
                record_fallback(&self.collector, "risk:assessment", &e.to_string());
                return deterministic_risk_proposals(&self.vendors);
            }
        };

        let mut proposals = Vec::new();
        if let Some(assessments) = result.get("assessments").and_then(|a| a.as_array()) {
            for assessment in assessments {
                let vendor_name = assessment
                    .get("vendor")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                proposals.push(ProposedFact::new(
                    ContextKey::Evaluations,
                    format!("risk:score:{}", slug(vendor_name)),
                    serde_json::to_string(assessment).unwrap_or_default(),
                    "live-vendor-risk",
                ));
            }
        }

        if proposals.is_empty() {
            return deterministic_risk_proposals(&self.vendors);
        }
        AgentEffect::with_proposals(proposals)
    }
}

fn deterministic_risk_proposals(vendors: &[VendorInput]) -> AgentEffect {
    let proposals: Vec<ProposedFact> = vendors
        .iter()
        .map(|v| {
            let level = if v.risk_score > 30.0 {
                "high"
            } else if v.risk_score > 15.0 {
                "medium"
            } else {
                "low"
            };
            ProposedFact::new(
                ContextKey::Evaluations,
                format!("risk:score:{}", slug(&v.name)),
                serde_json::json!({
                    "vendor": v.name,
                    "overall_risk": level,
                    "rationale": "deterministic fallback"
                })
                .to_string(),
                "live-vendor-risk",
            )
        })
        .collect();
    AgentEffect::with_proposals(proposals)
}

// ---------------------------------------------------------------------------
// Decision synthesis — strong model
// ---------------------------------------------------------------------------

pub(crate) struct LiveDecisionSynthesisAgent {
    pub(crate) llm: SelectedLlm,
    pub(crate) collector: InMemoryLlmCallCollector,
    pub(crate) prior_context: String,
}

#[async_trait]
impl Suggestor for LiveDecisionSynthesisAgent {
    fn name(&self) -> &str {
        "live-decision-synthesis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[
            ContextKey::Proposals,
            ContextKey::Evaluations,
            ContextKey::Strategies,
        ]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Strategies)
            .iter()
            .any(|f| f.id().as_str() == "strategy:vendor-sel:decision")
            && ctx
                .get(ContextKey::Proposals)
                .iter()
                .any(|f| f.id().as_str() == "vendor:shortlist")
            && !ctx
                .get(ContextKey::Evaluations)
                .iter()
                .any(|f| f.id().as_str() == "decision:recommendation")
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        if ctx
            .get(ContextKey::Evaluations)
            .iter()
            .any(|f| f.id().as_str() == "decision:recommendation")
        {
            return AgentEffect::with_proposals(vec![]);
        }

        let all_facts: Vec<String> = ctx
            .get(ContextKey::Seeds)
            .iter()
            .chain(ctx.get(ContextKey::Evaluations).iter())
            .chain(ctx.get(ContextKey::Proposals).iter())
            .map(|f| format!("[{}] {}", f.id(), f.content()))
            .collect();

        let prior_section = if self.prior_context.is_empty() {
            String::new()
        } else {
            format!("\n\nPrior decision context:\n{}", self.prior_context)
        };

        let prompt = format!(
            r#"You are a senior procurement decision maker. Based on all the evidence below, synthesize a final vendor recommendation.

Evidence:
{evidence}
{prior_section}

Respond with JSON only:
{{
  "recommendation": "vendor name",
  "confidence": 0.0 to 1.0,
  "needs_human_review": true or false,
  "rationale": "2-3 sentence explanation",
  "trade_offs": ["trade-off 1", "trade-off 2"]
}}"#,
            evidence = all_facts.join("\n"),
        );

        let result: serde_json::Value = match call_llm_json(
            &self.llm,
            "You are a careful procurement decision maker. Respond with JSON only.",
            &prompt,
            1500,
            "decision:synthesis",
            &self.collector,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "LLM decision synthesis failed, using deterministic fallback");
                record_fallback(&self.collector, "decision:synthesis", &e.to_string());
                let governed_vendor = selected_shortlist_vendor(ctx);
                let (recommendation, confidence, needs_human_review) =
                    if let Some(vendor) = governed_vendor {
                        (
                            format!("{vendor} recommended (governed shortlist rank #1)"),
                            0.75,
                            false,
                        )
                    } else {
                        ("manual review required".to_string(), 0.5, true)
                    };
                return AgentEffect::with_proposals(vec![ProposedFact::new(
                    ContextKey::Evaluations,
                    "decision:recommendation".to_string(),
                    serde_json::json!({
                        "recommendation": recommendation,
                        "confidence": confidence,
                        "needs_human_review": needs_human_review,
                        "llm_needs_human_review": true,
                        "rationale": format!("LLM synthesis failed: {e}")
                    })
                    .to_string(),
                    "live-decision-synthesis",
                )]);
            }
        };

        // Extract fields with safe defaults — LLM may return nulls
        let llm_recommendation = result
            .get("recommendation")
            .and_then(|v| v.as_str())
            .unwrap_or("manual review required")
            .to_string();
        let governed_vendor = selected_shortlist_vendor(ctx);
        let recommendation = governed_vendor
            .as_ref()
            .map(|vendor| format!("{vendor} recommended (governed shortlist rank #1)"))
            .unwrap_or_else(|| llm_recommendation.clone());
        let confidence = result
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);
        let llm_needs_human_review = result
            .get("needs_human_review")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let needs_human_review = if governed_vendor.is_some() {
            false
        } else {
            llm_needs_human_review
        };
        let rationale = result
            .get("rationale")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let trade_offs = result
            .get("trade_offs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        AgentEffect::with_proposals(vec![ProposedFact::new(
            ContextKey::Evaluations,
            "decision:recommendation".to_string(),
            serde_json::json!({
                "recommendation": recommendation,
                "llm_recommendation": llm_recommendation,
                "confidence": confidence,
                "needs_human_review": needs_human_review,
                "llm_needs_human_review": llm_needs_human_review,
                "rationale": rationale,
                "trade_offs": trade_offs
            })
            .to_string(),
            "live-decision-synthesis",
        )])
    }
}

fn selected_shortlist_vendor(ctx: &dyn ContextView) -> Option<String> {
    ctx.get(ContextKey::Proposals)
        .iter()
        .find(|fact| fact.id().as_str() == "vendor:shortlist")
        .and_then(|fact| serde_json::from_str::<serde_json::Value>(fact.content()).ok())
        .and_then(|payload| {
            payload
                .get("shortlist")
                .and_then(|value| value.as_array())
                .and_then(|shortlist| shortlist.first())
                .and_then(|entry| entry.get("vendor_name"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        })
}
