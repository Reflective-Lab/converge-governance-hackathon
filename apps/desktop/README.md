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

Remote calls should originate from the Rust core, not the UI. They may go through Kong or direct provider and service adapters for now.

For student-facing code, the desktop app should stay on the canonical Converge capability surface, not a separate gateway-specific contract:

1. Load `.env` in the Tauri layer.
2. Build or inject the chat backend at the app edge.
3. Use `ChatRequest` / `ChatResponse` in app code and suggestors.
4. Keep any Kong-specific routing below that boundary.
5. Use the same typed surface for live backends, Kong-backed routing, direct providers, mocks, and offline validators.

If using Kong:

```dotenv
KONG_AI_GATEWAY_URL=https://<provided-at-hackathon>
KONG_API_KEY=<your-team-key>
```

Direct provider keys are also acceptable during the current transition, as long as the app stays on the same typed capability surface.

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

The current Tauri command validates specs offline-first using `converge-tool::gherkin::GherkinValidator`. Business-sense and compilability checks are intentionally disabled until a fuller live validator is added on top of the same `ChatBackend` contract.

Current status: `apps/desktop/src-tauri` is on the canonical template: the Tauri layer selects a live `ChatBackend`, passes `ChatRequest` into the app logic, and uses `StaticChatBackend` for offline fallback. A future `KongProvider` or `RouterProvider` should sit under that same surface rather than replace it. See [Programming API Surfaces](../../kb/Development/Programming%20API%20Surfaces.md).
