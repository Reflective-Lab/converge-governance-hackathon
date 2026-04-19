# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [1.0.0] — 2026-04-19

### Changed
- All dependencies pinned to git tags: Converge v3.4.0, Organism v1.2.0, Axiom v0.6.0
- Cut Organism v1.2.0 tag to align with Converge v3.4.0 (eliminated duplicate crate versions)
- Desktop app deps switched from local paths to git tags
- Participants can now clone and build without sibling checkouts
- README, AGENTS.md, CAPABILITIES.md, CONTRIBUTING.md rewritten for clarity
- Documentation trimmed — crisp, to the point, no redundancy

### Added
- `kb/Converge/Organism Patterns.md` — six-stage pipeline, four topologies, five skepticism kinds, 15 domain packs
- Property tests (proptest) for governance-kernel and governance-truths
- Negative tests for invalid inputs, empty states, edge cases
- Soak tests for repeated execution stability and rollback correctness
- `just test-coverage` recipe for coverage reports
- Test categories documented in CONTRIBUTING.md

## [Unreleased]

### Changed
- Provider selection uses `select_healthy_chat_backend()` from converge v3.4.0
- Canonical participant-facing API surface: `converge-pack`, `converge-kernel`, `ChatBackend`, Organism types

### Added
- Policy-based vendor commitment truth
- Audit-vendor-decision truth
- KongGateway integration and desktop guidance

## 2026-04-01

### Added
- Hackathon starter kit: AI governance with multi-agent convergence
- Desktop app scaffold (SvelteKit + Tauri)
