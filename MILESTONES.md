# Vendor Selection Milestones

Single source of truth for what ships and when. Every session starts by reading this file. Scope work to the current milestone.

See `/Users/kpernyer/dev/work/EPIC.md` for workspace strategy when available.

## Current: Product Baseline

**Deadline:** TBD  
**Goal:** Convert the hackathon repo into a clean vendor-selection product starter with one release train, one canonical product truth, GitHub-ready docs, standard skills, clear deployment targets, and a truthful map of current versus planned architecture.

### Repository Scaffold

- [x] Root agent entrypoints exist: `AGENTS.md`, `CLAUDE.md`, `CODEX.md`, `GEMINI.md`
- [x] GitHub docs exist: `README.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `SUPPORT.md`, `CHANGELOG.md`
- [x] License file exists
- [x] GitHub issue and PR templates exist
- [x] Standard local workflow skills are present under `.claude/skills`
- [x] Workflow docs say release branches only; no feature branches and no worktrees
- [ ] Stale hackathon-only docs are either rewritten or moved under historical notes

### One Truth

- [ ] Make `vendor-selection` the only product truth exposed by app/operator surfaces
- [ ] Demote `evaluate-vendor`, `dynamic-due-diligence`, `audit-vendor-decision`, `authorize-vendor-commitment`, `budget-approval`, and `access-control` to examples, fixtures, or archived migration references
- [ ] Rename truth-catalog docs to make the single product truth explicit
- [ ] Keep one governed endpoint for the product workflow
- [ ] Keep supporting policy and audit behavior inside the `vendor-selection` flow instead of separate product truths

### Product Architecture

- [ ] Decide web app location: reuse `apps/desktop` UI shell or create `apps/web`
- [ ] Define gRPC API contracts for vendor evaluation, evidence, decision history, and audit trails
- [ ] Select monitored Google Cloud database target and schema migration tool
- [ ] Add Terraform for Firebase, backend runtime, database, monitoring, and secrets
- [ ] Add Firebase Hosting config based on the Wolfgang deployment pattern
- [ ] Add backend container build and Cloud Run deployment path

### Vendor Selection Core

- [ ] Promote `vendor-selection` to the canonical product truth
- [ ] Remove or archive product-facing references to noncanonical truths
- [ ] Replace participant language with buyer/operator language
- [ ] Add durable persistence for decisions, vendors, criteria, evidence, and audit entries
- [ ] Add provenance-preserving imports for vendor questionnaires and evidence packs

### Desktop Release

- [ ] Align Tauri app name, bundle identifiers, icons, and metadata with Vendor Selection
- [ ] Package macOS Apple silicon
- [ ] Package macOS Intel
- [ ] Package Windows
- [ ] Document signing, notarization, checksums, and release upload process

### Quality Gate

- [x] `just check` green
- [x] `just test` green
- [x] `just lint` clean
- [x] No `unsafe` code
- [ ] No committed secrets or `.env` files

## Next: Cloud Web App

**Goal:** Ship the Svelte web surface through Firebase and connect it to the Rust backend through a typed API boundary.

- [ ] Create or promote `apps/web`
- [ ] Add Firebase project config in repo root
- [ ] Add browser support check for Firefox
- [ ] Add web build recipe
- [ ] Add Firebase deploy recipe
- [ ] Add CI check for web build

## Later: Production Backend

**Goal:** Move from local HTTP harness to production Rust backend with gRPC and durable data.

- [ ] Define protobuf contracts
- [ ] Implement gRPC service
- [ ] Keep HTTP compatibility only if explicitly required by the product
- [ ] Add Cloud Run container build
- [ ] Add Cloud SQL or approved Google Cloud database integration
- [ ] Add OpenTelemetry traces, metrics, and logs

## Later: Desktop Distribution

**Goal:** Provide signed downloadable desktop builds for macOS and Windows.

- [ ] Configure Tauri bundle metadata
- [ ] Add release workflow for macOS universal or separate silicon/intel artifacts
- [ ] Add release workflow for Windows
- [ ] Publish checksums
- [ ] Document download verification
