# Release

## Desktop

The desktop app lives in `apps/desktop` and uses Tauri 2 + Svelte.

Local package command:

```bash
just package-desktop
```

Release targets:

- macOS Apple silicon
- macOS Intel
- Windows

Before publishing downloads:

- Confirm bundle identifier, app name, icons, and version.
- Build on each target platform or through trusted CI runners.
- Sign macOS and Windows artifacts.
- Notarize macOS artifacts.
- Generate checksums.
- Attach release notes and verification instructions.

## Web

The web release path is planned:

- build Svelte/SvelteKit web app
- deploy static output through Firebase Hosting
- verify Firefox support
- verify backend API compatibility

## Backend

The backend release path is planned:

- build container
- deploy Rust gRPC service to Google Cloud Run
- apply database migrations
- verify health, metrics, logs, and alerts
- run smoke tests against the deployed API
