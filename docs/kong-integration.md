# Kong AI Gateway Integration

This repo assumes a self-contained desktop app. There is no default remote governance backend between the UI and the Rust core. The only intended remote calls are outbound calls from the Rust core to Kong and the services Kong fronts.

Use Kong for:

- LLM traffic
- MCP tool access
- proxied business APIs

That gives teams one place for traffic control, observability, policy enforcement, and access governance.

The exact calling shape for Kong-hosted services may be defined by the Kong team during the event. This repo should treat those external calls as adapters at the edge of the Rust application, not as the center of the app architecture.

## Setup

Your team will receive:
- Kong AI Gateway URL (provided at hackathon)
- API key for your team
- Access to the Kong dashboard to see your API usage

Set those in `.env`:

```dotenv
KONG_AI_GATEWAY_URL=https://<provided-at-hackathon>
KONG_API_KEY=<your-team-key>
```

For this repo, the desktop app may also read optional route settings:

```dotenv
KONG_LLM_ROUTE=default
KONG_LLM_UPSTREAM_PROVIDER=openai
KONG_LLM_UPSTREAM_MODEL=gpt-4
KONG_LLM_REASONING=true
```

## Making LLM Calls Through Kong

Prefer `converge-provider` as the single Kong entry point. That keeps the desktop app, agents, and future tool integrations on one consistent adapter.

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

This is the default pattern for the hackathon:

1. `KongGateway::from_env()` reads `KONG_AI_GATEWAY_URL` and `KONG_API_KEY`.
2. The application defines the route shape with `KongRoute`.
3. All outbound LLM, MCP, and REST access flows from that gateway object.

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
- Centralized governance for external model access

## Example: Agent Using the Gateway

```rust
use converge_provider::kong::{KongGateway, KongRoute};
use converge_provider::provider_api::{CostClass, LlmProvider, LlmRequest};

struct ComplianceScreenerAgent {
    kong: KongGateway,
}

impl ComplianceScreenerAgent {
    fn screen(&self, vendor_data: &str) -> Result<String, String> {
        let llm = self.kong.llm_provider(
            KongRoute::new("default")
                .upstream("openai", "gpt-4")
                .cost(CostClass::Medium)
                .reasoning(true)
        );

        let prompt = format!("Check this vendor against GDPR: {vendor_data}");
        let response = llm
            .complete(&LlmRequest::new(prompt))
            .map_err(|error| format!("kong llm request failed: {error}"))?;

        Ok(response.content)
    }
}
```

## Using Kong for Non-LLM APIs

Kong can also gateway your vendor documentation APIs, compliance databases, and policy services:

```rust
use converge_provider::KongGateway;

let gateway = KongGateway::from_env()?;
let policy_url = gateway.api_url("compliance/v1/policies");
let (header_name, header_value) = gateway.auth_header();
```

This means all external API access is logged and governed through one gateway object.

## MCP Integration

Kong can also expose MCP (Model Context Protocol) tooling so agentic systems can access business services through tool calls instead of raw HTTP integrations.

Typical examples:

- vendor registry lookups
- procurement policy checks
- internal compliance evidence retrieval
- approval workflow actions

If your architecture uses MCP, the preferred pattern is still the same: go through Kong first so the tool access inherits the same governance model as LLM traffic.

```rust
use converge_provider::{KongGateway, McpClient, McpTransport};

let gateway = KongGateway::from_env()?;
let mcp = McpClient::new(
    "vendor-registry",
    McpTransport::Http {
        url: gateway.mcp_url("vendor-registry"),
    },
);
```

## Mocked Services Through Kong

During a hackathon, real enterprise services are often unavailable. Do not collapse the architecture just because the real backends are missing.

Instead:

1. Build a small local mock service in Rust.
2. Expose it with a stable API or MCP surface.
3. Register or proxy it through Kong.
4. Have agents call the mocked service through Kong, exactly as they would in the real architecture.

This keeps the demo honest. Teams still show the intended integration pattern without pretending that hardcoded agent logic is the same thing as governed service access.

Good candidates for mocks:

- policy service
- vendor dossier service
- pricing catalog
- procurement approval rules
- compliance evidence store

## Recommended Pattern For This Repo

The intended runtime shape is:

1. Svelte and Tauri run as the local shell.
2. The shell calls the Rust app layer locally.
3. Rust agents run inside the Converge-based application.
4. LLM-backed agents call models through Kong.
5. Service-backed agents use Kong-routed APIs or Kong-exposed MCP tools.
6. Missing enterprise systems are mocked locally and still accessed through Kong-facing adapters.

That gives the final demo a clean story: Converge governs decision-making inside the app, and Kong governs the only remote AI and business access outside the app.

## Current Desktop App Usage

The desktop editor already uses this pattern for Truth-heading rewrite guidance:

1. The Tauri config module loads `.env`.
2. It checks whether `KONG_AI_GATEWAY_URL` and `KONG_API_KEY` are present.
3. It builds a `KongRoute` from local editor settings.
4. It creates `KongGateway::from_env()`.
5. It uses `gateway.llm_provider(route)` to ask for a better Truth formulation.

If Kong is not configured, the editor falls back to a local heuristic rewrite instead of failing.
