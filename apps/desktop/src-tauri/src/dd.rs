//! Self-contained due diligence — searches the web, extracts facts, synthesizes.
//! Copied from Monterro's DD pipeline, simplified for the hackathon demo.

use std::time::Instant;

use governance_telemetry::{InMemoryLlmCallCollector, LlmCallSink, LlmCallTelemetry, LlmUsageSummary};
use converge_provider_api::{ChatMessage, ChatRequest, ChatRole, ResponseFormat};
use converge_provider::{ChatBackendSelectionConfig, select_healthy_chat_backend};
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

    let raw = call_llm(&prompt, "desktop-dd:analysis", &llm_call_collector).await?;
    let v: serde_json::Value = serde_json::from_str(&strip_fences(&raw))
        .unwrap_or_else(|_| serde_json::json!({}));

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
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let growth_opportunities: Vec<String> = v["growth_opportunities"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
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
            competitive_landscape: v["competitive_landscape"].as_str().unwrap_or("").to_string(),
            technology_assessment: v["technology_assessment"].as_str().unwrap_or("").to_string(),
            risk_factors,
            growth_opportunities,
            recommendation: v["recommendation"].as_str().unwrap_or("").to_string(),
        },
        pass1_hits: hits.into_iter().take(15).collect(),
        llm_calls: llm_call_collector.snapshot(),
    })
}

async fn search_brave(query: &str) -> anyhow::Result<Vec<SearchHit>> {
    let api_key = std::env::var("BRAVE_API_KEY")
        .map_err(|_| anyhow::anyhow!("BRAVE_API_KEY not set"))?;

    let client = reqwest::Client::new();
    let resp = client
        .get("https://api.search.brave.com/res/v1/web/search")
        .header("X-Subscription-Token", &api_key)
        .header("Accept", "application/json")
        .query(&[("q", &format!("{query} company product overview")), ("count", &"15".to_string())])
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

async fn call_llm(
    prompt: &str,
    context: &str,
    llm_call_sink: &impl LlmCallSink,
) -> anyhow::Result<String> {
    let started_at = Instant::now();
    let mut config = ChatBackendSelectionConfig::from_env().unwrap_or_default();
    if config.provider_override.is_none() && std::env::var_os("OPENROUTER_API_KEY").is_some() {
        config = config.with_provider_override("openrouter");
    }

    let selected = select_healthy_chat_backend(&config)
        .await
        .map_err(|e| anyhow::anyhow!("No LLM backend available: {e}"))?;

    let response = selected
        .backend
        .chat(ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: prompt.to_string(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: None,
            tools: Vec::new(),
            response_format: ResponseFormat::Json,
            max_tokens: Some(4096),
            temperature: Some(0.0),
            stop_sequences: Vec::new(),
            model: None,
        })
        .await
        .map_err(|e| anyhow::anyhow!("LLM error: {e}"))?;

    llm_call_sink.record_llm_call(LlmCallTelemetry {
        context: context.to_string(),
        provider: selected.provider().to_string(),
        model: selected.model().to_string(),
        elapsed_ms: started_at.elapsed().as_millis() as u64,
        finish_reason: response.finish_reason.as_ref().map(|reason| format!("{reason:?}")),
        usage: response.usage.as_ref().map(|usage| LlmUsageSummary {
            prompt_tokens: Some(u64::from(usage.prompt_tokens)),
            completion_tokens: Some(u64::from(usage.completion_tokens)),
            total_tokens: Some(u64::from(usage.total_tokens)),
        }),
        metadata: Default::default(),
    });

    Ok(response.content)
}

fn strip_fences(s: &str) -> String {
    let trimmed = s.trim();
    if let Some(start) = trimmed.find("```") {
        let after = &trimmed[start + 3..];
        if let Some(nl) = after.find('\n') {
            let body = &after[nl + 1..];
            if let Some(end) = body.rfind("```") {
                return body[..end].trim().to_string();
            }
            return body.trim().to_string();
        }
    }
    trimmed.to_string()
}
