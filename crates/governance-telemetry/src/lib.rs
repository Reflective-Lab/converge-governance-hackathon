use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsageSummary {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCallTelemetry {
    pub context: String,
    pub provider: String,
    pub model: String,
    pub elapsed_ms: u64,
    pub finish_reason: Option<String>,
    pub usage: Option<LlmUsageSummary>,
    pub metadata: HashMap<String, String>,
}

pub trait LlmCallSink: Send + Sync {
    fn record_llm_call(&self, call: LlmCallTelemetry);
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryLlmCallCollector {
    calls: Arc<Mutex<Vec<LlmCallTelemetry>>>,
}

impl LlmCallSink for InMemoryLlmCallCollector {
    fn record_llm_call(&self, call: LlmCallTelemetry) {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(call);
        }
    }
}

impl InMemoryLlmCallCollector {
    pub fn snapshot(&self) -> Vec<LlmCallTelemetry> {
        self.calls
            .lock()
            .map(|calls| calls.clone())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut calls) = self.calls.lock() {
            calls.clear();
        }
    }
}

#[derive(Debug, Default)]
pub struct NoopLlmCallSink;

impl LlmCallSink for NoopLlmCallSink {
    fn record_llm_call(&self, _call: LlmCallTelemetry) {}
}
