# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Changed
- Provider selection uses `select_healthy_chat_backend()` from converge v3.4.0 — automatic fallback past dead providers (e.g. exhausted Gemini free tier)
- Switch to local path deps for converge v3.4.0 and axiom-truth v0.5.0
- Removed manual `check_provider_health()` and `provider_health.rs` — upstream handles it
- Sync docs with Converge v3 contract
- Switch converge deps to crates.io, upgrade to edition 2024
- Define a canonical participant-facing API surface around `converge-pack`, `converge-kernel`, `ChatBackend`, and Organism intent/planning types
- Mark the desktop `KongGateway` / `LlmRequest` path as a migration target instead of the default template

### Added
- Policy-based vendor commitment truth
- Gemini as third agent, centralized docs in AGENTS.md
- Developers handbook
- Audit-vendor-decision truth
- KongGateway integration and desktop guidance

## 2026-04-01

### Added
- Hackathon starter kit: AI governance with multi-agent convergence
- Desktop app scaffold (SvelteKit + Tauri)
