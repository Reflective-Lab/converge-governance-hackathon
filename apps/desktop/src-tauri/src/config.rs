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

        let converge_provider = read_env("CONVERGE_LLM_FORCE_PROVIDER");
        let kong_provider = read_env("KONG_LLM_UPSTREAM_PROVIDER");
        let converge_model = read_env("CONVERGE_LLM_MODEL");
        let kong_model = read_env("KONG_LLM_UPSTREAM_MODEL");
        let openrouter_model = read_env("OPENROUTER_MODEL");

        Self {
            heading_provider_override: resolve_heading_provider_override(
                converge_provider.clone(),
                kong_provider.clone(),
                converge_model.as_deref(),
                kong_model.as_deref(),
                openrouter_model.as_deref(),
            ),
            heading_model_override: resolve_heading_model_override(
                converge_model,
                kong_model,
                openrouter_model,
            ),
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

fn resolve_heading_provider_override(
    converge_provider: Option<String>,
    kong_provider: Option<String>,
    converge_model: Option<&str>,
    kong_model: Option<&str>,
    openrouter_model: Option<&str>,
) -> Option<String> {
    converge_provider
        .or(kong_provider)
        .or_else(|| infer_provider_from_model(converge_model))
        .or_else(|| infer_provider_from_model(kong_model))
        .or_else(|| openrouter_model.map(|_| "openrouter".to_string()))
}

fn resolve_heading_model_override(
    converge_model: Option<String>,
    kong_model: Option<String>,
    openrouter_model: Option<String>,
) -> Option<String> {
    converge_model.or(kong_model).or(openrouter_model)
}

fn infer_provider_from_model(model: Option<&str>) -> Option<String> {
    let model = model?;
    if looks_like_openrouter_model(model) {
        Some("openrouter".to_string())
    } else {
        None
    }
}

fn looks_like_openrouter_model(model: &str) -> bool {
    model.contains('/')
}

#[cfg(test)]
mod tests {
    use super::{
        looks_like_openrouter_model, resolve_heading_model_override,
        resolve_heading_provider_override,
    };

    #[test]
    fn openrouter_model_is_used_as_heading_model_override() {
        assert_eq!(
            resolve_heading_model_override(
                None,
                None,
                Some("anthropic/claude-sonnet-4-20250514".to_string()),
            )
            .as_deref(),
            Some("anthropic/claude-sonnet-4-20250514")
        );
    }

    #[test]
    fn openrouter_model_implies_openrouter_provider_override() {
        assert_eq!(
            resolve_heading_provider_override(
                None,
                None,
                None,
                None,
                Some("anthropic/claude-sonnet-4-20250514"),
            )
            .as_deref(),
            Some("openrouter")
        );
    }

    #[test]
    fn explicit_provider_override_beats_inferred_openrouter_provider() {
        assert_eq!(
            resolve_heading_provider_override(
                Some("openai".to_string()),
                None,
                Some("anthropic/claude-sonnet-4-20250514"),
                None,
                None,
            )
            .as_deref(),
            Some("openai")
        );
    }

    #[test]
    fn slash_model_format_is_recognized_as_openrouter_style() {
        assert!(looks_like_openrouter_model(
            "anthropic/claude-sonnet-4-20250514"
        ));
        assert!(!looks_like_openrouter_model("gpt-4o-mini"));
    }
}
