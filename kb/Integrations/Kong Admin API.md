---
tags: [integrations, kong, api-reference, admin]
source: mixed
---
# Kong Konnect Admin APIs

All admin APIs are authenticated via **Personal Access Token** (PAT) in the `Authorization: Bearer` header against the Konnect control plane at `https://{region}.api.konghq.com`.

## Control Plane Management

| Endpoint | Method | Purpose |
|---|---|---|
| `/v2/control-planes` | GET | List all control planes |
| `/v2/control-planes/{id}` | GET | Get control plane details |
| `/v2/control-planes` | POST | Create a new control plane |
| `/v2/control-planes/{id}` | PATCH/DELETE | Update or delete a control plane |

## Gateway Entity Configuration

All under `/v2/control-planes/{cpId}/core-entities/`:

| Endpoint | Methods | Purpose |
|---|---|---|
| `.../services` | GET, POST | List/create upstream services |
| `.../services/{id}` | GET, PATCH, DELETE | Manage a specific service |
| `.../services/{id}/routes` | GET, POST | Routes bound to a service |
| `.../routes` | GET, POST | List/create routes |
| `.../routes/{id}` | GET, PATCH, DELETE | Manage a specific route |
| `.../plugins` | GET, POST | List/create plugins (including ai-proxy) |
| `.../plugins/{id}` | GET, PATCH, DELETE | Manage a specific plugin |
| `.../consumers` | GET, POST | List/create consumers (API key holders) |
| `.../consumers/{id}` | GET, PATCH, DELETE | Manage a specific consumer |
| `.../consumers/{id}/key-auth` | GET, POST | Manage API keys for a consumer |

## AI Proxy Plugin Configuration

Configured as a plugin on a service or route:

```json
POST .../services/{id}/plugins
{
  "name": "ai-proxy",
  "config": {
    "route_type": "llm/v1/chat",
    "auth": { "header_name": "Authorization", "header_value": "Bearer sk-..." },
    "model": {
      "provider": "openai",
      "name": "gpt-4o",
      "options": { "max_tokens": 4096, "temperature": 0.0 }
    },
    "logging": {
      "log_statistics": true,
      "log_payloads": false
    }
  }
}
```

The **AI Proxy Advanced** plugin extends this with multi-model load balancing and fallback chains.

## Analytics & Observability

| Endpoint | Method | Purpose |
|---|---|---|
| `/v2/api-requests` | GET | Query API request analytics with filters (time range, status, consumer, route) |
| `/v2/control-planes/{id}/analytics` | GET | Per-control-plane analytics dashboard data |

Konnect Advanced Analytics provides:
- **Token usage** — prompt, completion, and total tokens per model, consumer, and route
- **Cost tracking** — per-request cost calculation based on provider pricing
- **Latency metrics** — upstream latency, proxy latency, total request time
- **Error rates** — by status code, model, and consumer
- **Rate limit state** — consumption against configured limits

These feed into pre-built dashboards in Konnect, or can be exported to your own observability stack (Datadog, Prometheus, etc.) via logging plugins (`http-log`, `datadog`, `opentelemetry`).

## Audit Logging

| Endpoint | Method | Purpose |
|---|---|---|
| `/v2/audit-logs` | GET | Organization-wide audit trail (who changed what, when) |

The AI Proxy plugin also records per-request LLM usage statistics into whichever Kong log plugin is active, including token counts, model used, and optionally the full request/response payloads.

## Sources

- [Konnect Control Planes Config API](https://docs.konghq.com/konnect/api/control-plane-configuration/latest/)
- [Kong Admin API](https://developer.konghq.com/admin-api/)
- [AI Proxy Plugin](https://developer.konghq.com/plugins/ai-proxy/)
- [AI Proxy Advanced Plugin](https://developer.konghq.com/plugins/ai-proxy-advanced/)
- [Kong AI Gateway](https://developer.konghq.com/ai-gateway/)
- [Konnect OpenAPI Specifications](https://developer.konghq.com/api/)
- [Gateway Configuration in Konnect](https://docs.konghq.com/konnect/gateway-manager/configuration/)
- [Konnect Control Planes API v2](https://developer.konghq.com/api/konnect/control-planes/v2/)

See also: [[Integrations/Kong Gateway]], [[Integrations/Kong Chat Response]]
