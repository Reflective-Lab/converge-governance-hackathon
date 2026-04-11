---
tags: [architecture]
---
# Architecture Overview

This repo is an application starter built on top of [[Converge/Core Concepts|Converge]]. Two layers:

- **Converge layer:** shared runtime model for governed multi-agent execution
- **Hackathon layer:** opinionated local-first desktop application that teams extend

## Repo Ownership Split

**Converge owns:**
- Agent execution cycles
- Shared context and context partitions
- Fact proposal and promotion
- Criteria evaluation
- Convergence budgets and stop reasons

**This repo owns:**
- Governance domain records (vendors, decisions, audit entries)
- Truth definitions for hackathon use cases
- Projection from converged facts into domain records
- The shared application layer
- A lightweight local harness for developer testing

Teams should build *with* Converge, not around it. The value is governed convergence, not just "multiple calls to an LLM."

## Opinionated Implementation

- **Rust-first** for orchestration, domain logic, policy enforcement, integrations, and mocks
- **Svelte** for the user interface
- **Tauri** for desktop packaging and native shell integration
- **[[Integrations/Kong Gateway|Kong]]** as the only intended remote integration layer

## Local Input Model

The desktop app accepts two local input formats for [[Domain/Vendor Selection|vendor selection]]:

- Gherkin `.feature` files
- Truth-spec `.truths.json` files

Both are normalized in the Rust app layer before execution.

See also: [[Architecture/Layers]], [[Architecture/Convergence Loop]]
