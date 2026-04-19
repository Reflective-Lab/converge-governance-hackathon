---
source: mixed
tags: [configuration, providers, agents]
---

# Provider Configuration for Agents

Each agent in the governance system is matched to an AI model based on its **capability requirements**. When you start the desktop app, the system auto-detects available providers and recommends the best model for each agent's job.

---

## How It Works

### 1. Agent Requirements

Each agent declares what it needs: cost class, latency, quality, reasoning capability, etc.

| Agent | What it does | Key requirement | Preferred model |
|---|---|---|---|
| **Compliance Screener** | Rule-based vendor compliance checks | Fast (Low cost, <3s) | Claude Haiku, GPT-4o Mini |
| **Cost Analysis** | Budget and operating cost estimation | Fast (Low cost, <5s) | Claude Haiku, Mistral |
| **Capability Matcher** | Evaluate vendor capabilities | Capable (Medium cost, <8s) | Claude Sonnet, GPT-4 |
| **Risk Scorer** | Multi-dimensional risk assessment | Fast computation (Local or cheap model) | Claude Haiku |
| **Decision Synthesis** | Synthesize all evidence into recommendation | Reasoning (High quality, <10s) | Claude Sonnet, o1 |

### 2. Provider Detection

When the app starts, it:

1. Reads your `.env` file for API keys: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `OPENROUTER_API_KEY`, `KONG_API_KEY`
2. For each provider with a key, sends a minimal test prompt to verify it's working
3. Builds a list of available models per provider
4. Calls `ModelSelector` to find the best model for each agent's requirements
5. Shows the "Provider Setup" screen with recommended models

```
┌─────────────────────┐
│ Check .env for keys │
└──────────┬──────────┘
           ↓
┌──────────────────────────────┐
│ Ping each provider (test req) │
└──────────┬───────────────────┘
           ↓
┌──────────────────────────────┐
│ Build available models list  │
└──────────┬───────────────────┘
           ↓
┌──────────────────────────────┐
│ Match agents to models       │
│ (ModelSelector per agent req)│
└──────────┬───────────────────┘
           ↓
┌──────────────────────────────┐
│ Show Provider Setup screen   │
│ (recommended models ready)   │
└──────────────────────────────┘
```

### 3. Model Selection

Recommended models are chosen by `ModelSelector` using a fitness score that weighs:
- **40% Cost** — Prefer cheaper models that meet quality requirements
- **30% Latency** — Prefer faster models
- **30% Quality** — Higher quality models score better

---

## Setting Up Providers

### Option 1: Anthropic (Recommended)

```bash
# .env
ANTHROPIC_API_KEY=sk-ant-...
```

Cost: Free tier available (Claude 3.5 Haiku)

Models available:
- `claude-3-5-haiku-20241022` (Fast, cheap)
- `claude-3-5-sonnet-20241022` (Capable, reasoning)
- `claude-opus-4-1-20250805` (Most capable)

### Option 2: OpenAI

```bash
# .env
OPENAI_API_KEY=sk-proj-...
```

Cost: Paid only

Models available:
- `gpt-4o-mini` (Fast, cheap)
- `gpt-4o` (Capable)
- `gpt-4-turbo` (Reasoning)

### Option 3: OpenRouter (Free tier available)

```bash
# .env
OPENROUTER_API_KEY=sk-or-...
```

Cost: Free tier with credit (https://openrouter.ai)

Models available: 200+ models from multiple providers
- `mistral/mistral-7b-instruct` (Fast)
- `meta-llama/llama-3-70b-instruct` (Capable)
- Many others

### Option 4: Kong Gateway

```bash
# .env
KONG_AI_GATEWAY_URL=https://your-konnect-url
KONG_API_KEY=...
KONG_LLM_ROUTE=llm/v1/chat
```

Cost: Enterprise

Kong route details are configurable via `KONG_LLM_ROUTE`:
- Provide a path like `llm/v1/chat` (default)
- Leading/trailing slashes are stripped automatically
- `KONG_LLM_ROUTE` is optional; if omitted, `llm/v1/chat` is used

Routes through Kong for governance, cost tracking, PII redaction, rate limiting.

### Kong-Free Mode

You can run every participant-facing flow without Kong:

1. Remove or ignore `KONG_AI_GATEWAY_URL` and `KONG_API_KEY`.
2. Set one direct provider in `.env` (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc.).
3. Optionally set `CONVERGE_LLM_FORCE_PROVIDER=<provider>` to pin the provider explicitly.

This keeps the same governance contract and `TruthExecutionResult` API because selection still happens through `ChatBackend`/`ChatRequest` contracts.

Kong credentials are still available when needed, and the app can switch to them without changing core architecture code.

---

## Customizing Agent Requirements

If you want to change what model an agent uses, edit `governance-truths/src/lib.rs`:

```rust
pub const AGENT_MODELS: &[AgentModelConfig] = &[
    AgentModelConfig {
        agent_id: "compliance-screener",
        agent_name: "Compliance Screener",
        description: "...",
        requirements: AgentRequirements {
            max_cost_class: CostClass::Low,      // ← Increase to Medium to allow more expensive models
            max_latency_ms: 3000,                 // ← Increase for slower models
            min_quality: 0.7,                     // ← Increase for higher quality
            requires_reasoning: false,            // ← Set to true if you need reasoning
            // ... other fields
        },
    },
    // ... other agents
];
```

After editing, rebuild the server:

```bash
cargo build -p governance-server
just server  # Restart the server
```

When you reload the desktop app, it will detect your new requirements and recommend different models.

---

## How Provider Health Checks Work

When you start the demo, the server:

1. **Detects available providers** from `.env` (looks for API key env vars)
2. **Health checks each provider** by sending a minimal test request ("test", max 10 tokens)
3. **Filters out invalid keys** — if a key is wrong, expired, or revoked, it's silently removed from the available list
4. **Shows only working models** in the Provider Setup screen

This prevents 401 "authentication denied" errors during the actual demo run.

**How it works under the hood:**
```rust
// For each provider with an API key:
let backend = select_chat_backend(provider, model)?;
let test_req = ChatRequest { messages: [{ role: User, content: "test" }], max_tokens: 10 };
match backend.chat(test_req).await {
    Ok(_) => { /* Provider is valid, show it */ },
    Err(AuthError) => { /* Key is invalid, hide it */ },
    Err(NetworkError) => { /* Provider unreachable, hide it */ },
}
```

---

## Troubleshooting

### "No providers available"

- Check that at least one provider is configured in `.env` with a **valid API key**
- Restart the server after adding or updating a key in `.env`
- Check the server logs for health check failures:
  ```bash
  RUST_LOG=warn just server
  ```
  Look for: `Provider X/Y health check failed: authentication denied`
- For Kong: Check that `KONG_AI_GATEWAY_URL` is reachable and `KONG_API_KEY` is valid

### "No suitable provider found for this agent's requirements"

Your current providers don't have models that meet the agent's requirements. Either:

1. **Lower the requirements:** Edit the agent's `AgentRequirements` (raise `max_cost_class`, lower `min_quality`, etc.)
2. **Add another provider:** Add API key to `.env` and restart server
3. **Try a different model:** Some providers have multiple models available

### "Provider timeout" when loading the setup screen

The server is taking too long to health check providers. This usually means:

- A provider is down or unreachable
- Your internet connection is slow
- An API key is wrong (the provider rejects it slowly)

**Fix:**
- Check internet connection
- Verify the API key is correct
- Restart the server
- Check logs: `RUST_LOG=warn just server`

The health check has a reasonable timeout, but if a provider is slow, it will show as unavailable in the Provider Setup screen. This is intentional — we'd rather show no provider than risk a timeout during the demo.

---

## How Agents Use Models

Once you've selected providers in the setup screen, agents don't manually choose models. Instead, they declare what they need (`AgentRequirements`), and the system automatically uses the best available model.

**For participants:** You usually don't need to think about this. The setup screen shows you what models will be used, and you can click "Start Demo" to proceed.

**For developers:** If you're writing a new agent, add `AgentRequirements` to `governance-truths` so the system knows what kind of model it needs.

See also: [[Development/Writing Suggestors]], [[../Converge/Visualization Alternatives]]
