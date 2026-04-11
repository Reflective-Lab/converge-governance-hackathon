---
tags: [integrations, mcp]
---
# MCP Tools

Model Context Protocol lets agents call external tools using a structured tool-call interface instead of raw HTTP. [[Integrations/Kong Gateway|Kong]] fronts MCP servers the same way it fronts REST APIs.

## Usage

```rust
use converge_provider::{KongGateway, McpClient, McpTransport};

let gateway = KongGateway::from_env()?;
let mcp = McpClient::new(
    "vendor-registry",
    McpTransport::Http {
        url: gateway.mcp_url("vendor-registry"),
    },
);

// Discover available tools
let tools = mcp.list_tools()?;

// Call a specific tool
let result = mcp.call_tool("lookup_vendor", serde_json::json!({
    "vendor_name": "Acme AI",
    "fields": ["certifications", "regions", "pricing"]
}))?;
```

## MCP vs REST

| Use case | Choose |
|---|---|
| Known endpoint and payload at compile time | REST — simpler, faster, easier to type |
| Agent needs to discover tools dynamically | MCP |
| Tool server exposes many actions | MCP |
| LLM selects which tool to call based on context | MCP |

Both go through Kong. Both get the same governance, logging, and rate limiting.

## Good MCP Candidates

- Vendor registry lookups
- Procurement policy checks
- Internal compliance evidence retrieval
- Approval workflow actions

See also: [[Integrations/Kong Gateway]], [[Integrations/External Services]]
