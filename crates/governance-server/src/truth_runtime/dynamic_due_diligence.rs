//! Advanced truth executor: dynamic-due-diligence
//!
//! This ports the good shape from Monterro's convergent due-diligence flow
//! into an offline-first hackathon example:
//! Organism seeds typed research strategies, Converge governs a dynamic loop,
//! and a final brief is synthesized from explicit evidence and contradictions.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use converge_kernel::{Context, Engine, TypesRunHooks};
use converge_pack::{AgentEffect, Context as ContextView, ContextKey, ProposedFact, Suggestor};
use governance_kernel::{Actor, DecisionRecord, InMemoryStore};
use governance_truths::{DynamicDueDiligenceEvaluator, build_intent, find_truth};
use organism_pack::{IntentPacket, Plan, PlanStep, ReasoningSystem, SharedBudget};
use organism_runtime::Registry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{TruthExecutionResult, TruthProjection, common};

const FINAL_BRIEF_ID: &str = "dd:final-brief";

#[derive(Debug, Clone)]
struct PlannedStrategy {
    id: &'static str,
    _plan: Plan,
    payload: StrategyPayload,
}

#[derive(Debug, Clone)]
struct DueDiligencePlanningSeed {
    _intent: IntentPacket,
    strategies: Vec<PlannedStrategy>,
    registry_pack_count: usize,
}

impl DueDiligencePlanningSeed {
    fn strategy_facts(&self) -> Result<Vec<(&str, String)>, String> {
        self.strategies
            .iter()
            .map(|strategy| {
                serde_json::to_string(&strategy.payload)
                    .map(|content| (strategy.id, content))
                    .map_err(|error| {
                        format!(
                            "failed to serialize due-diligence strategy {}: {error}",
                            strategy.id
                        )
                    })
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StrategyPayload {
    label: String,
    mode: StrategyMode,
    query: String,
    reason: String,
    focus_areas: Vec<String>,
    gap_category: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum StrategyMode {
    Breadth,
    Depth,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ResearchProvider {
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResearchClaimPayload {
    category: String,
    claim: String,
    confidence: f64,
    topic: Option<String>,
    normalized_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResearchSignalPayload {
    document_id: String,
    title: String,
    url: String,
    provider: ResearchProvider,
    summary: String,
    categories: Vec<String>,
    claims: Vec<ResearchClaimPayload>,
    strategy_id: String,
    strategy_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HypothesisPayload {
    category: String,
    claim: String,
    source_id: String,
    source_title: String,
    source_url: String,
    provider: ResearchProvider,
    confidence: f64,
    topic: Option<String>,
    normalized_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContradictionPayload {
    topic: String,
    observed_values: Vec<String>,
    claim_ids: Vec<String>,
    significance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DueDiligenceReport {
    company_name: String,
    focus_areas: Vec<String>,
    executive_summary: String,
    market_analysis: Vec<String>,
    competitive_landscape: Vec<String>,
    technology_assessment: Vec<String>,
    ownership_and_financials: Vec<String>,
    contradictions: Vec<String>,
    remaining_gaps: Vec<String>,
    recommendation: String,
    confidence: f64,
    needs_human_review: bool,
}

#[derive(Debug, Clone)]
struct ResearchDocument {
    id: String,
    title: String,
    url: String,
    provider: ResearchProvider,
    mode: StrategyMode,
    stage: ResearchStage,
    tags: Vec<String>,
    summary: String,
    claims: Vec<ResearchClaimPayload>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResearchStage {
    Initial,
    FollowUp,
}

struct PlanningSeedSuggestor {
    planning: DueDiligencePlanningSeed,
}

#[async_trait]
impl Suggestor for PlanningSeedSuggestor {
    fn name(&self) -> &str {
        "dd-planning-seed"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        !ctx.get(ContextKey::Strategies)
            .iter()
            .any(|fact| fact.id.starts_with("dd:strategy:"))
    }

    async fn execute(&self, _ctx: &dyn ContextView) -> AgentEffect {
        match self.planning.strategy_facts() {
            Ok(strategies) => AgentEffect::with_proposals(
                strategies
                    .into_iter()
                    .map(|(id, content)| {
                        ProposedFact::new(
                            ContextKey::Strategies,
                            id,
                            content,
                            "organism-planning:dynamic-due-diligence",
                        )
                        .with_confidence(1.0)
                    })
                    .collect(),
            ),
            Err(error) => AgentEffect::with_proposal(
                ProposedFact::new(
                    ContextKey::Constraints,
                    "dd:constraint:planning-seed",
                    error,
                    "dd-planning-seed",
                )
                .with_confidence(1.0),
            ),
        }
    }
}

fn new_dd_budget(max_searches: usize, max_analysis_passes: usize) -> SharedBudget {
    SharedBudget::new()
        .with_limit("searches", max_searches)
        .with_limit("analysis", max_analysis_passes)
}

struct BreadthResearchSuggestor {
    company: String,
    focus_areas: Vec<String>,
    budget: Arc<SharedBudget>,
    processed_strategy_ids: Mutex<HashSet<String>>,
}

impl BreadthResearchSuggestor {
    fn unprocessed_strategies(&self, ctx: &dyn ContextView) -> Vec<(String, StrategyPayload)> {
        let processed = self.processed_strategy_ids.lock().unwrap();
        collect_strategies(ctx, StrategyMode::Breadth)
            .into_iter()
            .filter(|(id, _)| !processed.contains(id))
            .collect()
    }
}

#[async_trait]
impl Suggestor for BreadthResearchSuggestor {
    fn name(&self) -> &str {
        "dd-breadth-research"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        self.budget.remaining("searches") > 0 && !self.unprocessed_strategies(ctx).is_empty()
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let strategies = self.unprocessed_strategies(ctx);
        let known_signal_ids = known_fact_ids(ctx, ContextKey::Signals);
        let mut emitted_signal_ids = HashSet::new();
        let mut proposals = Vec::new();

        for (strategy_id, strategy) in strategies {
            if !self.budget.try_use("searches") {
                break;
            }

            for document in search_corpus(
                &self.company,
                &self.focus_areas,
                StrategyMode::Breadth,
                strategy.gap_category.as_deref(),
            ) {
                let signal_id = format!("dd:signal:{}", document.id);
                if known_signal_ids.contains(&signal_id) || emitted_signal_ids.contains(&signal_id)
                {
                    continue;
                }

                let payload = ResearchSignalPayload {
                    document_id: document.id.clone(),
                    title: document.title.clone(),
                    url: document.url.clone(),
                    provider: document.provider,
                    summary: document.summary.clone(),
                    categories: document.tags.clone(),
                    claims: document.claims.clone(),
                    strategy_id: strategy_id.clone(),
                    strategy_label: strategy.label.clone(),
                };

                if let Ok(content) = serde_json::to_string(&payload) {
                    proposals.push(
                        ProposedFact::new(
                            ContextKey::Signals,
                            &signal_id,
                            content,
                            "mock-breadth-research",
                        )
                        .with_confidence(0.95),
                    );
                    emitted_signal_ids.insert(signal_id);
                }
            }

            self.processed_strategy_ids
                .lock()
                .unwrap()
                .insert(strategy_id);
        }

        AgentEffect { proposals }
    }
}

struct DepthResearchSuggestor {
    company: String,
    focus_areas: Vec<String>,
    budget: Arc<SharedBudget>,
    processed_strategy_ids: Mutex<HashSet<String>>,
}

impl DepthResearchSuggestor {
    fn unprocessed_strategies(&self, ctx: &dyn ContextView) -> Vec<(String, StrategyPayload)> {
        let processed = self.processed_strategy_ids.lock().unwrap();
        collect_strategies(ctx, StrategyMode::Depth)
            .into_iter()
            .filter(|(id, _)| !processed.contains(id))
            .collect()
    }
}

#[async_trait]
impl Suggestor for DepthResearchSuggestor {
    fn name(&self) -> &str {
        "dd-depth-research"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Strategies]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        self.budget.remaining("searches") > 0 && !self.unprocessed_strategies(ctx).is_empty()
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let strategies = self.unprocessed_strategies(ctx);
        let known_signal_ids = known_fact_ids(ctx, ContextKey::Signals);
        let mut emitted_signal_ids = HashSet::new();
        let mut proposals = Vec::new();

        for (strategy_id, strategy) in strategies {
            if !self.budget.try_use("searches") {
                break;
            }

            for document in search_corpus(
                &self.company,
                &self.focus_areas,
                StrategyMode::Depth,
                strategy.gap_category.as_deref(),
            ) {
                let signal_id = format!("dd:signal:{}", document.id);
                if known_signal_ids.contains(&signal_id) || emitted_signal_ids.contains(&signal_id)
                {
                    continue;
                }

                let payload = ResearchSignalPayload {
                    document_id: document.id.clone(),
                    title: document.title.clone(),
                    url: document.url.clone(),
                    provider: document.provider,
                    summary: document.summary.clone(),
                    categories: document.tags.clone(),
                    claims: document.claims.clone(),
                    strategy_id: strategy_id.clone(),
                    strategy_label: strategy.label.clone(),
                };

                if let Ok(content) = serde_json::to_string(&payload) {
                    proposals.push(
                        ProposedFact::new(
                            ContextKey::Signals,
                            &signal_id,
                            content,
                            "mock-depth-research",
                        )
                        .with_confidence(0.95),
                    );
                    emitted_signal_ids.insert(signal_id);
                }
            }

            self.processed_strategy_ids
                .lock()
                .unwrap()
                .insert(strategy_id);
        }

        AgentEffect { proposals }
    }
}

struct FactExtractorSuggestor {
    processed_signal_ids: Mutex<HashSet<String>>,
}

#[async_trait]
impl Suggestor for FactExtractorSuggestor {
    fn name(&self) -> &str {
        "dd-fact-extractor"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Signals]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        ctx.get(ContextKey::Signals).iter().any(|signal| {
            signal.id.starts_with("dd:signal:")
                && !self
                    .processed_signal_ids
                    .lock()
                    .unwrap()
                    .contains(&signal.id)
        })
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let mut proposals = Vec::new();
        let existing_hypotheses = known_fact_ids(ctx, ContextKey::Hypotheses);

        for signal in ctx.get(ContextKey::Signals) {
            if !signal.id.starts_with("dd:signal:") {
                continue;
            }

            if self
                .processed_signal_ids
                .lock()
                .unwrap()
                .contains(&signal.id)
            {
                continue;
            }

            let Ok(payload) = serde_json::from_str::<ResearchSignalPayload>(&signal.content) else {
                self.processed_signal_ids
                    .lock()
                    .unwrap()
                    .insert(signal.id.clone());
                continue;
            };

            for claim in payload.claims {
                let claim_key = slug(&claim.claim);
                let hypothesis_id = format!("dd:hypothesis:{}:{claim_key}", payload.document_id);
                if existing_hypotheses.contains(&hypothesis_id) {
                    continue;
                }

                let hypothesis = HypothesisPayload {
                    category: claim.category,
                    claim: claim.claim,
                    source_id: signal.id.clone(),
                    source_title: payload.title.clone(),
                    source_url: payload.url.clone(),
                    provider: payload.provider,
                    confidence: claim.confidence,
                    topic: claim.topic,
                    normalized_value: claim.normalized_value,
                };

                if let Ok(content) = serde_json::to_string(&hypothesis) {
                    proposals.push(
                        ProposedFact::new(
                            ContextKey::Hypotheses,
                            &hypothesis_id,
                            content,
                            "dd-fact-extractor",
                        )
                        .with_confidence(hypothesis.confidence),
                    );
                }
            }

            self.processed_signal_ids
                .lock()
                .unwrap()
                .insert(signal.id.clone());
        }

        AgentEffect { proposals }
    }
}

struct ContradictionSuggestor {
    processed_topics: Mutex<HashSet<String>>,
}

#[async_trait]
impl Suggestor for ContradictionSuggestor {
    fn name(&self) -> &str {
        "dd-contradiction-detector"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        let processed = self.processed_topics.lock().unwrap();
        contradiction_candidates(ctx)
            .keys()
            .any(|topic| !processed.contains(topic))
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let candidates = contradiction_candidates(ctx);
        let processed = self.processed_topics.lock().unwrap().clone();
        let mut proposals = Vec::new();

        for (topic, values) in candidates {
            if processed.contains(&topic) {
                continue;
            }

            let observed_values: HashSet<String> =
                values.iter().map(|(_, value, _)| value.clone()).collect();
            if observed_values.len() < 2 {
                continue;
            }

            let payload = ContradictionPayload {
                topic: topic.clone(),
                observed_values: observed_values.into_iter().collect(),
                claim_ids: values.iter().map(|(id, _, _)| id.clone()).collect(),
                significance: format!(
                    "Different sources disagree on {topic}; a human should reconcile the discrepancy."
                ),
            };

            if let Ok(content) = serde_json::to_string(&payload) {
                let contradiction_id = format!("dd:contradiction:{topic}");
                proposals.push(
                    ProposedFact::new(
                        ContextKey::Evaluations,
                        &contradiction_id,
                        content,
                        "dd-contradiction-detector",
                    )
                    .with_confidence(0.9),
                );
                self.processed_topics.lock().unwrap().insert(topic);
            }
        }

        AgentEffect { proposals }
    }
}

struct LooseEndSuggestor {
    focus_areas: Vec<String>,
    budget: Arc<SharedBudget>,
    generated_gap_ids: Mutex<HashSet<String>>,
}

#[async_trait]
impl Suggestor for LooseEndSuggestor {
    fn name(&self) -> &str {
        "dd-loose-end-detector"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses, ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        if self.budget.remaining("analysis") == 0 {
            return false;
        }
        !pending_gaps(
            ctx,
            &self.focus_areas,
            &self.generated_gap_ids.lock().unwrap(),
        )
        .is_empty()
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        if !self.budget.try_use("analysis") {
            return AgentEffect::default();
        }

        let mut proposals = Vec::new();
        let gaps = pending_gaps(
            ctx,
            &self.focus_areas,
            &self.generated_gap_ids.lock().unwrap(),
        );

        for (gap_id, strategy) in gaps.into_iter().take(2) {
            if let Ok(content) = serde_json::to_string(&strategy) {
                proposals.push(
                    ProposedFact::new(
                        ContextKey::Strategies,
                        &gap_id,
                        content,
                        "dd-loose-end-detector",
                    )
                    .with_confidence(0.92),
                );
                self.generated_gap_ids.lock().unwrap().insert(gap_id);
            }
        }

        AgentEffect { proposals }
    }
}

struct SynthesisSuggestor {
    company: String,
    focus_areas: Vec<String>,
    last_hypothesis_count: Mutex<usize>,
    stable_cycles: Mutex<usize>,
}

#[async_trait]
impl Suggestor for SynthesisSuggestor {
    fn name(&self) -> &str {
        "dd-synthesis"
    }

    fn dependencies(&self) -> &[ContextKey] {
        &[ContextKey::Hypotheses, ContextKey::Evaluations]
    }

    fn accepts(&self, ctx: &dyn ContextView) -> bool {
        if ctx
            .get(ContextKey::Proposals)
            .iter()
            .any(|fact| fact.id == FINAL_BRIEF_ID)
        {
            return false;
        }

        let current = ctx.get(ContextKey::Hypotheses).len();
        let mut last = self.last_hypothesis_count.lock().unwrap();
        let mut stable = self.stable_cycles.lock().unwrap();

        if current == *last && current > 0 {
            *stable += 1;
        } else {
            *stable = 0;
            *last = current;
        }

        let required = critical_categories_covered(ctx);
        *stable >= 1 && required
    }

    async fn execute(&self, ctx: &dyn ContextView) -> AgentEffect {
        let report = build_report(&self.company, &self.focus_areas, ctx);
        match serde_json::to_string(&report) {
            Ok(content) => AgentEffect::with_proposal(
                ProposedFact::new(
                    ContextKey::Proposals,
                    FINAL_BRIEF_ID,
                    content,
                    "dd-synthesis",
                )
                .with_confidence(report.confidence),
            ),
            Err(error) => AgentEffect::with_proposal(
                ProposedFact::new(
                    ContextKey::Constraints,
                    "dd:constraint:synthesis",
                    format!("failed to serialize due-diligence report: {error}"),
                    "dd-synthesis",
                )
                .with_confidence(1.0),
            ),
        }
    }
}

pub async fn execute(
    store: &InMemoryStore,
    inputs: &HashMap<String, String>,
    persist: bool,
) -> Result<TruthExecutionResult, String> {
    let truth = find_truth("dynamic-due-diligence").ok_or("truth not found")?;
    let intent = build_intent(truth);
    let company = common::optional_input(inputs, "company")
        .or_else(|| common::optional_input(inputs, "company_name"))
        .ok_or_else(|| "missing required input: company".to_string())?;
    let focus_areas = common::optional_input(inputs, "focus_areas")
        .unwrap_or_else(|| "financials, compliance, competition".into())
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    let planning_seed = build_due_diligence_planning_seed(&company, &focus_areas);
    let budget = Arc::new(new_dd_budget(6, 3));

    let mut engine = Engine::new();
    engine.register_suggestor_in_pack(
        "planning-pack",
        PlanningSeedSuggestor {
            planning: planning_seed.clone(),
        },
    );
    engine.register_suggestor_in_pack(
        "research-pack",
        BreadthResearchSuggestor {
            company: company.clone(),
            focus_areas: focus_areas.clone(),
            budget: Arc::clone(&budget),
            processed_strategy_ids: Mutex::new(HashSet::new()),
        },
    );
    engine.register_suggestor_in_pack(
        "research-pack",
        DepthResearchSuggestor {
            company: company.clone(),
            focus_areas: focus_areas.clone(),
            budget: Arc::clone(&budget),
            processed_strategy_ids: Mutex::new(HashSet::new()),
        },
    );
    engine.register_suggestor_in_pack(
        "analysis-pack",
        FactExtractorSuggestor {
            processed_signal_ids: Mutex::new(HashSet::new()),
        },
    );
    engine.register_suggestor_in_pack(
        "analysis-pack",
        ContradictionSuggestor {
            processed_topics: Mutex::new(HashSet::new()),
        },
    );
    engine.register_suggestor_in_pack(
        "analysis-pack",
        LooseEndSuggestor {
            focus_areas: focus_areas.clone(),
            budget: Arc::clone(&budget),
            generated_gap_ids: Mutex::new(HashSet::new()),
        },
    );
    engine.register_suggestor_in_pack(
        "synthesis-pack",
        SynthesisSuggestor {
            company: company.clone(),
            focus_areas: focus_areas.clone(),
            last_hypothesis_count: Mutex::new(0),
            stable_cycles: Mutex::new(0),
        },
    );

    let result = engine
        .run_with_types_intent_and_hooks(
            Context::new(),
            &intent,
            TypesRunHooks {
                criterion_evaluator: Some(Arc::new(DynamicDueDiligenceEvaluator)),
                event_observer: None,
            },
        )
        .await
        .map_err(|error| format!("convergence failed: {error}"))?;

    let final_report: DueDiligenceReport =
        common::payload_from_result(&result, ContextKey::Proposals, FINAL_BRIEF_ID)?;
    let projection = if persist {
        let company_for_projection = company.clone();
        let final_report_for_projection = final_report.clone();
        let write_result = store
            .write_with_events(|kernel| {
                let actor = Actor::agent("dd-synthesis");
                let vendor_id = kernel
                    .vendors
                    .values()
                    .find(|vendor| vendor.name.eq_ignore_ascii_case(&company_for_projection))
                    .map(|vendor| vendor.id)
                    .unwrap_or_else(|| {
                        kernel
                            .register_vendor(
                                company_for_projection.clone(),
                                "Dynamic due-diligence subject".into(),
                                &actor,
                            )
                            .id
                    });

                kernel.record_decision(
                    DecisionRecord {
                        id: Uuid::new_v4(),
                        truth_key: "dynamic-due-diligence".into(),
                        recommendation: final_report_for_projection.recommendation.clone(),
                        confidence_bps: common::converge_confidence_to_bps(
                            final_report_for_projection.confidence,
                        ),
                        rationale: format!(
                            "Dynamic due diligence synthesized from {} hypotheses, {} contradictions, and Organism standard packs ({})",
                            result.context.get(ContextKey::Hypotheses).len(),
                            result.context.get(ContextKey::Evaluations).len(),
                            planning_seed.registry_pack_count,
                        ),
                        vendor_id: Some(vendor_id),
                        needs_human_review: final_report_for_projection.needs_human_review,
                        decided_by: actor.clone(),
                        decided_at: Utc::now(),
                    },
                    &actor,
                );
                Ok(())
            })
            .map_err(|error| format!("projection failed: {error}"))?;
        Some(TruthProjection {
            events_emitted: write_result.events.len(),
            details: Some(serde_json::to_value(final_report.clone()).map_err(|error| {
                format!("failed to serialize due-diligence projection: {error}")
            })?),
        })
    } else {
        None
    };

    Ok(TruthExecutionResult {
        converged: result.converged,
        cycles: result.cycles,
        stop_reason: format!("{:?}", result.stop_reason),
        criteria_outcomes: result
            .criteria_outcomes
            .iter()
            .map(|outcome| super::CriterionOutcomeView {
                criterion: outcome.criterion.description.clone(),
                result: format!("{:?}", outcome.result),
            })
            .collect(),
        projection,
        llm_calls: None,
    })
}

fn build_due_diligence_planning_seed(
    company: &str,
    focus_areas: &[String],
) -> DueDiligencePlanningSeed {
    let registry = Registry::with_standard_packs();
    let focus_text = if focus_areas.is_empty() {
        "financials, compliance, and competition".to_string()
    } else {
        focus_areas.join(", ")
    };
    let intent = IntentPacket::new(
        format!("Build a due diligence brief for {company}"),
        Utc::now() + Duration::hours(1),
    )
    .with_context(serde_json::json!({
        "company": company,
        "focus_areas": focus_areas,
        "goal": "build an evidence-backed due diligence brief with explicit contradictions and gaps",
    }))
    .with_authority(vec!["research".to_string()]);

    let breadth_company = StrategyPayload {
        label: "company-overview".into(),
        mode: StrategyMode::Breadth,
        query: format!("{company} product customers market position"),
        reason: "Establish the baseline company and market story".into(),
        focus_areas: focus_areas.to_vec(),
        gap_category: None,
    };
    let breadth_competition = StrategyPayload {
        label: "competitive-landscape".into(),
        mode: StrategyMode::Breadth,
        query: format!("{company} competitors trends regions"),
        reason: "Map the competitive landscape before deeper analysis".into(),
        focus_areas: focus_areas.to_vec(),
        gap_category: None,
    };
    let depth_technology = StrategyPayload {
        label: "technology-depth".into(),
        mode: StrategyMode::Depth,
        query: format!("{company} architecture integrations security posture"),
        reason: format!("Deepen technical diligence across {focus_text}"),
        focus_areas: focus_areas.to_vec(),
        gap_category: None,
    };
    let depth_financials = StrategyPayload {
        label: "ownership-and-financials".into(),
        mode: StrategyMode::Depth,
        query: format!("{company} revenue investors ownership"),
        reason: "Establish ownership and financial evidence".into(),
        focus_areas: focus_areas.to_vec(),
        gap_category: None,
    };

    let mut plans = Vec::new();

    let mut plan = Plan::new(
        &intent,
        "Search broad for the company story and customer context",
    );
    plan.contributor = ReasoningSystem::DomainModel;
    plan.steps = vec![PlanStep {
        action: breadth_company.query.clone(),
        expected_effect: breadth_company.reason.clone(),
    }];
    plans.push(PlannedStrategy {
        id: "dd:strategy:breadth-overview",
        _plan: plan,
        payload: breadth_company,
    });

    let mut plan = Plan::new(
        &intent,
        "Search broad for competition, trends, and market movement",
    );
    plan.contributor = ReasoningSystem::CausalAnalysis;
    plan.steps = vec![PlanStep {
        action: breadth_competition.query.clone(),
        expected_effect: breadth_competition.reason.clone(),
    }];
    plans.push(PlannedStrategy {
        id: "dd:strategy:breadth-competition",
        _plan: plan,
        payload: breadth_competition,
    });

    let mut plan = Plan::new(
        &intent,
        "Search deep for architecture, integrations, and security evidence",
    );
    plan.contributor = ReasoningSystem::ConstraintSolver;
    plan.steps = vec![PlanStep {
        action: depth_technology.query.clone(),
        expected_effect: depth_technology.reason.clone(),
    }];
    plans.push(PlannedStrategy {
        id: "dd:strategy:depth-technology",
        _plan: plan,
        payload: depth_technology,
    });

    let mut plan = Plan::new(
        &intent,
        "Search deep for ownership, financing, and commercial health",
    );
    plan.contributor = ReasoningSystem::CostEstimation;
    plan.steps = vec![PlanStep {
        action: depth_financials.query.clone(),
        expected_effect: depth_financials.reason.clone(),
    }];
    plans.push(PlannedStrategy {
        id: "dd:strategy:depth-financials",
        _plan: plan,
        payload: depth_financials,
    });

    DueDiligencePlanningSeed {
        _intent: intent,
        strategies: plans,
        registry_pack_count: registry.packs().len(),
    }
}

fn collect_strategies(ctx: &dyn ContextView, mode: StrategyMode) -> Vec<(String, StrategyPayload)> {
    ctx.get(ContextKey::Strategies)
        .iter()
        .filter_map(|fact| {
            serde_json::from_str::<StrategyPayload>(&fact.content)
                .ok()
                .filter(|payload| payload.mode == mode)
                .map(|payload| (fact.id.clone(), payload))
        })
        .collect()
}

fn known_fact_ids(ctx: &dyn ContextView, key: ContextKey) -> HashSet<String> {
    ctx.get(key).iter().map(|fact| fact.id.clone()).collect()
}

fn contradiction_candidates(
    ctx: &dyn ContextView,
) -> HashMap<String, Vec<(String, String, String)>> {
    let mut grouped: HashMap<String, Vec<(String, String, String)>> = HashMap::new();
    for fact in ctx.get(ContextKey::Hypotheses) {
        let Ok(payload) = serde_json::from_str::<HypothesisPayload>(&fact.content) else {
            continue;
        };
        let Some(topic) = payload.topic else {
            continue;
        };
        let Some(normalized_value) = payload.normalized_value else {
            continue;
        };
        grouped
            .entry(topic)
            .or_default()
            .push((fact.id.clone(), normalized_value, payload.claim));
    }
    grouped
}

fn pending_gaps(
    ctx: &dyn ContextView,
    focus_areas: &[String],
    generated_gap_ids: &HashSet<String>,
) -> Vec<(String, StrategyPayload)> {
    let categories = hypothesis_categories(ctx);
    let contradictions = ctx
        .get(ContextKey::Evaluations)
        .iter()
        .filter(|fact| fact.id.starts_with("dd:contradiction:"))
        .count();
    let wants_financials = focus_areas
        .iter()
        .any(|focus| focus.eq_ignore_ascii_case("financials"));
    let wants_compliance = focus_areas
        .iter()
        .any(|focus| focus.eq_ignore_ascii_case("compliance"));

    let mut gaps = Vec::new();

    if (!categories.contains("compliance") || wants_compliance)
        && !generated_gap_ids.contains("dd:strategy:gap:compliance")
    {
        gaps.push((
            "dd:strategy:gap:compliance".to_string(),
            StrategyPayload {
                label: "compliance-gap".into(),
                mode: StrategyMode::Depth,
                query: "compliance certifications residency obligations".into(),
                reason: "Close the governance and compliance gap before synthesis".into(),
                focus_areas: focus_areas.to_vec(),
                gap_category: Some("compliance".into()),
            },
        ));
    }

    if (wants_financials || contradictions == 0)
        && !generated_gap_ids.contains("dd:strategy:gap:financials-follow-up")
    {
        gaps.push((
            "dd:strategy:gap:financials-follow-up".to_string(),
            StrategyPayload {
                label: "financial-follow-up".into(),
                mode: StrategyMode::Depth,
                query: "updated ARR growth investor commentary".into(),
                reason: "Cross-check financial claims with a second source".into(),
                focus_areas: focus_areas.to_vec(),
                gap_category: Some("financials".into()),
            },
        ));
    }

    gaps
}

fn critical_categories_covered(ctx: &dyn ContextView) -> bool {
    let categories = hypothesis_categories(ctx);
    [
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
}

fn hypothesis_categories(ctx: &dyn ContextView) -> HashSet<String> {
    ctx.get(ContextKey::Hypotheses)
        .iter()
        .filter_map(|fact| serde_json::from_str::<HypothesisPayload>(&fact.content).ok())
        .map(|payload| payload.category)
        .collect()
}

fn build_report(
    company: &str,
    focus_areas: &[String],
    ctx: &dyn ContextView,
) -> DueDiligenceReport {
    let hypotheses: Vec<HypothesisPayload> = ctx
        .get(ContextKey::Hypotheses)
        .iter()
        .filter_map(|fact| serde_json::from_str::<HypothesisPayload>(&fact.content).ok())
        .collect();
    let contradictions: Vec<ContradictionPayload> = ctx
        .get(ContextKey::Evaluations)
        .iter()
        .filter_map(|fact| serde_json::from_str::<ContradictionPayload>(&fact.content).ok())
        .collect();

    let market_analysis = claims_for(&hypotheses, &["market", "customers"]);
    let competitive_landscape = claims_for(&hypotheses, &["competition"]);
    let technology_assessment = claims_for(&hypotheses, &["technology", "compliance"]);
    let ownership_and_financials = claims_for(&hypotheses, &["ownership", "financials"]);

    let contradiction_lines = contradictions
        .iter()
        .map(|contradiction| {
            format!(
                "{} disagrees across sources: {}",
                contradiction.topic,
                contradiction.observed_values.join(" vs ")
            )
        })
        .collect::<Vec<_>>();

    let remaining_gaps = {
        let categories = hypothesis_categories(ctx);
        let mut gaps = Vec::new();
        for category in [
            "product",
            "market",
            "competition",
            "technology",
            "ownership",
            "financials",
            "compliance",
        ] {
            if !categories.contains(category) {
                gaps.push(format!("More evidence is required for {category}."));
            }
        }
        gaps
    };

    let needs_human_review = !contradiction_lines.is_empty();
    let recommendation = if needs_human_review {
        format!(
            "Proceed with a guarded due-diligence review for {company}, but hold a final commitment until the contradictory financial evidence is reconciled."
        )
    } else {
        format!(
            "Proceed with a partner review for {company}; the current evidence is directionally positive and broadly consistent."
        )
    };

    let confidence = if needs_human_review { 0.72 } else { 0.84 };
    let executive_summary = format!(
        "{company} now has a governed due-diligence brief built from {} research signals and {} extracted hypotheses. Focus areas: {}.",
        ctx.get(ContextKey::Signals).len(),
        hypotheses.len(),
        if focus_areas.is_empty() {
            "baseline diligence".into()
        } else {
            focus_areas.join(", ")
        }
    );

    DueDiligenceReport {
        company_name: company.to_string(),
        focus_areas: focus_areas.to_vec(),
        executive_summary,
        market_analysis,
        competitive_landscape,
        technology_assessment,
        ownership_and_financials,
        contradictions: contradiction_lines,
        remaining_gaps,
        recommendation,
        confidence,
        needs_human_review,
    }
}

fn claims_for(hypotheses: &[HypothesisPayload], categories: &[&str]) -> Vec<String> {
    let allowed: HashSet<&str> = categories.iter().copied().collect();
    hypotheses
        .iter()
        .filter(|payload| allowed.contains(payload.category.as_str()))
        .map(|payload| payload.claim.clone())
        .collect()
}

fn search_corpus(
    company: &str,
    focus_areas: &[String],
    mode: StrategyMode,
    gap_category: Option<&str>,
) -> Vec<ResearchDocument> {
    mock_corpus(company)
        .into_iter()
        .filter(|document| document.mode == mode)
        .filter(|document| match gap_category {
            Some(category) => {
                document.stage == ResearchStage::FollowUp
                    && document
                        .tags
                        .iter()
                        .any(|tag| tag.eq_ignore_ascii_case(category))
            }
            None => document.stage == ResearchStage::Initial,
        })
        .filter(|document| {
            if gap_category.is_some() || focus_areas.is_empty() {
                return true;
            }
            let document_tags: HashSet<String> = document
                .tags
                .iter()
                .map(|tag| tag.to_ascii_lowercase())
                .collect();
            focus_areas.iter().any(|focus| {
                let focus = focus.to_ascii_lowercase();
                document_tags.contains(&focus)
            }) || document.stage == ResearchStage::Initial
        })
        .collect()
}

fn mock_corpus(company: &str) -> Vec<ResearchDocument> {
    let slugged = slug(company);
    vec![
        ResearchDocument {
            id: format!("{slugged}-overview"),
            title: format!("{company} overview and customer footprint"),
            url: format!("https://mock.local/{slugged}/overview"),
            provider: ResearchProvider::Anthropic,
            mode: StrategyMode::Breadth,
            stage: ResearchStage::Initial,
            tags: vec!["product".into(), "customers".into(), "market".into()],
            summary: format!(
                "{company} serves regulated enterprise workflows with an audit-heavy AI control plane."
            ),
            claims: vec![
                ResearchClaimPayload {
                    category: "product".into(),
                    claim: format!(
                        "{company} packages an audit-focused AI governance layer for enterprise operations teams."
                    ),
                    confidence: 0.87,
                    topic: None,
                    normalized_value: None,
                },
                ResearchClaimPayload {
                    category: "customers".into(),
                    claim: format!(
                        "{company} targets regulated mid-market and enterprise buyers that need approval-aware automation."
                    ),
                    confidence: 0.81,
                    topic: None,
                    normalized_value: None,
                },
                ResearchClaimPayload {
                    category: "market".into(),
                    claim: format!(
                        "{company} sits in the governed enterprise AI operations market rather than pure model hosting."
                    ),
                    confidence: 0.76,
                    topic: None,
                    normalized_value: None,
                },
            ],
        },
        ResearchDocument {
            id: format!("{slugged}-competition"),
            title: format!("{company} competitive landscape"),
            url: format!("https://mock.local/{slugged}/competition"),
            provider: ResearchProvider::Anthropic,
            mode: StrategyMode::Breadth,
            stage: ResearchStage::Initial,
            tags: vec!["competition".into(), "regions".into(), "market".into()],
            summary: format!(
                "{company} is compared against governance-first workflow vendors and larger horizontal platforms."
            ),
            claims: vec![
                ResearchClaimPayload {
                    category: "competition".into(),
                    claim: format!(
                        "{company} is most often compared with broader workflow and governance platforms rather than pure LLM vendors."
                    ),
                    confidence: 0.79,
                    topic: None,
                    normalized_value: None,
                },
                ResearchClaimPayload {
                    category: "market".into(),
                    claim: format!(
                        "{company} appears strongest in Europe where governance, auditability, and approval controls are selling points."
                    ),
                    confidence: 0.73,
                    topic: None,
                    normalized_value: None,
                },
            ],
        },
        ResearchDocument {
            id: format!("{slugged}-technology"),
            title: format!("{company} architecture and integrations"),
            url: format!("https://mock.local/{slugged}/technology"),
            provider: ResearchProvider::Anthropic,
            mode: StrategyMode::Depth,
            stage: ResearchStage::Initial,
            tags: vec![
                "technology".into(),
                "integrations".into(),
                "security".into(),
            ],
            summary: format!(
                "{company} emphasizes typed workflows, policy gates, and integration adapters instead of monolithic automation."
            ),
            claims: vec![
                ResearchClaimPayload {
                    category: "technology".into(),
                    claim: format!(
                        "{company} exposes typed workflow and policy integration points that support governed deployment patterns."
                    ),
                    confidence: 0.86,
                    topic: None,
                    normalized_value: None,
                },
                ResearchClaimPayload {
                    category: "technology".into(),
                    claim: format!(
                        "{company} positions its integration layer as a moat because customers can keep business logic local and audited."
                    ),
                    confidence: 0.75,
                    topic: None,
                    normalized_value: None,
                },
            ],
        },
        ResearchDocument {
            id: format!("{slugged}-financials-primary"),
            title: format!("{company} ownership and operating snapshot"),
            url: format!("https://mock.local/{slugged}/financials-primary"),
            provider: ResearchProvider::Anthropic,
            mode: StrategyMode::Depth,
            stage: ResearchStage::Initial,
            tags: vec!["ownership".into(), "financials".into(), "investors".into()],
            summary: format!(
                "{company} is described as founder-led with early institutional backing and healthy growth."
            ),
            claims: vec![
                ResearchClaimPayload {
                    category: "ownership".into(),
                    claim: format!(
                        "{company} is still founder-led, with two early-stage institutional investors on the cap table."
                    ),
                    confidence: 0.82,
                    topic: Some("ownership".into()),
                    normalized_value: Some("founder-led-plus-two-institutional".into()),
                },
                ResearchClaimPayload {
                    category: "financials".into(),
                    claim: format!(
                        "{company} is reported at approximately $18M ARR with strong year-on-year expansion."
                    ),
                    confidence: 0.7,
                    topic: Some("arr".into()),
                    normalized_value: Some("18m-arr".into()),
                },
            ],
        },
        ResearchDocument {
            id: format!("{slugged}-financials-secondary"),
            title: format!("{company} follow-up financial commentary"),
            url: format!("https://mock.local/{slugged}/financials-secondary"),
            provider: ResearchProvider::Anthropic,
            mode: StrategyMode::Depth,
            stage: ResearchStage::FollowUp,
            tags: vec!["financials".into(), "growth".into(), "risk".into()],
            summary: format!(
                "{company} has mixed third-party commentary on recent run-rate and growth quality."
            ),
            claims: vec![
                ResearchClaimPayload {
                    category: "financials".into(),
                    claim: format!(
                        "{company} is also described in follow-up commentary as closer to $22M ARR, suggesting the public numbers may be stale."
                    ),
                    confidence: 0.64,
                    topic: Some("arr".into()),
                    normalized_value: Some("22m-arr".into()),
                },
                ResearchClaimPayload {
                    category: "risk".into(),
                    claim: format!(
                        "{company} has uneven external reporting on commercial momentum, so financial diligence needs a direct management check."
                    ),
                    confidence: 0.72,
                    topic: None,
                    normalized_value: None,
                },
            ],
        },
        ResearchDocument {
            id: format!("{slugged}-compliance"),
            title: format!("{company} compliance posture and deployment controls"),
            url: format!("https://mock.local/{slugged}/compliance"),
            provider: ResearchProvider::Anthropic,
            mode: StrategyMode::Depth,
            stage: ResearchStage::FollowUp,
            tags: vec!["compliance".into(), "technology".into(), "security".into()],
            summary: format!(
                "{company} documents data-residency controls, review gates, and deployment constraints that matter in regulated settings."
            ),
            claims: vec![
                ResearchClaimPayload {
                    category: "compliance".into(),
                    claim: format!(
                        "{company} highlights EU-friendly deployment controls, approval gates, and auditable change paths for regulated buyers."
                    ),
                    confidence: 0.83,
                    topic: None,
                    normalized_value: None,
                },
                ResearchClaimPayload {
                    category: "technology".into(),
                    claim: format!(
                        "{company} keeps execution logic and policy checks explicit, which reduces governance drift during rollout."
                    ),
                    confidence: 0.78,
                    topic: None,
                    normalized_value: None,
                },
            ],
        },
    ]
}

fn slug(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn dynamic_due_diligence_executes_and_projects_report() {
        let store = InMemoryStore::new();
        let inputs = HashMap::from([
            ("company".into(), "Acme AI".into()),
            ("focus_areas".into(), "financials, compliance".into()),
        ]);

        let result = execute(&store, &inputs, true).await.unwrap();

        assert!(result.converged);
        assert_eq!(store.read(|kernel| kernel.decisions.len()).unwrap(), 1);
        assert!(result.projection.as_ref().unwrap().details.is_some());
    }

    #[tokio::test]
    async fn missing_company_returns_error() {
        let store = InMemoryStore::new();
        let error = execute(&store, &HashMap::new(), false).await.unwrap_err();
        assert!(error.contains("company"));
    }
}
