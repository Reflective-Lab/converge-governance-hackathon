---
tags: [integrations, mcp]
---
# MCP Tools

Model Context Protocol lets agents call external tools using a structured tool-call interface instead of raw HTTP. [[Integrations/Kong Gateway|Kong]] can front MCP servers the same way it fronts REST APIs, but direct MCP endpoints are also fine during the current transition.

## Usage

```rust
use converge_provider::{McpClient, McpTransport};

let mcp = McpClient::new(
    "vendor-registry",
    McpTransport::Http {
        url: configured_mcp_url,
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

Resolve Kong-routed MCP URLs at the application edge when Kong is in use, or pass direct MCP URLs in local setups. Keep the student-facing API on `McpClient` and `McpTransport`, not on a gateway-specific object.

## MCP vs REST

| Use case | Choose |
|---|---|
| Known endpoint and payload at compile time | REST — simpler, faster, easier to type |
| Agent needs to discover tools dynamically | MCP |
| Tool server exposes many actions | MCP |
| LLM selects which tool to call based on context | MCP |

Both can go through Kong when Kong is in use. Kong becomes especially valuable when multiple tools need shared governance, auth, rate limiting, and MCP bridging behind one router.

## Good MCP Candidates

- Vendor registry lookups
- Procurement policy checks
- Internal compliance evidence retrieval
- Approval workflow actions

See also: [[Integrations/Kong Gateway]], [[Integrations/External Services]]
