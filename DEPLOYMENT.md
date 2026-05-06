# Deployment

This document captures the intended deployment shape. The current repo still contains a local backend harness and a Tauri desktop app scaffold; production cloud wiring is part of the current milestone.

## Targets

| Target | Status | Direction |
|---|---:|---|
| Web | Planned | Svelte/SvelteKit app deployed with Firebase Hosting |
| Backend | Planned | Rust gRPC service on Google Cloud Run |
| Database | Planned | Monitored Google Cloud database with durable projections |
| Desktop | Current scaffold | Tauri packages for macOS Apple silicon, macOS Intel, and Windows |

## Web

The web app should follow the Firebase Hosting pattern used by neighboring apps such as `../wolfgang`:

- Firebase config at repo root: `firebase.json` and `.firebaserc`
- Build output ignored from git
- SPA rewrites to `index.html`
- Long cache headers for hashed JS/CSS assets
- No-cache headers for `index.html`

Firefox is a supported browser target for the web app.

## Backend

The local HTTP harness stays useful for development, but production backend work should move toward:

- protobuf contracts
- gRPC service implementation in Rust
- Cloud Run container
- health and readiness endpoints
- structured logs, metrics, and traces
- secrets from Google Secret Manager

## Database

The database must be monitored and provisioned through Terraform. Required data surfaces:

- vendors
- selection criteria
- evidence and source provenance
- proposed facts
- promoted facts
- decisions
- audit entries
- run telemetry

## Infrastructure

Cloud resources must be defined in Terraform. Use `gcloud` only for checks and operational fixes.

Expected infrastructure modules:

- Firebase Hosting
- Cloud Run backend
- database
- Secret Manager
- service accounts and IAM
- monitoring dashboards and alerts
