---
tags: [converge, reference]
---
# Converge Crate Catalog

All published Converge crates at v2.1.2 on crates.io. This project uses some of them directly — the rest are available when you need them.

## Core

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-traits` | Shared trait definitions across the Converge ecosystem | Indirect (via converge-core) |
| `converge-core` | Engine, Agent, Fact, Context, promotion gates, convergence loop, HITL gates, budgets, criteria | Yes — workspace dependency |

## Domain and Intelligence

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-domain` | Pre-built agent packs: trust, money, delivery, knowledge, data_metrics. Invariants and domain patterns. | Yes — workspace dependency |
| `converge-knowledge` | Knowledge management, signal capture, hypothesis testing, canonical decisions | Available |
| `converge-experience` | Experience tracking and recall across runs. Agents learn from past decisions. | Available |

## Integration

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-provider` | KongGateway, KongRoute, LlmProvider, McpClient. Single entry point for all external access. | Yes — desktop Tauri crate (`features = ["kong"]`) |
| `converge-mcp` | Model Context Protocol support. MCP server/client for tool-based agent access to business services. | Available |
| `converge-tool` | Spec validation, Gherkin parsing, truth-spec parsing. Offline-first validation. | Yes — desktop Tauri crate |

## Optimization

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-optimization` | Multi-criteria optimization via OR-Tools. Weighted scoring, constraint solving. | Available — stretch goal for RiskScorerAgent |
| `ortools-sys` | FFI bindings to Google OR-Tools. Low-level dependency of converge-optimization. | Indirect |

## Version

All crates are at **v2.1.2** on crates.io. GitHub tag `v2.1.2` on the Converge repo.

## Adding a New Converge Dependency

For workspace crates (`governance-*`), add to `Cargo.toml` workspace dependencies:

```toml
[workspace.dependencies]
converge-experience = "2.1.2"
```

Then in the crate's own `Cargo.toml`:

```toml
[dependencies]
converge-experience = { workspace = true }
```

For the desktop Tauri crate (`apps/desktop/src-tauri/Cargo.toml`), which is outside the workspace:

```toml
[dependencies]
converge-experience = "2.1.2"
```

## When to Use What

- **Building agents?** You already have `converge-core`. That's enough for basic agents.
- **Want pre-built audit/trust agents?** Use `converge-domain` (already available).
- **Need LLM or API access?** Use `converge-provider` with Kong (already in desktop crate).
- **Want agents to learn from past runs?** Add `converge-experience`.
- **Building MCP tool servers?** Add `converge-mcp`.
- **Need multi-criteria optimization?** Add `converge-optimization` (pulls in OR-Tools via `ortools-sys`).
- **Managing knowledge graphs or canonical decisions?** Add `converge-knowledge`.

If you need something that doesn't exist in any of these crates, say so. We patch Converge. We don't work around it.

See also: [[Converge/Core Concepts]], [[Converge/Building Blocks]], [[Converge/Domain Packs]]
