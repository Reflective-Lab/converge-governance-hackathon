use std::env;
use std::sync::OnceLock;

use converge_provider::provider_api::CostClass;
use converge_provider::KongRoute;

#[derive(Debug, Clone)]
pub struct EditorConfig {
    kong_gateway_configured: bool,
    kong_llm_route: String,
    kong_llm_upstream_provider: String,
    kong_llm_upstream_model: String,
    kong_llm_reasoning: bool,
}

static CONFIG: OnceLock<EditorConfig> = OnceLock::new();

pub fn editor_config() -> &'static EditorConfig {
    CONFIG.get_or_init(EditorConfig::load)
}

impl EditorConfig {
    fn load() -> Self {
        dotenv::dotenv().ok();

        Self {
            kong_gateway_configured: read_env("KONG_AI_GATEWAY_URL").is_some()
                && read_env("KONG_API_KEY").is_some(),
            kong_llm_route: read_env("KONG_LLM_ROUTE").unwrap_or_else(|| "default".to_string()),
            kong_llm_upstream_provider: read_env("KONG_LLM_UPSTREAM_PROVIDER")
                .unwrap_or_else(|| "openai".to_string()),
            kong_llm_upstream_model: read_env("KONG_LLM_UPSTREAM_MODEL")
                .unwrap_or_else(|| "gpt-4".to_string()),
            kong_llm_reasoning: read_bool("KONG_LLM_REASONING").unwrap_or(true),
        }
    }

    pub fn kong_gateway_configured(&self) -> bool {
        self.kong_gateway_configured
    }

    pub fn kong_heading_route(&self) -> KongRoute {
        KongRoute::new(self.kong_llm_route.clone())
            .upstream(
                self.kong_llm_upstream_provider.clone(),
                self.kong_llm_upstream_model.clone(),
            )
            .cost(CostClass::Medium)
            .reasoning(self.kong_llm_reasoning)
    }

    pub fn kong_heading_route_name(&self) -> &str {
        &self.kong_llm_route
    }
}

fn read_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_bool(key: &str) -> Option<bool> {
    match read_env(key)?.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
