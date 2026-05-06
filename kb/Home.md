---
tags: [moc]
source: mixed
---
# Vendor Selection

Knowledge base for the governed vendor-selection app. This repo is being migrated from a hackathon starter kit into a product baseline for web, backend, database, and desktop delivery.

## Architecture
- [[Architecture/Overview]] — five-layer model, repo ownership split
- [[Architecture/Layers]] — crate responsibilities and stack diagram
- [[Architecture/Axiom Truth Contract]] — Axiom as the normative truth-and-policy specification layer
- [[Architecture/Convergence Loop]] — how the engine reaches a decision

## Domain
- [[Domain/Vendor Selection]] — the challenge and success criteria
- [[Domain/Landscape]] — where this project sits vs. Ramp, Coupa, Ariba, Zip, Brex, Airbase
- [[Domain/Agents]] — the five suggestors to build
- [[Domain/Truths]] — the single product truth and supporting migration fixtures
- [[Domain/Key Types]] — Engine, Context, Fact, Criterion, etc.

## Development
- [[Development/Getting Started]] — toolchain, build commands, quick reference
- [[Development/Provider Configuration]] — Agent model matching, provider setup, customization
- [[Development/Template Handoff]] — boundary between template-era work and product integration
- [[Development/Programming API Surfaces]] — canonical import and layering rules
- [[Development/Writing Suggestors]] — suggestor trait, patterns, rules
- [[Development/Writing Truths]] — historical guide; product work should extend `vendor-selection`
- [[Development/Streaming]] — real-time convergence callbacks

## Integrations
- [[Integrations/Kong Gateway]] — LLM, MCP, and API access through Kong
- [[Integrations/Why Kong]] — How Kong maps to governance requirements
- [[Integrations/Kong Demo Story]] — End-to-end two-layer governance walkthrough
- [[Integrations/Web Search Demo Story]] — Brave-wide and Tavily-deep provider mix for governed evidence gathering
- [[Integrations/MCP Tools]] — Model Context Protocol for business services
- [[Integrations/External Services]] — mocking pattern for unavailable backends

## Presentations
- [[Presentations/Team Pitch - Governed Vendor Selection]] — team pitch, demo lines, and fulfillment plan for the vendor-selection story

## Workflow
- [[Workflow/Working with Claude]] — skills vs Justfile, when to use which
- [[Workflow/Working with Codex]] — prompt patterns and workflow equivalents for Codex users
- [[Workflow/Working with Gemini]] — tool integration and delegation for Gemini users
- [[Workflow/Daily Journey]] — start-to-finish cheat sheet for your day
- [[Workflow/Skills Reference]] — all `/slash` commands and what they do

## Maintenance

- [[LOG]] — mutation log for all kb/ updates
- [[Experiments/INDEX]] — hypothesis-driven development with evidence logging
- [[Experiments/LOG]] — mutation log of all experiments

## Converge Platform
- [[Converge/Crate Catalog]] — all 10 published crates, what they do, when to use them
- [[Converge/Core Concepts]] — correctness-first multi-agent runtime
- [[Converge/Building Blocks]] — types and traits from the curated Converge surfaces
- [[Converge/Context Keys]] — partitioning evidence in the shared context
- [[Converge/Domain Packs]] — pre-built suggestor packs from converge-domain
- [[Converge/HITL Gates]] — human-in-the-loop gated decisions
- [[Converge/Experience and Recall]] — learning from past runs
- [[Converge/Governed Artifacts]] — lifecycle management for system outputs
- [[Converge/Organism Patterns]] — the six-stage pipeline: intent, planning, adversarial, simulation, learning, domain packs
- [[Converge/Organism Blueprints]] — ~22 production-shaped business cases from Organism
- [[Converge/Desktop Naming]] — Helm, Axiom, and the stack vocabulary

## Visualization
- [[Converge/Threlte Visualization]] — Why Threlte for 3D agent convergence visualization
- [[Converge/Visualization Alternatives]] — Trade-offs: Three.js, React Three Fiber, Bevy, Canvas 2D, etc.
- [[Converge/Bevy Deep Dive]] — ECS architecture, when Bevy wins, scenarios and code examples
