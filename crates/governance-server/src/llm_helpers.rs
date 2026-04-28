use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Instant;

use anyhow::anyhow;
use converge_provider::{
    AnthropicBackend, ChatBackendSelectionConfig, GeminiBackend, MistralBackend, OpenAiBackend,
    OpenRouterBackend, select_healthy_chat_backend,
};
use converge_provider_api::{
    ChatMessage, ChatRequest, ChatRole, DynChatBackend, LlmError, ResponseFormat,
    SelectionCriteria,
};
use governance_telemetry::{LlmCallSink, LlmCallTelemetry, LlmUsageSummary};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct SelectedLlm {
    pub provider: String,
    pub model: String,
    pub model_override: Option<String>,
    pub backend: Arc<dyn DynChatBackend>,
}

pub async fn select_llm(
    provider_override: Option<&str>,
    model_override: Option<&str>,
) -> anyhow::Result<SelectedLlm> {
    if provider_override.is_none()
        && let Some(model) = model_override
        && let Some(provider) = infer_direct_provider(model)
    {
        return select_direct_model(provider, model).await;
    }

    if provider_override.is_none()
        && model_override.is_some_and(|model| model.contains('/'))
        && std::env::var_os("OPENROUTER_API_KEY").is_some()
    {
        return select_openrouter_model(model_override.unwrap()).await;
    }

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

pub async fn select_llm_for_model(model: &str) -> anyhow::Result<SelectedLlm> {
    select_llm(None, Some(model)).await
}

async fn select_openrouter_model(model: &str) -> anyhow::Result<SelectedLlm> {
    let backend: Arc<dyn DynChatBackend> = Arc::new(
        OpenRouterBackend::from_env()
            .map_err(|error| anyhow!("failed to configure OpenRouter: {error}"))?
            .with_model(model),
    );
    probe_model(&backend, model)
        .await
        .map_err(|error| anyhow!("OpenRouter model probe failed for {model}: {error}"))?;

    Ok(SelectedLlm {
        provider: "openrouter".to_string(),
        model: model.to_string(),
        model_override: Some(model.to_string()),
        backend,
    })
}

async fn select_direct_model(provider: &str, model: &str) -> anyhow::Result<SelectedLlm> {
    let backend: Arc<dyn DynChatBackend> = match provider {
        "openai" => Arc::new(
            OpenAiBackend::from_env()
                .map_err(|error| anyhow!("failed to configure OpenAI: {error}"))?
                .with_model(model),
        ),
        "anthropic" => Arc::new(
            AnthropicBackend::from_env()
                .map_err(|error| anyhow!("failed to configure Anthropic: {error}"))?
                .with_model(model),
        ),
        "gemini" => Arc::new(
            GeminiBackend::from_env()
                .map_err(|error| anyhow!("failed to configure Gemini: {error}"))?
                .with_model(model),
        ),
        "mistral" => Arc::new(
            MistralBackend::from_env()
                .map_err(|error| anyhow!("failed to configure Mistral: {error}"))?
                .with_model(model),
        ),
        _ => return Err(anyhow!("unsupported direct provider for model {model}: {provider}")),
    };
    probe_model(&backend, model)
        .await
        .map_err(|error| anyhow!("{provider} model probe failed for {model}: {error}"))?;

    Ok(SelectedLlm {
        provider: provider.to_string(),
        model: model.to_string(),
        model_override: Some(model.to_string()),
        backend,
    })
}

fn infer_direct_provider(model: &str) -> Option<&'static str> {
    if model.starts_with("gpt-") || model.starts_with("o1") || model.starts_with("o3") || model.starts_with("o4") {
        return std::env::var_os("OPENAI_API_KEY").map(|_| "openai");
    }
    if model.starts_with("claude-") {
        return std::env::var_os("ANTHROPIC_API_KEY").map(|_| "anthropic");
    }
    if model.starts_with("gemini-") {
        return std::env::var_os("GEMINI_API_KEY").map(|_| "gemini");
    }
    if model.starts_with("mistral-")
        || model.starts_with("ministral-")
        || model.starts_with("open-mistral")
    {
        return std::env::var_os("MISTRAL_API_KEY").map(|_| "mistral");
    }
    None
}

async fn probe_model(backend: &Arc<dyn DynChatBackend>, model: &str) -> Result<(), LlmError> {
    backend
        .chat(ChatRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "Return ok.".to_string(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            }],
            system: None,
            tools: Vec::new(),
            response_format: ResponseFormat::Text,
            max_tokens: Some(2),
            temperature: Some(0.0),
            stop_sequences: Vec::new(),
            model: Some(model.to_string()),
        })
        .await
        .map(|_| ())
}

pub async fn call_llm_text(
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

    let text = response.content.trim().to_string();
    if text.is_empty() {
        return Err(anyhow!(
            "LLM returned empty response (model={}, finish={:?})",
            llm.model_override.as_deref().unwrap_or(&llm.model),
            response.finish_reason,
        ));
    }
    Ok(text)
}

pub async fn call_llm_json<T: for<'de> Deserialize<'de>>(
    llm: &SelectedLlm,
    system: &str,
    prompt: &str,
    max_tokens: u32,
    context: &str,
    llm_call_sink: &impl LlmCallSink,
) -> anyhow::Result<T> {
    let started_at = Instant::now();
    // Use ResponseFormat::Text so truncated JSON reaches our repair logic
    // instead of being rejected at the provider transport layer.
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

    let raw = strip_markdown_fences(&response.content);
    if raw.trim().is_empty() {
        return Err(anyhow!(
            "LLM returned empty response (model={}, finish={:?})",
            llm.model_override.as_deref().unwrap_or(&llm.model),
            response.finish_reason,
        ));
    }
    parse_json_response(llm, &raw, max_tokens, context, llm_call_sink).await
}

pub async fn parse_json_response<T: for<'de> Deserialize<'de>>(
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

pub fn push_llm_call_telemetry(
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
        model: llm
            .model_override
            .clone()
            .unwrap_or_else(|| llm.model.clone()),
        elapsed_ms: elapsed.as_millis() as u64,
        finish_reason,
        usage,
        metadata,
    });
}

pub fn strip_markdown_fences(value: &str) -> String {
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

pub fn repair_truncated_json(value: &str) -> String {
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

pub fn strip_trailing_commas(value: &str) -> String {
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

pub fn load_env() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let _ = dotenv::dotenv();
    });
}
