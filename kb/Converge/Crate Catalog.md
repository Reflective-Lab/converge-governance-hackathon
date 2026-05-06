---
tags: [converge, reference]
---
# Converge Crate Catalog

Converge core crates are pinned to v3.8.1. Extension crates are pinned to the Converge 3.8.1-compatible versions in the root Cargo manifest.

## Public Contract Crates

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-pack` | Suggestor, context, invariant authoring contract |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available and preferred for participant-facing authoring |
| `converge-provider-adapters` | Dependency alias for `converge-manifold-adapters`: LLM, search, tool, and storage adapters, including Kong | Yes — workspace and desktop manifests| Chat contracts, backend identity, and capability routing |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available and preferred for participant-facing capability code |
| `converge-model` | Curated semantic types (Fact, Proposal, PromotionRecord) |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available |
| `converge-kernel` | In-process embedding API (Engine, RunResult, Budget) |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available |
| `converge-protocol` | Generated `converge.v1` wire types (protobuf/gRPC) |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available |
| `converge-client` | Remote Rust SDK for Converge runtimes |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available |

## Core

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-core` | Constitutional internals and legacy re-exports used by current implementation code |`converge-domain` | Dependency alias for `converge-atelier-domain` domain packs: trust, money, delivery, knowledge, data_metrics | Yes — workspace dependency |

## Domain and Intelligence

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-domain` | Dependency alias for `converge-atelier-domain` domain packs: trust, money, delivery, knowledge, data_metrics | Yes — workspace dependency|`converge-domain` | Dependency alias for `converge-atelier-domain` domain packs: trust, money, delivery, knowledge, data_metrics | Yes — workspace dependency|`converge-domain` | Dependency alias for `converge-atelier-domain` domain packs: trust, money, delivery, knowledge, data_metrics | Yes — workspace dependency |
| `converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available|`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available|`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available |
| `converge-experience` | Experience tracking and recall across runs |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available |

## Integration

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-provider-adapters` | Dependency alias for `converge-manifold-adapters`: LLM, search, tool, and storage adapters, including Kong | Yes — workspace and desktop manifests|`converge-provider-adapters` | Dependency alias for `converge-manifold-adapters`: LLM, search, tool, and storage adapters, including Kong | Yes — workspace and desktop manifests|`converge-provider-adapters` | Dependency alias for `converge-manifold-adapters`: LLM, search, tool, and storage adapters, including Kong | Yes — workspace and desktop manifests |
| `converge-mcp` | Model Context Protocol support |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available |
| `axiom-truth` | Truth validation, Gherkin parsing, truth-spec parsing | Yes — desktop Tauri crate |

## Optimization

| Crate | What it does | Used in this project |
|---|---|---|
| `converge-optimization` | Multi-criteria optimization via OR-Tools |`converge-knowledge` | Dependency alias for `converge-mnemos-knowledge`: knowledge management, signal capture, hypothesis testing | Available |
| `ortools-sys` | FFI bindings to Google OR-Tools | Indirect |

## Deprecated

| Crate | Status |
|---|---|
| `converge-traits` | Deprecated. Use `converge-pack` + `converge-provider-adapters` | Dependency alias for `converge-manifold-adapters`: LLM, search, tool, and storage adapters, including Kong | Yes — workspace and desktop manifestsinstead. |

## Adding a New Converge Dependency

For workspace crates (`governance-*`), add to `Cargo.toml` workspace dependencies:

```toml
[workspace.dependencies]
converge-experience = "3.8.1"
```

Then in the crate's own `Cargo.toml`:

```toml
[dependencies]
converge-experience = { workspace = true }
```

For the desktop Tauri crate (`apps/desktop/src-tauri/Cargo.toml`), which is outside the workspace:

```toml
[dependencies]
converge-experience = "3.8.1"
```

## When to Use What

- **Building suggestors?** Start with `converge-pack`.
- **Want pre-built audit/trust suggestors?** Use `converge-domain` | Dependency alias for `converge-atelier-domain` domain packs: trust, money, delivery, knowledge, data_metrics | Yes — workspace dependency(already available).
- **Need chat or selection contracts?** Use `converge-provider`.
- **Need ready-made LLM or API adapters?** Use `converge-provider-adapters` | Dependency alias for `converge-manifold-adapters`: LLM, search, tool, and storage adapters, including Kong | Yes — workspace and desktop manifestswith Kong or direct providers under the same contract.
- **Want suggestors to learn from past runs?** Add `converge-experience`.
- **Building MCP tool servers?** Add `converge-mcp`.
- **Need multi-criteria optimization?** Add `converge-optimization`.
- **Managing knowledge graphs?** Add `converge-knowledge`.

If you need something that doesn't exist in any of these crates, say so. We patch Converge. We don't work around it.

See also: [[Converge/Core Concepts]], [[Converge/Building Blocks]], [[Converge/Domain Packs]]
