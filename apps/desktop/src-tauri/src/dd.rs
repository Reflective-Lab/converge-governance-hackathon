//! Self-contained due diligence — searches the web, extracts facts, synthesizes.
//! Copied from Monterro's DD pipeline, simplified for the hackathon demo.

use std::time::Instant;

use converge_provider::{
    ChatBackendSelectionConfig, JsonChatResponse, chat_json_lenient, select_healthy_chat_backend,
};
use converge_provider_api::{ChatMessage, ChatRequest, ChatRole, ResponseFormat};
use governance_telemetry::{
    InMemoryLlmCallCollector, LlmCallSink, LlmCallTelemetry, LlmUsageSummary,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DdReport {
    pub company_name: String,
    pub product_name: Option<String>,
    pub pass1: Pass1,
    pub final_report: FinalReport,
    pub pass1_hits: Vec<SearchHit>,
    pub llm_calls: Vec<LlmCallTelemetry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pass1 {
    pub summary: String,
    pub key_facts: Vec<TaggedFact>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinalReport {
    pub market_analysis: String,
    pub competitive_landscape: String,
    pub technology_assessment: String,
    pub risk_factors: Vec<String>,
    pub growth_opportunities: Vec<String>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaggedFact {
    pub claim: String,
    pub category: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub title: String,
    pub url: String,
}

pub async fn run_dd(company: &str, product: Option<&str>) -> anyhow::Result<DdReport> {
    let subject = match product {
        Some(p) => format!("{company} ({p})"),
        None => company.to_string(),
    };

    let llm_call_collector = InMemoryLlmCallCollector::default();

    // 1. Search via Brave
    let hits = search_brave(&subject).await.unwrap_or_default();

    // 2. Build context from search results
    let search_context: String = hits
        .iter()
        .take(15)
        .enumerate()
        .map(|(i, h)| format!("[{}] {} — {}", i + 1, h.title, h.url))
        .collect::<Vec<_>>()
        .join("\n");

    // 3. Call LLM for analysis
    let prompt = format!(
        r#"You are a due diligence analyst. Analyze this company based on the search results below.

Company: {company}

Search Results:
{search_context}

Produce a JSON response with this exact structure:
{{
  "summary": "2-3 paragraph executive summary",
  "key_facts": [
    {{"claim": "fact text", "category": "market|technology|customers|financials|competition|product", "confidence": 0.9}}
  ],
  "market_analysis": "market analysis paragraph",
  "competitive_landscape": "competitive analysis paragraph",
  "technology_assessment": "technology assessment paragraph",
  "risk_factors": ["risk 1", "risk 2"],
  "growth_opportunities": ["opportunity 1", "opportunity 2"],
  "recommendation": "investment recommendation paragraph"
}}"#
    );

    let v = call_llm_json(&prompt, "desktop-dd:analysis", &llm_call_collector)
        .await
        .unwrap_or_else(|error| fallback_analysis_json(&subject, &hits, &error));

    let key_facts: Vec<TaggedFact> = v["key_facts"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    Some(TaggedFact {
                        claim: f["claim"].as_str()?.to_string(),
                        category: f["category"].as_str().unwrap_or("other").to_string(),
                        confidence: f["confidence"].as_f64().unwrap_or(0.7),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let risk_factors: Vec<String> = v["risk_factors"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let growth_opportunities: Vec<String> = v["growth_opportunities"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    Ok(DdReport {
        company_name: company.to_string(),
        product_name: product.map(String::from),
        pass1: Pass1 {
            summary: v["summary"].as_str().unwrap_or("").to_string(),
            key_facts,
        },
        final_report: FinalReport {
            market_analysis: v["market_analysis"].as_str().unwrap_or("").to_string(),
            competitive_landscape: v["competitive_landscape"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            technology_assessment: v["technology_assessment"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            risk_factors,
            growth_opportunities,
            recommendation: v["recommendation"].as_str().unwrap_or("").to_string(),
        },
        pass1_hits: hits.into_iter().take(15).collect(),
        llm_calls: llm_call_collector.snapshot(),
    })
}

async fn search_brave(query: &str) -> anyhow::Result<Vec<SearchHit>> {
    let api_key =
        std::env::var("BRAVE_API_KEY").map_err(|_| anyhow::anyhow!("BRAVE_API_KEY not set"))?;

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.search.brave.com/res/v1/web/search")
        .header("X-Subscription-Token", &api_key)
        .header("Accept", "application/json")
        .query(&[
            ("q", &format!("{query} company product overview")),
            ("count", &"15".to_string()),
        ])
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let hits = resp["web"]["results"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|r| {
                    Some(SearchHit {
                        title: r["title"].as_str()?.to_string(),
                        url: r["url"].as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(hits)
}

async fn call_llm_json(
    prompt: &str,
    context: &str,
    llm_call_sink: &impl LlmCallSink,
) -> anyhow::Result<serde_json::Value> {
    let started_at = Instant::now();
    let mut config = ChatBackendSelectionConfig::from_env().unwrap_or_default();
    if config.provider_override.is_none() && std::env::var_os("OPENROUTER_API_KEY").is_some() {
        config = config.with_provider_override("openrouter");
    }

    let selected = select_healthy_chat_backend(&config)
        .await
        .map_err(|e| anyhow::anyhow!("No LLM backend available: {e}"))?;

    let JsonChatResponse { value, response } = chat_json_lenient::<serde_json::Value>(
        selected.backend.as_ref(),
        ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: prompt.to_string(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: Some(
                "You are a rigorous due diligence analyst. Respond with JSON only.".to_string(),
            ),
            tools: Vec::new(),
            response_format: ResponseFormat::Json,
            max_tokens: Some(4096),
            temperature: Some(0.0),
            stop_sequences: Vec::new(),
            model: None,
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!("LLM error: {e}"))?;

    llm_call_sink.record_llm_call(LlmCallTelemetry {
        context: context.to_string(),
        provider: selected.provider().to_string(),
        model: selected.model().to_string(),
        elapsed_ms: started_at.elapsed().as_millis() as u64,
        finish_reason: response
            .finish_reason
            .as_ref()
            .map(|reason| format!("{reason:?}")),
        usage: response.usage.as_ref().map(|usage| LlmUsageSummary {
            prompt_tokens: Some(u64::from(usage.prompt_tokens)),
            completion_tokens: Some(u64::from(usage.completion_tokens)),
            total_tokens: Some(u64::from(usage.total_tokens)),
        }),
        metadata: Default::default(),
    });

    Ok(value)
}

fn fallback_analysis_json(
    subject: &str,
    hits: &[SearchHit],
    error: &anyhow::Error,
) -> serde_json::Value {
    let top_hits = hits
        .iter()
        .take(5)
        .map(|hit| format!("{} ({})", hit.title, hit.url))
        .collect::<Vec<_>>();
    let evidence = if top_hits.is_empty() {
        "No Brave search results were available for this run.".to_string()
    } else {
        format!("Available evidence: {}", top_hits.join("; "))
    };

    serde_json::json!({
        "summary": format!(
            "{subject} due diligence continued with a search-grounded fallback because the LLM analysis response could not be recovered. {evidence}"
        ),
        "key_facts": hits.iter().take(6).map(|hit| {
            serde_json::json!({
                "claim": format!("Search result found: {} ({})", hit.title, hit.url),
                "category": "source",
                "confidence": 0.45
            })
        }).collect::<Vec<_>>(),
        "market_analysis": "Market analysis requires analyst review because the structured LLM response failed after provider-level JSON repair.",
        "competitive_landscape": "Competitive landscape is deferred to the sourced search results until a successful deep analysis pass is available.",
        "technology_assessment": "Technology assessment is not asserted by the fallback path; treat this as an evidence collection checkpoint.",
        "risk_factors": [
            format!("LLM structured analysis failed: {error}"),
            "Report is based on search-result metadata only; claims need source review before promotion."
        ],
        "growth_opportunities": [
            "Retry the analysis with a model ranked higher for short structured synthesis.",
            "Use deep search on the strongest source URLs before final promotion."
        ],
        "recommendation": "Do not promote this due diligence report as final. Use it as a recoverable checkpoint and rerun the governed analysis with provider-level JSON repair enabled."
    })
}
