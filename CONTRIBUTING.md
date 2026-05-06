# Contributing

## Getting Started

1. Clone the repo.
2. Read [AGENTS.md](AGENTS.md) and [MILESTONES.md](MILESTONES.md).
3. Run `just setup` on a fresh machine.
4. Work on the current train. Use `main` locally, or `release/<version>` when a release branch exists.
5. Run the relevant checks before opening a pull request.

## Git Workflow

- Do not use git worktrees.
- Do not create feature, topic, fix, docs, chore, or spike branches for normal work.
- Use `main` as the single train.
- Use `release/<version>` branches only for release stabilization.
- Never push to `main` without explicit confirmation.

## Development Rules

- Rust edition 2024, minimum rust-version 1.94.
- Bun is the JavaScript package manager.
- Svelte/SvelteKit for web UI.
- Tauri 2 + Svelte for desktop.
- No React.
- No `unsafe` code.
- Use typed enums for semantic state.
- Cloud infrastructure must be Terraform-managed.
- Firebase config belongs at the repo root when the web app is added.
- `vendor-selection` is the only product truth; extend it instead of adding new product truths.

## Verification

Run the smallest relevant checks while developing, then run the full gate before review:

```bash
just check
just test
just lint
```

For desktop packaging changes:

```bash
just package-desktop
```

## Pull Requests

- Keep PRs focused on the current train or release branch.
- Update documentation when setup, behavior, deployment, policy, or data handling changes.
- Include tests for behavior changes.
- Do not commit secrets, `.env` files, credentials, build artifacts, or generated targets.
- Use the GitHub pull request template and note any checks that were not run.

## Security

See [SECURITY.md](SECURITY.md). Do not open public issues for vulnerabilities.

## License

By contributing, you agree that your contributions are licensed under the MIT License.
