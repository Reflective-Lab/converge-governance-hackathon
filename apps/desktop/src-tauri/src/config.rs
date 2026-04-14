use std::env;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct EditorConfig {
    heading_provider_override: Option<String>,
    heading_model_override: Option<String>,
}

static CONFIG: OnceLock<EditorConfig> = OnceLock::new();

pub fn editor_config() -> &'static EditorConfig {
    CONFIG.get_or_init(EditorConfig::load)
}

impl EditorConfig {
    fn load() -> Self {
        dotenv::dotenv().ok();

        Self {
            heading_provider_override: read_env("CONVERGE_LLM_FORCE_PROVIDER")
                .or_else(|| read_env("KONG_LLM_UPSTREAM_PROVIDER")),
            heading_model_override: read_env("CONVERGE_LLM_MODEL")
                .or_else(|| read_env("KONG_LLM_UPSTREAM_MODEL")),
        }
    }

    pub fn heading_provider_override(&self) -> Option<&str> {
        self.heading_provider_override.as_deref()
    }

    pub fn heading_model_override(&self) -> Option<&str> {
        self.heading_model_override.as_deref()
    }
}

fn read_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
