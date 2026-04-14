use std::collections::HashMap;

use converge_kernel::{ContextKey, ConvergeResult};
use serde::de::DeserializeOwned;

pub fn required_input<'a>(
    inputs: &'a HashMap<String, String>,
    key: &str,
) -> Result<&'a str, String> {
    inputs
        .get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("missing required input: {key}"))
}

pub fn optional_input(inputs: &HashMap<String, String>, key: &str) -> Option<String> {
    inputs
        .get(key)
        .map(String::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToString::to_string)
}

pub fn converge_confidence_to_bps(confidence: f64) -> u16 {
    (confidence.clamp(0.0, 1.0) * 10_000.0).round() as u16
}

pub fn payload_from_result<T: DeserializeOwned>(
    result: &ConvergeResult,
    key: ContextKey,
    fact_id: &str,
) -> Result<T, String> {
    let fact = result
        .context
        .get(key)
        .iter()
        .find(|f| f.id == fact_id)
        .ok_or_else(|| format!("missing fact: {fact_id}"))?;
    serde_json::from_str(&fact.content).map_err(|e| format!("invalid {fact_id} payload: {e}"))
}
