---
tags: [converge, reference]
---
# Converge Crate Catalog

All Converge crates at v3.0.0.

## Public Contract Crates

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-pack` | Suggestor, context, invariant authoring contract | Available and preferred for participant-facing authoring |
| `converge-provider-api` | Chat contracts, backend identity, and capability routing | Available and preferred for participant-facing capability code |
| `converge-model` | Curated semantic types (Fact, Proposal, PromotionRecord) | Available |
| `converge-kernel` | In-process embedding API (Engine, RunResult, Budget) | Available |
| `converge-protocol` | Generated `converge.v1` wire types (protobuf/gRPC) | Available |
| `converge-client` | Remote Rust SDK for Converge runtimes | Available |

## Core

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-core` | Constitutional internals and legacy re-exports used by current implementation code | Yes — workspace dependency |

## Domain and Intelligence

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-domain` | Pre-built suggestor packs: trust, money, delivery, knowledge, data_metrics | Yes — workspace dependency |
| `converge-knowledge` | Knowledge management, signal capture, hypothesis testing | Available |
| `converge-experience` | Experience tracking and recall across runs | Available |

## Integration

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-provider` | LLM provider adapters and Kong integration | Yes — desktop Tauri crate (`features = ["kong"]`) |
| `converge-mcp` | Model Context Protocol support | Available |
| `axiom-truth` | Truth validation, Gherkin parsing, truth-spec parsing | Yes — desktop Tauri crate |

## Optimization

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-optimization` | Multi-criteria optimization via OR-Tools | Available |
| `ortools-sys` | FFI bindings to Google OR-Tools | Indirect |

## Deprecated

| Crate | Status |
|---|---|
| `converge-traits` | Deprecated. Use `converge-pack` + `converge-provider-api` instead. |

## Adding a New Converge Dependency

For workspace crates (`governance-*`), add to `Cargo.toml` workspace dependencies:

```toml
[workspace.dependencies]
converge-experience = "3.0.0"
```

Then in the crate's own `Cargo.toml`:

```toml
[dependencies]
converge-experience = { workspace = true }
```

For the desktop Tauri crate (`apps/desktop/src-tauri/Cargo.toml`), which is outside the workspace:

```toml
[dependencies]
converge-experience = "3.0.0"
```

## When to Use What

- **Building suggestors?** Start with `converge-pack`.
- **Want pre-built audit/trust suggestors?** Use `converge-domain` (already available).
- **Need chat or selection contracts?** Use `converge-provider-api`.
- **Need ready-made LLM or API adapters?** Use `converge-provider` with Kong or direct providers under the same contract.
- **Want suggestors to learn from past runs?** Add `converge-experience`.
- **Building MCP tool servers?** Add `converge-mcp`.
- **Need multi-criteria optimization?** Add `converge-optimization`.
- **Managing knowledge graphs?** Add `converge-knowledge`.

If you need something that doesn't exist in any of these crates, say so. We patch Converge. We don't work around it.

See also: [[Converge/Core Concepts]], [[Converge/Building Blocks]], [[Converge/Domain Packs]]
