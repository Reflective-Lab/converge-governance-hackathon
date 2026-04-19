---
tags: [integrations, kong, api-reference]
source: mixed
---
# Kong AI Gateway — Chat Completion Response

## Response Body

The AI Gateway returns an OpenAI-compatible JSON response:

```json
{
  "model": "gpt-4o",
  "choices": [{
    "message": {
      "content": "The assistant's text reply",
      "tool_calls": [
        { "id": "call_1", "type": "function", "function": { "name": "get_weather", "arguments": "{\"city\":\"Stockholm\"}" } }
      ]
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 42,
    "completion_tokens": 18,
    "total_tokens": 60
  }
}
```

| Field | Description |
|---|---|
| `model` | Actual model that served the request (may differ from the requested model if the gateway applies routing or fallback) |
| `choices[].message.content` | The assistant's text reply |
| `choices[].message.tool_calls` | Tool/function calls the model wants to invoke — each has an `id`, function `name`, and JSON `arguments` |
| `choices[].finish_reason` | Why generation stopped: `stop` (natural end), `length` (token limit hit), `tool_calls` (model is requesting tool use), `content_filter` (blocked by content policy) |
| `usage.prompt_tokens` | Tokens consumed by the input |
| `usage.completion_tokens` | Tokens generated in the output |
| `usage.total_tokens` | Sum of prompt + completion tokens |

## Response Headers

The AI Gateway injects additional headers into every response for observability, tracing, and rate limit awareness.

| Header | Example value | Description |
|---|---|---|
| `x-kong-request-id` | `abc123-def456` | Unique request ID for tracing through the gateway |
| `x-kong-proxy-latency` | `3` | Milliseconds spent inside the Kong proxy layer |
| `x-kong-upstream-latency` | `142` | Milliseconds the upstream LLM provider took to respond |
| `x-kong-llm-model` | `openai/gpt-4o` | The provider/model pair the gateway routed to |
| `x-ratelimit-remaining-requests` | `499` | Remaining requests in the current rate limit window |
| `ratelimit-remaining` | `498` | Remaining capacity (standard rate limit header) |
| `x-request-id` | `req-789` | Correlation ID (may originate from the upstream provider) |

Upstream provider headers (e.g. `x-openai-*`, `openai-*`) are passed through transparently when the upstream includes them.

## Sources

- [AI Proxy Plugin](https://developer.konghq.com/plugins/ai-proxy/)
- [Kong AI Gateway](https://developer.konghq.com/ai-gateway/)

See also: [[Integrations/Kong Gateway]], [[Integrations/Kong Admin API]]
