# Desktop App Direction

`apps/desktop/` is the home of the self-contained operator application.

The intended split is:

- `src/` for the Svelte UI
- `src-tauri/` for the Tauri shell
- `crates/governance-app` as the Rust application core the shell calls locally

The first end-to-end desktop flow is vendor-selection spec validation:

1. Paste or load Gherkin into one large editor.
2. Validate it locally with `converge-tool`.
3. Show Converge syntax, governance, and convention findings.
4. Expand this into full truth execution after validation is clean.

The only intended remote calls are outbound calls from the Rust core to Kong and the LLM or business services it fronts.

For Kong integration, the desktop app should use `converge-provider`, not raw gateway HTTP calls:

1. Load `.env` in the Tauri layer.
2. Build `KongGateway::from_env()`.
3. Define a `KongRoute` for the editor LLM use case.
4. Call `gateway.llm_provider(route)` for guided validation or rewrite flows.
5. Use `gateway.mcp_url(...)` or `gateway.api_url(...)` for Kong-routed tools and services.

Required `.env`:

```dotenv
KONG_AI_GATEWAY_URL=https://<provided-at-hackathon>
KONG_API_KEY=<your-team-key>
```

Optional desktop LLM settings:

```dotenv
KONG_LLM_ROUTE=default
KONG_LLM_UPSTREAM_PROVIDER=openai
KONG_LLM_UPSTREAM_MODEL=gpt-4
KONG_LLM_REASONING=true
```

The desktop toolchain uses Bun:

- `just install-desktop`
- `just dev-desktop`
- `just build-desktop`
- `just package-desktop` to build a native bundle for your platform
- `just deploy` as the default packaging path

The current Tauri command validates specs offline-first using `converge-tool::gherkin::GherkinValidator`. Business-sense and compilability checks are intentionally disabled until a Kong-backed LLM validator is wired in.
