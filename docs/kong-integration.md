# Kong AI Gateway Integration

All LLM API calls should be routed through Kong AI Gateway. This gives you rate limiting, PII detection, cost tracking, and prompt logging — enterprise governance for AI out of the box.

## Setup

Your team will receive:
- Kong AI Gateway URL (provided at hackathon)
- API key for your team
- Access to the Kong dashboard to see your API usage

## Making LLM Calls Through Kong

Instead of calling OpenAI/Anthropic directly:

```
// Direct (DON'T do this)
POST https://api.openai.com/v1/chat/completions

// Through Kong (DO this)
POST https://<kong-gateway>/ai/chat/completions
Header: x-api-key: <your-team-key>
```

Kong proxies the request to the configured LLM provider with:
- Rate limiting per team
- PII detection on prompts (redacts sensitive data)
- Token usage logging
- Cost tracking per request
- Latency monitoring

## Example: LLM Agent Making a Kong-Routed Call

```rust
use reqwest::Client;
use serde_json::json;

async fn call_llm_via_kong(prompt: &str, kong_url: &str, api_key: &str) -> Result<String, String> {
    let client = Client::new();
    let response = client
        .post(format!("{kong_url}/ai/chat/completions"))
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.3
        }))
        .send()
        .await
        .map_err(|e| format!("kong request failed: {e}"))?;

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("kong response parse failed: {e}"))?;

    body["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "no content in response".into())
}
```

## Using Kong for Non-LLM APIs

Kong can also gateway your vendor documentation APIs, compliance databases, etc:

```
GET https://<kong-gateway>/vendors/<vendor-id>/compliance-docs
Header: x-api-key: <your-team-key>
```

This means all external API access is logged and governed through one gateway.

## MCP Server Integration

Kong exposes MCP (Model Context Protocol) endpoints that Claude and other AI tools can use. If your agents use MCP tools, route them through Kong for the same governance benefits.
