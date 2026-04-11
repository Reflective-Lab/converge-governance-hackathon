---
tags: [integrations]
---
# External Services and Mocking

During the hackathon, real enterprise services are unavailable. Don't collapse the architecture — mock them behind the same interface.

## Pattern

1. Define a trait for the capability the suggestor needs
2. Implement it against [[Integrations/Kong Gateway|Kong]] for production
3. Implement it as a local mock for development
4. Inject it into the suggestor at construction time

The suggestor never knows whether it talks to a real service or a mock.

```rust
trait PolicyService: Send + Sync {
    fn get_policies(&self, jurisdiction: &str) -> Result<Vec<PolicyRule>, String>;
}

struct KongPolicyService {
    gateway: KongGateway,
}

impl PolicyService for KongPolicyService {
    fn get_policies(&self, jurisdiction: &str) -> Result<Vec<PolicyRule>, String> {
        let url = self.gateway.api_url(
            &format!("compliance/v1/policies?jurisdiction={jurisdiction}")
        );
        // HTTP call through Kong
        todo!()
    }
}

struct MockPolicyService;

impl PolicyService for MockPolicyService {
    fn get_policies(&self, _jurisdiction: &str) -> Result<Vec<PolicyRule>, String> {
        Ok(vec![
            PolicyRule { id: "gdpr-1".into(), description: "Data must stay in EU".into() },
            PolicyRule { id: "ai-act-1".into(), description: "High-risk AI requires conformity assessment".into() },
        ])
    }
}

// Inject into agent:
struct ComplianceScreenerAgent {
    policies: Arc<dyn PolicyService>,
}
```

Swapping from mock to real is a one-line change at the injection site. This keeps the demo honest — teams show the intended integration pattern without pretending that hardcoded agent logic is the same as governed service access.

## Good Mock Candidates

- **Vendor profile service** — certifications, regions, pricing plans
- **Policy engine** — internal guardrails, jurisdiction rules
- **Procurement approval service** — budget thresholds, escalation rules
- **Compliance evidence store** — structured documents for screening
- **Pricing catalog** — token costs, volume discounts

## Mock Through Kong

If the mock is useful to other teams:
1. Build a small local mock service in Rust
2. Expose it with a stable API or [[Integrations/MCP Tools|MCP]] surface
3. Register or proxy it through Kong
4. Agents call the mock through Kong, exactly as they would the real service

See also: [[Development/Writing Agents]], [[Integrations/Kong Gateway]]
