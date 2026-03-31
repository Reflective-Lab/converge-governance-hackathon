# AGENTS

This repository is intentionally opinionated. These choices are defaults, not suggestions.

## Product Shape

- Build a self-contained desktop app.
- The primary use case is AI vendor selection.
- The UI is local. It should not depend on a remote governance backend.
- The only intended remote calls are outbound calls to Kong and the LLM or business services Kong fronts.

## Stack

- Use Rust for as much of the system as possible.
- Use Converge as the execution model for governed multi-agent behavior.
- Use Svelte for the desktop UI.
- Use Tauri as the desktop shell.
- Use Bun for the frontend package manager and task runner.

## App Boundary

- The Svelte frontend should call local Rust commands through Tauri.
- Do not make HTTP, REST, OpenAPI, or gRPC the default app boundary.
- The existing Rust server is a local harness for development, not the product surface.

## Validation

- Accept Gherkin or Converge Truth syntax as input.
- Use `converge-tool` for spec validation.
- The current desktop validator is offline-first:
  syntax, Truth governance blocks, and Converge conventions are checked locally.
- Business-sense and compilability validation should only be enabled once a Kong-backed LLM path is defined.

## Domain Focus

- Keep vendor selection as the anchor workflow.
- Prefer flows that produce traceable evidence:
  compliance screening, cost analysis, capability matching, risk scoring, recommendation.
- If the system cannot justify a decision, it should stop honestly and ask for human review.

## Integration Rules

- Route LLM access through `converge-provider` and `KongGateway::from_env()`.
- Use Kong-hosted MCP or Kong-defined service contracts for business integrations.
- Read Kong credentials from `.env` with `KONG_AI_GATEWAY_URL` and `KONG_API_KEY`.
- Define LLM routes with `KongRoute` in app code instead of scattering raw gateway URLs through the codebase.
- If a business service is unavailable, mock it locally but preserve the same adapter boundary.

## Build Workflow

- Use `just install-desktop` to install Bun dependencies.
- Use `just dev-desktop` for the Tauri development shell.
- Use `just build-desktop` for the frontend bundle.
- Use `just package-desktop` or `just deploy` for the packaged desktop `.app` bundle.
