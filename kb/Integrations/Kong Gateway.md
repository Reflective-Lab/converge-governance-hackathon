---
tags: [integrations, kong]
---
# Kong AI Gateway

The only intended remote integration layer. All outbound calls from the Rust core go through Kong.

## What Kong Handles

- **Rate limiting** per team
- **PII detection** on prompts (redacts sensitive data)
- **Token usage logging**
- **Cost tracking** per request
- **Latency monitoring**
- **Centralized governance** for all external model and service access
- [[Integrations/MCP Tools|MCP tool access]]
- Proxied business APIs

## Setup

Your team will receive a Kong AI Gateway URL, an API key, and access to the Kong dashboard to see your API usage.

Set in `.env`:

```dotenv
KONG_AI_GATEWAY_URL=https://<provided-at-hackathon>
KONG_API_KEY=<your-team-key>

# Optional route settings
KONG_LLM_ROUTE=default
KONG_LLM_UPSTREAM_PROVIDER=openai
KONG_LLM_UPSTREAM_MODEL=gpt-4
KONG_LLM_REASONING=true
```

## Making LLM Calls

Use `converge-provider` as the single entry point:

```rust
use converge_provider::{KongGateway, KongRoute, LlmProvider, LlmRequest};
use converge_provider::provider_api::CostClass;

let gateway = KongGateway::from_env()?;

let llm = gateway.llm_provider(
    KongRoute::new("default")
        .upstream("openai", "gpt-4")
        .cost(CostClass::Medium)
        .reasoning(true)
);

let response = llm.complete(&LlmRequest::new(
    "Analyze this vendor for GDPR compliance"
))?;
```

All LLM traffic goes through Kong instead of direct provider APIs.

## Non-LLM APIs

```rust
let gateway = KongGateway::from_env()?;
let policy_url = gateway.api_url("compliance/v1/policies");
let (header_name, header_value) = gateway.auth_header();
```

All external API access is logged and governed through one gateway object.

## Desktop App Pattern

1. Tauri config loads `.env`
2. Checks for `KONG_AI_GATEWAY_URL` and `KONG_API_KEY`
3. Builds `KongRoute` from local settings
4. Creates `KongGateway::from_env()`
5. Uses `gateway.llm_provider(route)` for LLM access
6. Falls back to local heuristic if Kong not configured

## Intended Runtime Shape

1. Svelte and Tauri run as the local shell
2. The shell calls the Rust app layer locally
3. Rust agents run inside the Converge runtime
4. LLM-backed agents call models through Kong
5. Service-backed agents use Kong-routed APIs or Kong-exposed [[Integrations/MCP Tools|MCP tools]]
6. Missing enterprise systems are mocked locally and still accessed through Kong-facing adapters

Converge governs decision-making inside the app. Kong governs the only remote AI and business access outside the app.

See also: [[Integrations/MCP Tools]], [[Integrations/External Services]], [[Domain/Agents]]
