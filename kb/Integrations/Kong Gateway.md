---
tags: [integrations, kong]
---
# Kong AI Gateway

An optional remote integration layer for now. When used, outbound calls from the Rust core can go through Kong.

For student-facing Rust examples, keep Kong below the same capability surface used elsewhere in Converge. Do not teach a separate primary programming API for Kong-routed calls if the rest of the repo teaches `ChatBackend`. The near-term value of Kong is shared routing, observability, and especially MCP tool bridging.

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

If you use Kong, your team will receive a Kong AI Gateway URL, an API key, and access to the Kong dashboard to see your API usage.

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

## Student-Facing Contract

Application code should stay on the canonical Converge capability boundary:

```rust
use converge_core::traits::{ChatMessage, ChatRequest, ChatRole, DynChatBackend, ResponseFormat};
```

The intended shape is:

1. The application edge constructs or injects an `Arc<dyn DynChatBackend>`
2. Suggestors and app services build `ChatRequest`
3. The backend implementation handles routing, credentials, and remote transport

That keeps the programming surface stable even if the transport path changes.

## What Not To Teach As The Default

Do not introduce these as the primary student-facing Rust API in new docs or examples:

- `KongGateway`
- `KongRoute`
- `LlmProvider`
- `LlmRequest`

Those names describe one adapter strategy, not the canonical capability contract we want students to internalize.

## Desktop App Pattern

1. Tauri config loads `.env`
2. Checks for Kong credentials if Kong routing is enabled
3. Builds or injects the chat backend at the application edge
4. Passes `ChatRequest` / `ChatResponse` across the app boundary
5. Falls back to local heuristics or offline tooling when live remote access is unavailable

## Intended Runtime Shape

1. Svelte and Tauri run as the local shell
2. The shell calls the Rust app layer locally
3. Rust suggestors run inside the Converge runtime
4. LLM-backed suggestors may call models through Kong or direct provider adapters for now
5. Service-backed suggestors should prefer MCP or service adapters, with Kong especially useful when it fronts shared MCP tools
6. Missing enterprise systems are mocked locally behind the same capability contracts

Converge governs decision-making inside the app. Kong is one optional remote routing and tool layer outside the app.

Current status: this repo now uses the canonical `ChatBackend` path, with backend selection at the Tauri edge and offline fallback through `StaticChatBackend`. Future direction: add a `KongProvider` or more general `RouterProvider` under that same capability surface.

See also: [[Integrations/MCP Tools]], [[Integrations/External Services]], [[Domain/Agents]]
