use anyhow::{Context as AnyhowContext, anyhow};
use converge_provider::{
    BraveSearchProvider, SearchDepth, TavilySearchProvider, WebSearchBackend, WebSearchRequest,
};
use serde::{Deserialize, Serialize};

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

pub async fn run_search(provider: SearchProvider, query: &str) -> anyhow::Result<Vec<SearchHit>> {
    match provider {
        SearchProvider::Brave => search_brave(query).await,
        SearchProvider::Tavily => search_tavily(query).await,
    }
}

pub async fn search_brave(query: &str) -> anyhow::Result<Vec<SearchHit>> {
    let query = query.to_string();
    tokio::task::spawn_blocking(move || {
        let now = chrono::Utc::now();
        let response = BraveSearchProvider::from_env()
            .context("BRAVE_API_KEY is not configured")?
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

pub async fn search_tavily(query: &str) -> anyhow::Result<Vec<SearchHit>> {
    let query = query.to_string();
    tokio::task::spawn_blocking(move || {
        let now = chrono::Utc::now();
        let response = TavilySearchProvider::from_env()
            .context("TAVILY_API_KEY is not configured")?
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

pub fn format_hits_for_prompt(hits: &[SearchHit]) -> String {
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
