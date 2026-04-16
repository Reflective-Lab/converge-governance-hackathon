---
tags: [moc]
---
# Converge Governance Hackathon

Knowledge base for the AI governance hackathon starter kit. Multi-agent vendor selection built on [[Converge/Core Concepts|Converge]].

## Architecture
- [[Architecture/Overview]] — five-layer model, repo ownership split
- [[Architecture/Layers]] — crate responsibilities and stack diagram
- [[Architecture/Convergence Loop]] — how the engine reaches a decision

## Domain
- [[Domain/Vendor Selection]] — the challenge and success criteria
- [[Domain/Agents]] — the five suggestors to build
- [[Domain/Truths]] — truth definitions and catalog
- [[Domain/Key Types]] — Engine, Context, Fact, Criterion, etc.

## Development
- [[Development/Getting Started]] — toolchain, build commands, quick reference
- [[Development/Programming API Surfaces]] — the canonical student-facing import and layering rules
- [[Development/Writing Suggestors]] — suggestor trait, patterns, rules
- [[Development/Writing Truths]] — step-by-step guide to adding a new truth
- [[Development/Streaming]] — real-time convergence callbacks

## Integrations
- [[Integrations/Kong Gateway]] — LLM, MCP, and API access through Kong
- [[Integrations/Why Kong]] — How Kong maps to every challenge requirement
- [[Integrations/Kong Demo Story]] — End-to-end two-layer governance walkthrough
- [[Integrations/MCP Tools]] — Model Context Protocol for business services
- [[Integrations/External Services]] — mocking pattern for unavailable backends

## Workflow
- [[Workflow/Working with Claude]] — skills vs Justfile, when to use which
- [[Workflow/Working with Codex]] — prompt patterns and workflow equivalents for Codex users
- [[Workflow/Working with Gemini]] — tool integration and delegation for Gemini users
- [[Workflow/Daily Journey]] — start-to-finish cheat sheet for your day
- [[Workflow/Skills Reference]] — all `/slash` commands and what they do

## Converge Platform
- [[Converge/Crate Catalog]] — all 10 published crates, what they do, when to use them
- [[Converge/Core Concepts]] — correctness-first multi-agent runtime
- [[Converge/Building Blocks]] — types and traits from converge-core
- [[Converge/Context Keys]] — partitioning evidence in the shared context
- [[Converge/Domain Packs]] — pre-built suggestor packs from converge-domain
- [[Converge/HITL Gates]] — human-in-the-loop gated decisions
- [[Converge/Experience and Recall]] — learning from past runs
- [[Converge/Governed Artifacts]] — lifecycle management for system outputs
- [[Converge/Organism Blueprints]] — ~22 production-shaped business cases from Organism, available as hackathon examples
- [[Converge/Desktop Naming]] — Helm, Axiom, and the stack vocabulary
