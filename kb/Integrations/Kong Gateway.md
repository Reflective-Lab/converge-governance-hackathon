---
tags: [integrations, kong]
---
# Kong AI Gateway

Kong AI Gateway provides the **external governance layer** for all AI system access. When used with Converge, you get **two-layer AI governance**:

- **Converge (internal):** Agents propose facts. Cedar policies decide who can propose what. The engine promotes only what passes. Every decision has full provenance.
- **Kong (external):** Every LLM call is routed through Kong AI Gateway. Kong meters token usage, redacts PII, rate-limits per team, and provides a single credential surface.

Together they answer the two questions an auditor asks:
1. "How did you reach this decision?" → Converge: full convergence trail, criteria evaluation, Cedar policy gates
2. "What external resources did you consult, at what cost, and did you leak any sensitive data?" → Kong: API audit log, token costs, PII redaction receipts

## What Kong Handles

- **Rate limiting** — Per-team token budget enforcement
- **PII detection** — Redacts sensitive data before it hits the model
- **Token usage logging** — Every request logged with tokens consumed
- **Cost tracking** — Per-request and cumulative cost visibility
- **Latency monitoring** — Proxy and upstream latency per request
- **Centralized governance** — Single credential surface for all providers
- **Provider routing** — Dynamically route to OpenAI, Anthropic, Gemini, Mistral, etc. through one gateway
- **MCP tool access** — Kong can front shared MCP servers

## Konnect Setup

This hackathon uses **Kong Konnect** (cloud SaaS). You'll receive:

- **KONG_AI_GATEWAY_URL** — Your Konnect runtime URL (e.g., `https://<org>.kongcloud.io`)
- **KONG_API_KEY** — Your Konnect personal access token

### Environment Configuration

```dotenv
# Kong AI Gateway (primary LLM routing for this hackathon)
KONG_AI_GATEWAY_URL=https://<your-konnect-url>
KONG_API_KEY=<your-konnect-token>
KONG_LLM_ROUTE=llm/v1/chat

# Converge backend selection: use Kong by default
CONVERGE_LLM_FORCE_PROVIDER=kong
# Optional: override the upstream model Kong routes to
# CONVERGE_LLM_MODEL=anthropic/claude-sonnet-4-20250514
```

## Implementation

The `KongChatBackend` in `converge-provider` implements `DynChatBackend`. It:

1. Accepts `ChatRequest` (canonical Converge format)
2. Sends OpenAI-format body to `{KONG_AI_GATEWAY_URL}/{KONG_LLM_ROUTE}`
   - Default route: `llm/v1/chat`
   - Configure this in `KONG_LLM_ROUTE` when your Konnect route is named/path-based differently
3. Authenticates via `apikey` header (Konnect Key Auth)
4. Kong translates to whatever upstream provider is configured
5. Returns standard `ChatResponse` with `TokenUsage` from the response body

```rust
// Selection is automatic when KONG_API_KEY + KONG_AI_GATEWAY_URL are set
use converge_provider::select_chat_backend;
let selected = select_chat_backend(&config)?;
// selected.backend is now KongChatBackend
```

## Kong Plugins Enabled

This hackathon enables these Kong Enterprise plugins:

- **AI Proxy** — Routes requests to upstream providers (OpenAI, Anthropic, etc.)
- **Key Auth** — `apikey` header authentication
- **AI Rate Limiting** — Per-team token budgets (optional, enable in Konnect dashboard)
- **AI PII Sanitizer** — Redacts PII before prompts reach models (optional)

## Desktop App Pattern

1. Tauri config loads `.env`
2. `CONVERGE_LLM_FORCE_PROVIDER=kong` is set by default
3. Backend selection uses Kong when `KONG_API_KEY` is available
4. All LLM calls route through Kong automatically
5. Token usage flows into `ChatResponse.usage` for ExperienceStore capture
6. Falls back to offline `StaticChatBackend` when Kong is unreachable

## What Not To Teach As The Default

These are Kong-specific internal types — do not expose as primary participant-facing APIs:

- `KongGateway`
- `KongRoute`
- `LlmProvider`
- `LlmRequest`

Keep application code on `ChatBackend` / `ChatRequest`. Kong is the transport, not the contract.

## Demo Story

See [[Integrations/Kong Demo Story]] for the end-to-end demonstration walkthrough.

## Provider Availability

When the `kong` feature is enabled in `converge-provider`:

| Provider | Secret Key | Auto-Discovery |
|---|---|---|
| `kong` | `KONG_API_KEY` + `KONG_AI_GATEWAY_URL` env var | Yes — if both are set, Kong participates in auto-selection |

To force Kong: `CONVERGE_LLM_FORCE_PROVIDER=kong`

To bypass Kong: Set `CONVERGE_LLM_FORCE_PROVIDER=anthropic` (or another provider)

## Files Changed

- `converge/crates/provider/src/llm/kong.rs` — KongChatBackend implementation
- `converge/crates/provider/src/llm/mod.rs` — Module declaration
- `converge/crates/provider/src/lib.rs` — Re-export
- `converge/crates/provider/src/llm/selection.rs` — Backend selection wiring
- `hackathon/Cargo.toml` — `kong` feature enabled
- `hackathon/apps/desktop/src-tauri/Cargo.toml` — Path dep + `kong` feature

See also: [[Integrations/Kong Chat Response]], [[Integrations/Kong Admin API]], [[Integrations/Kong Demo Story]], [[Integrations/MCP Tools]], [[Integrations/External Services]], [[Domain/Agents]]
