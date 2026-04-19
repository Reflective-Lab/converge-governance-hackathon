use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use anyhow::{Context as AnyhowContext, anyhow, bail};
use converge_provider::{
    BraveSearchProvider, ChatBackendSelectionConfig, SearchDepth, TavilySearchProvider,
    WebSearchBackend, WebSearchRequest, select_healthy_chat_backend,
};
use converge_provider_api::{
    ChatMessage, ChatRequest, ChatRole, DynChatBackend, ResponseFormat, SelectionCriteria,
};
use governance_telemetry::{
    InMemoryLlmCallCollector, LlmCallSink, LlmCallTelemetry, LlmUsageSummary,
};
use organism_pack::{IntentPacket, Plan, PlanContribution, PlanStep, Reasoner, ReasoningSystem};
use organism_planning::huddle::Huddle;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DueDiligenceRequest {
    pub company_name: String,
    pub product_name: Option<String>,
    pub focus_areas: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchProvider {
    Brave,
    Tavily,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub title: String,
    pub url: String,
    pub content: String,
    pub provider: SearchProvider,
    pub query: String,
    pub retrieved_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedFact {
    pub claim: String,
    pub category: String,
    pub source_indices: Vec<usize>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub topic: String,
    pub claim_a: String,
    pub source_a: String,
    pub claim_b: String,
    pub source_b: String,
    pub significance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphHooks {
    #[serde(default)]
    pub investors: Vec<String>,
    #[serde(default)]
    pub business_areas: Vec<String>,
    #[serde(default)]
    pub regions: Vec<String>,
    #[serde(default)]
    pub competitors: Vec<String>,
    #[serde(default)]
    pub company_trends: Vec<String>,
    #[serde(default)]
    pub market_trends: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pass1Consolidation {
    pub summary: String,
    #[serde(default)]
    pub key_facts: Vec<TaggedFact>,
    #[serde(default)]
    pub hooks: GraphHooks,
    #[serde(default)]
    pub contradictions: Vec<Contradiction>,
    #[serde(default)]
    pub loose_ends: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepDiveResult {
    pub question: String,
    pub hits: Vec<SearchHit>,
    pub synthesis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalReport {
    pub market_analysis: String,
    pub competitive_landscape: String,
    pub technology_assessment: String,
    #[serde(default)]
    pub risk_factors: Vec<String>,
    #[serde(default)]
    pub growth_opportunities: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPlanStep {
    pub provider: SearchProvider,
    pub query: String,
    pub expected_effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuddlePlanView {
    pub reasoner: String,
    pub rationale: String,
    pub steps: Vec<SearchPlanStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiPassReport {
    pub correlation_id: String,
    pub company_name: String,
    pub product_name: Option<String>,
    pub focus_areas: Vec<String>,
    pub llm_provider: String,
    pub llm_model: String,
    pub llm_calls: Vec<LlmCallTelemetry>,
    pub huddle_plans: Vec<HuddlePlanView>,
    pub pass1_hits: Vec<SearchHit>,
    pub pass1: Pass1Consolidation,
    pub deep_dives: Vec<DeepDiveResult>,
    pub final_report: FinalReport,
    pub remaining_loose_ends: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub company_name: String,
    pub product_name: Option<String>,
    pub focus_areas: Vec<String>,
    pub provider_override: Option<String>,
    pub model_override: Option<String>,
    pub output_path: Option<PathBuf>,
    pub deep_dives: usize,
}

#[derive(Clone)]
pub struct SelectedLlm {
    pub provider: String,
    pub model: String,
    pub model_override: Option<String>,
    pub backend: Arc<dyn DynChatBackend>,
}

#[derive(Debug, Deserialize)]
struct ReasonerPlanPayload {
    rationale: String,
    steps: Vec<ReasonerSearchStep>,
}

#[derive(Debug, Deserialize)]
struct ReasonerSearchStep {
    provider: SearchProvider,
    query: String,
    expected_effect: String,
}

#[derive(Clone)]
struct HuddleReasoner {
    name: String,
    system_type: ReasoningSystem,
    request: DueDiligenceRequest,
    guidance: &'static str,
    llm: SelectedLlm,
    llm_call_collector: InMemoryLlmCallCollector,
}

#[async_trait::async_trait]
impl Reasoner for HuddleReasoner {
    fn name(&self) -> &str {
        &self.name
    }

    fn system_type(&self) -> ReasoningSystem {
        self.system_type
    }

    async fn propose(&self, intent: &IntentPacket) -> anyhow::Result<Plan> {
        let subject = subject_label(&self.request);
        let prompt = format!(
            r#"You are participating in a software due-diligence planning huddle.

Intent:
- Goal: {}
- Subject: {}
- Focus areas: {}

Your role:
{}

Return ONLY valid JSON:
{{
  "rationale": "why this plan matters",
  "steps": [
    {{
      "provider": "brave",
      "query": "search query",
      "expected_effect": "what this search should clarify"
    }}
  ]
}}

Rules:
- Return 1-2 steps only.
- Use `brave` for broad market, competitor, and trend discovery.
- Use `tavily` for deeper technical, ownership, compliance, or financial evidence.
- Queries should be specific to the subject.
- Prefer queries that reduce diligence ambiguity, not generic background reading."#,
            intent.outcome,
            subject,
            focus_area_label(&self.request.focus_areas),
            self.guidance,
        );

        let payload: ReasonerPlanPayload = call_llm_json(
            &self.llm,
            "You are a precise due-diligence planning reasoner. Respond with JSON only.",
            &prompt,
            700,
            &format!("huddle::{}", self.name),
            &self.llm_call_collector,
        )
        .await
        .with_context(|| format!("huddle reasoner {} failed", self.name))?;

        let mut plan = Plan::new(intent, payload.rationale);
        plan.contributor = self.system_type;
        plan.steps = payload
            .steps
            .into_iter()
            .map(|step| PlanStep {
                action: encode_step_action(step.provider, &step.query),
                expected_effect: step.expected_effect,
            })
            .collect();
        Ok(plan)
    }

    fn contribute(&self, _context: &Value) -> PlanContribution {
        PlanContribution {
            system: self.system_type,
            suggestions: vec![self.guidance.to_string()],
            constraints: vec!["Keep queries specific and evidence-seeking.".to_string()],
            risks: vec![],
        }
    }
}

pub async fn run_from_cli() -> anyhow::Result<()> {
    let config = CliConfig::parse_from_env()?;
    let report = run_live_due_diligence(config.clone()).await?;

    print_report(&report);

    if let Some(path) = config.output_path {
        let body = serde_json::to_string_pretty(&report)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, body)?;
        println!("\nSaved report to {}", path.display());
    }

    Ok(())
}

pub async fn run_live_due_diligence(config: CliConfig) -> anyhow::Result<MultiPassReport> {
    load_env();

    let request = DueDiligenceRequest {
        company_name: config.company_name,
        product_name: config.product_name,
        focus_areas: config.focus_areas,
    };
    let llm = select_llm(
        config.provider_override.as_deref(),
        config.model_override.as_deref(),
    )
    .await?;
    let llm_call_collector = InMemoryLlmCallCollector::default();

    let huddle_plans = plan_with_huddle(&request, &llm, &llm_call_collector).await?;
    let correlation_id = uuid::Uuid::new_v4().to_string();

    let pass1_hits = run_pass1_searches(&request, &huddle_plans).await?;
    let pass1 = consolidate_pass1(&llm, &request, &pass1_hits, &llm_call_collector).await?;

    let loose_ends_to_chase = pass1
        .loose_ends
        .iter()
        .take(config.deep_dives)
        .cloned()
        .collect::<Vec<_>>();
    let remaining_loose_ends = pass1
        .loose_ends
        .iter()
        .skip(config.deep_dives)
        .cloned()
        .collect::<Vec<_>>();

    let mut deep_dives = Vec::new();
    for question in loose_ends_to_chase {
        let hits = run_deep_dive_searches(&request, &question).await?;
        let synthesis =
            synthesize_deep_dive(&llm, &request, &question, &hits, &llm_call_collector).await?;
        deep_dives.push(DeepDiveResult {
            question,
            hits,
            synthesis,
        });
    }

    let final_report =
        final_consolidation(&llm, &request, &pass1, &deep_dives, &llm_call_collector).await?;
    let llm_calls = llm_call_collector.snapshot();

    Ok(MultiPassReport {
        correlation_id,
        company_name: request.company_name,
        product_name: request.product_name,
        focus_areas: request.focus_areas,
        llm_provider: llm.provider,
        llm_model: llm.model,
        llm_calls,
        huddle_plans,
        pass1_hits,
        pass1,
        deep_dives,
        final_report,
        remaining_loose_ends,
    })
}

async fn run_pass1_searches(
    request: &DueDiligenceRequest,
    huddle_plans: &[HuddlePlanView],
) -> anyhow::Result<Vec<SearchHit>> {
    let mut hits = Vec::new();
    for plan in huddle_plans {
        for step in &plan.steps {
            hits.extend(run_search(step.provider, &step.query).await?);
        }
    }

    let pre_filter = hits.len();
    hits.retain(|hit| is_relevant(hit, &request.company_name));
    dedup_hits(&mut hits);
    let filtered = pre_filter.saturating_sub(hits.len());
    if filtered > 0 {
        tracing::info!("filtered {filtered} irrelevant or duplicate search results");
    }
    Ok(hits)
}

async fn run_deep_dive_searches(
    request: &DueDiligenceRequest,
    question: &str,
) -> anyhow::Result<Vec<SearchHit>> {
    let query = format!("{} {}", request.company_name, question);
    let mut hits = run_search(SearchProvider::Tavily, &query).await?;
    hits.extend(run_search(SearchProvider::Brave, &query).await?);
    hits.retain(|hit| is_relevant(hit, &request.company_name));
    dedup_hits(&mut hits);
    Ok(hits)
}

async fn plan_with_huddle(
    request: &DueDiligenceRequest,
    llm: &SelectedLlm,
    llm_call_collector: &InMemoryLlmCallCollector,
) -> anyhow::Result<Vec<HuddlePlanView>> {
    let subject = subject_label(request);
    let intent = IntentPacket::new(
        format!("Build a due diligence brief for {subject}"),
        chrono::Utc::now() + chrono::Duration::hours(1),
    )
    .with_context(serde_json::json!({
        "company_name": request.company_name,
        "product_name": request.product_name,
        "focus_areas": request.focus_areas,
        "goal": "form a broad and deep diligence flow that can surface contradictions and unresolved questions",
    }))
    .with_authority(vec!["research".to_string()]);

    let reasoners = build_huddle_reasoners(request, llm, llm_call_collector);
    let mut huddle = Huddle::new();
    for reasoner in &reasoners {
        huddle = huddle.add(Box::new(reasoner.clone()));
    }

    let plans = huddle.run(&intent).await;
    if plans.is_empty() {
        let mut errors = Vec::new();
        for reasoner in reasoners {
            if let Err(error) = reasoner.propose(&intent).await {
                errors.push(format!("{}: {error:#}", reasoner.name()));
            }
        }
        if errors.is_empty() {
            bail!("huddle produced no viable diligence plans");
        }
        bail!(
            "huddle produced no viable diligence plans:\n{}",
            errors.join("\n")
        );
    }

    let mut views = Vec::new();
    for plan in plans {
        let steps = plan
            .steps
            .iter()
            .filter_map(|step| {
                decode_step_action(&step.action).map(|(provider, query)| SearchPlanStep {
                    provider,
                    query,
                    expected_effect: step.expected_effect.clone(),
                })
            })
            .collect::<Vec<_>>();
        if !steps.is_empty() {
            views.push(HuddlePlanView {
                reasoner: format!("{:?}", plan.contributor),
                rationale: plan.rationale,
                steps,
            });
        }
    }

    if views.is_empty() {
        bail!("huddle plans did not contain executable search steps");
    }

    Ok(views)
}

fn build_huddle_reasoners(
    request: &DueDiligenceRequest,
    llm: &SelectedLlm,
    llm_call_collector: &InMemoryLlmCallCollector,
) -> Vec<HuddleReasoner> {
    vec![
        HuddleReasoner {
            name: "domain-model".into(),
            system_type: ReasoningSystem::DomainModel,
            request: request.clone(),
            guidance: "Focus on products, customer segments, positioning, and what the company appears to sell.",
            llm: llm.clone(),
            llm_call_collector: llm_call_collector.clone(),
        },
        HuddleReasoner {
            name: "causal-analysis".into(),
            system_type: ReasoningSystem::CausalAnalysis,
            request: request.clone(),
            guidance: "Focus on competitors, market trends, recent strategic moves, and causal market drivers.",
            llm: llm.clone(),
            llm_call_collector: llm_call_collector.clone(),
        },
        HuddleReasoner {
            name: "constraint-solver".into(),
            system_type: ReasoningSystem::ConstraintSolver,
            request: request.clone(),
            guidance: "Focus on technical architecture, compliance, security posture, and deployment constraints.",
            llm: llm.clone(),
            llm_call_collector: llm_call_collector.clone(),
        },
        HuddleReasoner {
            name: "cost-estimation".into(),
            system_type: ReasoningSystem::CostEstimation,
            request: request.clone(),
            guidance: "Focus on ownership, investors, pricing, revenue, efficiency, and commercial health.",
            llm: llm.clone(),
            llm_call_collector: llm_call_collector.clone(),
        },
    ]
}

async fn run_search(provider: SearchProvider, query: &str) -> anyhow::Result<Vec<SearchHit>> {
    match provider {
        SearchProvider::Brave => search_brave(query).await,
        SearchProvider::Tavily => search_tavily(query).await,
    }
}

async fn search_brave(query: &str) -> anyhow::Result<Vec<SearchHit>> {
    let query = query.to_string();
    tokio::task::spawn_blocking(move || {
        let now = chrono::Utc::now();
        let response = BraveSearchProvider::from_env()
            .context("BRAVE_API_KEY is not configured for live due diligence")?
            .search_web(
                &WebSearchRequest::new(&query)
                    .with_max_results(8)
                    .with_search_depth(SearchDepth::Advanced)
                    .with_raw_content(true),
            )
            .map_err(|error| anyhow!("brave search failed: {error}"))?;

        Ok(response
            .results
            .into_iter()
            .map(|result| SearchHit {
                title: result.title,
                url: result.url,
                content: result
                    .raw_content
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(result.content),
                provider: SearchProvider::Brave,
                query: query.clone(),
                retrieved_at: now,
            })
            .collect::<Vec<_>>())
    })
    .await
    .map_err(|error| anyhow!("brave search task failed: {error}"))?
}

async fn search_tavily(query: &str) -> anyhow::Result<Vec<SearchHit>> {
    let query = query.to_string();
    tokio::task::spawn_blocking(move || {
        let now = chrono::Utc::now();
        let response = TavilySearchProvider::from_env()
            .context("TAVILY_API_KEY is not configured for live due diligence")?
            .search_web(
                &WebSearchRequest::new(&query)
                    .with_max_results(8)
                    .with_search_depth(SearchDepth::Advanced)
                    .with_answer(true)
                    .with_raw_content(true),
            )
            .map_err(|error| anyhow!("tavily search failed: {error}"))?;

        Ok(response
            .results
            .into_iter()
            .map(|result| SearchHit {
                title: result.title,
                url: result.url,
                content: result
                    .raw_content
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or(result.content),
                provider: SearchProvider::Tavily,
                query: query.clone(),
                retrieved_at: now,
            })
            .collect::<Vec<_>>())
    })
    .await
    .map_err(|error| anyhow!("tavily search task failed: {error}"))?
}

async fn consolidate_pass1(
    llm: &SelectedLlm,
    request: &DueDiligenceRequest,
    hits: &[SearchHit],
    llm_call_sink: &InMemoryLlmCallCollector,
) -> anyhow::Result<Pass1Consolidation> {
    let subject = subject_label(request);
    let prompt = format!(
        r#"You are a software due diligence analyst. Analyze research about {subject}. Each source is numbered [Source N].

{sources}

Respond with ONLY valid JSON:
{{
  "summary": "2-3 paragraph overview",
  "key_facts": [
    {{
      "claim": "specific factual claim",
      "category": "market|technology|team|financials|competition|product|risk|governance|customers|ownership|compliance",
      "source_indices": [0, 3],
      "confidence": 0.9
    }}
  ],
  "hooks": {{
    "investors": [],
    "business_areas": [],
    "regions": [],
    "competitors": [],
    "company_trends": [],
    "market_trends": []
  }},
  "contradictions": [
    {{
      "topic": "what the disagreement is about",
      "claim_a": "Source X says...",
      "source_a": "[Source 2]",
      "claim_b": "Source Y says...",
      "source_b": "[Source 5]",
      "significance": "why this matters for diligence"
    }}
  ],
  "loose_ends": ["unanswered question 1", "question 2"]
}}

Rules:
- Every fact must cite source_indices.
- confidence: 0.9+ for primary evidence, 0.7 for strong secondary evidence, 0.5 for inferred patterns.
- Focus on product, customers, technology, competition, market, financials, ownership, compliance, and risk.
- Identify 4-8 loose ends that matter for a real investment or procurement decision.
- Do not invent facts. If the evidence is vague, say so."#,
        sources = format_hits_for_prompt(hits),
    );

    call_llm_json(
        llm,
        "You are a rigorous due diligence analyst. Respond with JSON only.",
        &prompt,
        3200,
        "pass1-consolidation",
        llm_call_sink,
    )
    .await
}

async fn synthesize_deep_dive(
    llm: &SelectedLlm,
    request: &DueDiligenceRequest,
    question: &str,
    hits: &[SearchHit],
    llm_call_sink: &InMemoryLlmCallCollector,
) -> anyhow::Result<String> {
    let prompt = format!(
        r#"You are a software due diligence analyst investigating {}.

Question: "{}"

Sources:
{}

Write a focused analysis answering the question. Cite [Source N] inline for each material claim.
If the question cannot be fully answered, state explicitly what remains unknown."#,
        subject_label(request),
        question,
        format_hits_for_prompt(hits),
    );

    call_llm_text(
        llm,
        "You are a concise due diligence analyst. No preamble.",
        &prompt,
        1200,
        &format!("deep-dive:{question}"),
        llm_call_sink,
    )
    .await
}

async fn final_consolidation(
    llm: &SelectedLlm,
    request: &DueDiligenceRequest,
    pass1: &Pass1Consolidation,
    deep_dives: &[DeepDiveResult],
    llm_call_sink: &InMemoryLlmCallCollector,
) -> anyhow::Result<FinalReport> {
    let facts_text = pass1
        .key_facts
        .iter()
        .map(|fact| {
            format!(
                "- [{}] ({:.0}%) {}",
                fact.category,
                fact.confidence * 100.0,
                fact.claim
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let deep_dive_text = deep_dives
        .iter()
        .enumerate()
        .map(|(idx, dive)| {
            format!(
                "### Deep Dive {}: {}\n{}",
                idx + 1,
                dive.question,
                dive.synthesis
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        r#"You are a senior software due diligence analyst.

Final due diligence report for {}.

## Summary
{}

## Key Facts
{}

## Deep Dives
{}

Respond with ONLY valid JSON:
{{
  "market_analysis": "single string",
  "competitive_landscape": "single string",
  "technology_assessment": "single string",
  "risk_factors": ["risk 1", "risk 2"],
  "growth_opportunities": ["opportunity 1", "opportunity 2"],
  "recommendation": "single string"
}}

Keep it specific. Do not output markdown fences."#,
        subject_label(request),
        pass1.summary,
        facts_text,
        deep_dive_text,
    );

    call_llm_json(
        llm,
        "You are a careful due diligence analyst. Respond with JSON only.",
        &prompt,
        1800,
        "final-consolidation",
        llm_call_sink,
    )
    .await
}

async fn select_llm(
    provider_override: Option<&str>,
    model_override: Option<&str>,
) -> anyhow::Result<SelectedLlm> {
    let mut config = ChatBackendSelectionConfig::from_env().unwrap_or_default();
    config.criteria = SelectionCriteria::analysis();
    if let Some(provider) = provider_override {
        config = config.with_provider_override(provider.to_string());
    }
    let selected = select_healthy_chat_backend(&config)
        .await
        .map_err(|error| anyhow!("failed to select chat backend: {error}"))?;
    Ok(SelectedLlm {
        provider: selected.provider().to_string(),
        model: selected.model().to_string(),
        model_override: model_override.map(ToString::to_string),
        backend: selected.backend,
    })
}

async fn call_llm_text(
    llm: &SelectedLlm,
    system: &str,
    prompt: &str,
    max_tokens: u32,
    context: &str,
    llm_call_sink: &impl LlmCallSink,
) -> anyhow::Result<String> {
    let started_at = Instant::now();
    let response = llm
        .backend
        .chat(ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: prompt.to_string(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: Some(system.to_string()),
            tools: Vec::new(),
            response_format: ResponseFormat::Text,
            max_tokens: Some(max_tokens),
            temperature: Some(0.2),
            stop_sequences: Vec::new(),
            model: llm.model_override.clone(),
        })
        .await
        .map_err(|error| anyhow!("chat request failed: {error}"))?;

    push_llm_call_telemetry(
        llm,
        context,
        started_at.elapsed(),
        response.usage.as_ref().map(|usage| LlmUsageSummary {
            prompt_tokens: Some(u64::from(usage.prompt_tokens)),
            completion_tokens: Some(u64::from(usage.completion_tokens)),
            total_tokens: Some(u64::from(usage.total_tokens)),
        }),
        response
            .finish_reason
            .as_ref()
            .map(|reason| format!("{reason:?}")),
        llm_call_sink,
    );

    Ok(response.content.trim().to_string())
}

async fn call_llm_json<T: for<'de> Deserialize<'de>>(
    llm: &SelectedLlm,
    system: &str,
    prompt: &str,
    max_tokens: u32,
    context: &str,
    llm_call_sink: &impl LlmCallSink,
) -> anyhow::Result<T> {
    let started_at = Instant::now();
    let response = llm
        .backend
        .chat(ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: prompt.to_string(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: Some(system.to_string()),
            tools: Vec::new(),
            response_format: ResponseFormat::Json,
            max_tokens: Some(max_tokens),
            temperature: Some(0.2),
            stop_sequences: Vec::new(),
            model: llm.model_override.clone(),
        })
        .await
        .map_err(|error| anyhow!("json chat request failed: {error}"))?;

    push_llm_call_telemetry(
        llm,
        context,
        started_at.elapsed(),
        response.usage.as_ref().map(|usage| LlmUsageSummary {
            prompt_tokens: Some(u64::from(usage.prompt_tokens)),
            completion_tokens: Some(u64::from(usage.completion_tokens)),
            total_tokens: Some(u64::from(usage.total_tokens)),
        }),
        response
            .finish_reason
            .as_ref()
            .map(|reason| format!("{reason:?}")),
        llm_call_sink,
    );

    let raw = strip_markdown_fences(&response.content);
    parse_json_response(llm, &raw, max_tokens, context, llm_call_sink).await
}

async fn parse_json_response<T: for<'de> Deserialize<'de>>(
    llm: &SelectedLlm,
    raw: &str,
    max_tokens: u32,
    context: &str,
    llm_call_sink: &impl LlmCallSink,
) -> anyhow::Result<T> {
    let repaired = repair_truncated_json(&strip_trailing_commas(raw));
    match serde_json::from_str::<T>(&repaired) {
        Ok(parsed) => Ok(parsed),
        Err(parse_error) => {
            let normalized = repair_json_with_llm(
                llm,
                raw,
                max_tokens,
                &format!("{context} (repair-json)"),
                llm_call_sink,
            )
            .await?;
            let normalized = repair_truncated_json(&strip_trailing_commas(&normalized));
            serde_json::from_str(&normalized).map_err(|repair_error| {
                anyhow!(
                    "failed to parse llm json: {parse_error}; repair failed: {repair_error}\n\nraw preview:\n{}",
                    &raw[..raw.len().min(800)]
                )
            })
        }
    }
}

async fn repair_json_with_llm(
    llm: &SelectedLlm,
    raw: &str,
    max_tokens: u32,
    context: &str,
    llm_call_sink: &impl LlmCallSink,
) -> anyhow::Result<String> {
    let prompt = format!(
        r#"Repair the following malformed JSON so that it becomes valid JSON.

Rules:
- Preserve the original meaning and content.
- Do not add explanations.
- Return JSON only.
- Remove trailing commas, close arrays/objects, and keep existing keys if present.

Malformed JSON:
{raw}"#
    );

    call_llm_text(
        llm,
        "You repair malformed JSON. Return JSON only.",
        &prompt,
        max_tokens.min(2000),
        context,
        llm_call_sink,
    )
    .await
}

fn push_llm_call_telemetry(
    llm: &SelectedLlm,
    context: &str,
    elapsed: std::time::Duration,
    usage: Option<LlmUsageSummary>,
    finish_reason: Option<String>,
    llm_call_sink: &impl LlmCallSink,
) {
    let metadata = HashMap::new();

    llm_call_sink.record_llm_call(LlmCallTelemetry {
        context: context.to_string(),
        provider: llm.provider.clone(),
        model: llm.model.clone(),
        elapsed_ms: elapsed.as_millis() as u64,
        finish_reason,
        usage,
        metadata,
    });
}

fn format_hits_for_prompt(hits: &[SearchHit]) -> String {
    hits.iter()
        .enumerate()
        .map(|(idx, hit)| {
            format!(
                "[Source {idx}] ({}) {}\n  URL: {}\n  {}",
                match hit.provider {
                    SearchProvider::Brave => "brave",
                    SearchProvider::Tavily => "tavily",
                },
                hit.title,
                hit.url,
                hit.content
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn subject_label(request: &DueDiligenceRequest) -> String {
    match &request.product_name {
        Some(product) if !product.trim().is_empty() => {
            format!("{} (product: {})", request.company_name, product.trim())
        }
        _ => request.company_name.clone(),
    }
}

fn focus_area_label(focus_areas: &[String]) -> String {
    if focus_areas.is_empty() {
        "general software due diligence".to_string()
    } else {
        focus_areas.join(", ")
    }
}

fn encode_step_action(provider: SearchProvider, query: &str) -> String {
    let provider = match provider {
        SearchProvider::Brave => "brave",
        SearchProvider::Tavily => "tavily",
    };
    format!("[{provider}] {}", query.trim())
}

fn decode_step_action(action: &str) -> Option<(SearchProvider, String)> {
    let trimmed = action.trim();
    let (provider, rest) = if let Some(rest) = trimmed.strip_prefix("[brave]") {
        (SearchProvider::Brave, rest)
    } else if let Some(rest) = trimmed.strip_prefix("[tavily]") {
        (SearchProvider::Tavily, rest)
    } else {
        return None;
    };
    let query = rest.trim();
    if query.is_empty() {
        None
    } else {
        Some((provider, query.to_string()))
    }
}

fn is_relevant(hit: &SearchHit, company: &str) -> bool {
    let company_lower = company.to_ascii_lowercase();
    let title_lower = hit.title.to_ascii_lowercase();
    let content_lower = hit.content.to_ascii_lowercase();
    let url_lower = hit.url.to_ascii_lowercase();
    let compact = company_lower.replace(' ', "");
    let hyphenated = company_lower.replace(' ', "-");

    title_lower.contains(&company_lower)
        || content_lower.contains(&company_lower)
        || url_lower.contains(&compact)
        || url_lower.contains(&hyphenated)
}

fn dedup_hits(hits: &mut Vec<SearchHit>) {
    let mut seen = HashSet::new();
    hits.retain(|hit| seen.insert(hit.url.clone()));
}

fn strip_markdown_fences(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(start) = trimmed.find("```") {
        let after = &trimmed[start + 3..];
        if let Some(newline) = after.find('\n') {
            let body = &after[newline + 1..];
            if let Some(end) = body.rfind("```") {
                return body[..end].trim().to_string();
            }
            return body.trim().to_string();
        }
    }
    trimmed.to_string()
}

fn repair_truncated_json(value: &str) -> String {
    let mut result = value.to_string();
    let mut in_string = false;
    let mut escape = false;
    let mut stack = Vec::new();

    for ch in result.chars() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '{' => stack.push('}'),
            '[' => stack.push(']'),
            '}' | ']' => {
                stack.pop();
            }
            _ => {}
        }
    }

    if in_string {
        result.push('"');
    }
    while let Some(ch) = stack.pop() {
        result.push(ch);
    }
    result
}

fn strip_trailing_commas(value: &str) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(value.len());
    let mut in_string = false;
    let mut escape = false;
    let mut index = 0;

    while index < chars.len() {
        let ch = chars[index];
        if escape {
            output.push(ch);
            escape = false;
            index += 1;
            continue;
        }
        if ch == '\\' && in_string {
            output.push(ch);
            escape = true;
            index += 1;
            continue;
        }
        if ch == '"' {
            output.push(ch);
            in_string = !in_string;
            index += 1;
            continue;
        }
        if !in_string && ch == ',' {
            let mut lookahead = index + 1;
            while lookahead < chars.len() && chars[lookahead].is_whitespace() {
                lookahead += 1;
            }
            if lookahead < chars.len() && matches!(chars[lookahead], '}' | ']') {
                index += 1;
                continue;
            }
        }
        output.push(ch);
        index += 1;
    }

    output
}

fn load_env() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let _ = dotenv::dotenv();
    });
}

impl CliConfig {
    pub fn parse_from_env() -> anyhow::Result<Self> {
        load_env();

        let args = env::args().skip(1).collect::<Vec<_>>();
        let mut company_name = None;
        let mut product_name = None;
        let mut focus_areas = Vec::new();
        let mut provider_override = None;
        let mut model_override = None;
        let mut output_path = None;
        let mut deep_dives = 2usize;

        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--company" => {
                    index += 1;
                    company_name = Some(next_arg(&args, index, "--company")?);
                }
                "--product" => {
                    index += 1;
                    product_name = Some(next_arg(&args, index, "--product")?);
                }
                "--focus" => {
                    index += 1;
                    focus_areas = split_csv(&next_arg(&args, index, "--focus")?);
                }
                "--provider" => {
                    index += 1;
                    provider_override = Some(next_arg(&args, index, "--provider")?);
                }
                "--model" => {
                    index += 1;
                    model_override = Some(next_arg(&args, index, "--model")?);
                }
                "--output" => {
                    index += 1;
                    output_path = Some(PathBuf::from(next_arg(&args, index, "--output")?));
                }
                "--deep-dives" => {
                    index += 1;
                    deep_dives = next_arg(&args, index, "--deep-dives")?
                        .parse()
                        .context("--deep-dives must be a positive integer")?;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                value if !value.starts_with("--") && company_name.is_none() => {
                    company_name = Some(value.to_string());
                }
                other => bail!("unexpected argument: {other}"),
            }
            index += 1;
        }

        let company_name = company_name.ok_or_else(|| {
            anyhow!("missing company name. Pass it as the first positional arg or via --company")
        })?;

        Ok(Self {
            company_name,
            product_name,
            focus_areas,
            provider_override,
            model_override,
            output_path,
            deep_dives,
        })
    }
}

fn next_arg(args: &[String], index: usize, flag: &str) -> anyhow::Result<String> {
    args.get(index)
        .cloned()
        .ok_or_else(|| anyhow!("{flag} requires a value"))
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn print_help() {
    println!(
        "live_due_diligence [COMPANY] [--company NAME] [--product PRODUCT] [--focus a,b,c] [--provider anthropic] [--model MODEL] [--deep-dives 2] [--output report.json]"
    );
}

fn print_report(report: &MultiPassReport) {
    println!("=== Live Due Diligence ===");
    println!("Subject: {}", report.company_name);
    if let Some(product) = &report.product_name {
        println!("Product: {product}");
    }
    println!("LLM: {} ({})", report.llm_provider, report.llm_model);
    println!("Focus: {}", focus_area_label(&report.focus_areas));

    println!("\n--- Huddle Plan ---");
    for plan in &report.huddle_plans {
        println!("{}: {}", plan.reasoner, plan.rationale);
        for step in &plan.steps {
            println!("  - {:?}: {}", step.provider, step.query);
        }
    }

    println!("\n--- Pass 1 Summary ---");
    println!("{}", report.pass1.summary);

    println!("\n--- Key Facts ---");
    for fact in &report.pass1.key_facts {
        println!(
            "- [{}] ({:.0}%) {}",
            fact.category,
            fact.confidence * 100.0,
            fact.claim
        );
    }

    if !report.pass1.contradictions.is_empty() {
        println!("\n--- Contradictions ---");
        for contradiction in &report.pass1.contradictions {
            println!(
                "- {}: {} vs {}",
                contradiction.topic, contradiction.claim_a, contradiction.claim_b
            );
        }
    }

    if !report.deep_dives.is_empty() {
        println!("\n--- Deep Dives ---");
        for dive in &report.deep_dives {
            println!("Q: {}", dive.question);
            println!("{}\n", dive.synthesis);
        }
    }

    println!("\n--- Final Recommendation ---");
    println!("{}", report.final_report.recommendation);

    if !report.llm_calls.is_empty() {
        println!("\n--- LLM Telemetry ---");
        println!("Calls captured: {}", report.llm_calls.len());
        for (index, call) in report.llm_calls.iter().enumerate() {
            let tokens = call
                .usage
                .as_ref()
                .map(|usage| {
                    format!(
                        "p:{} c:{} t:{}",
                        usage.prompt_tokens.unwrap_or(0),
                        usage.completion_tokens.unwrap_or(0),
                        usage.total_tokens.unwrap_or(0),
                    )
                })
                .unwrap_or_else(|| "no usage".to_string());
            println!(
                "[{}] {} [{}] {}ms {}",
                index + 1,
                call.provider,
                call.context,
                call.elapsed_ms,
                tokens,
            );
            if let Some(finish_reason) = &call.finish_reason {
                println!("     finish_reason: {finish_reason}");
            }
            if !call.metadata.is_empty() {
                println!("     metadata_keys: {}", call.metadata.len());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_encoded_step_round_trips() {
        let action = encode_step_action(SearchProvider::Tavily, "acme investors revenue");
        let decoded = decode_step_action(&action).unwrap();
        assert_eq!(decoded.0, SearchProvider::Tavily);
        assert_eq!(decoded.1, "acme investors revenue");
    }

    #[test]
    fn repair_json_closes_open_structures() {
        let repaired = repair_truncated_json(r#"{"a":[1,2"#);
        assert_eq!(repaired, r#"{"a":[1,2]}"#);
    }

    #[test]
    fn strip_trailing_commas_before_closing_tokens() {
        let cleaned = strip_trailing_commas("{\"a\": [1,2,],}");
        assert_eq!(cleaned, "{\"a\": [1,2]}");
    }
}
