# Contributing

## Getting Started

1. Use GitHub's "Use this template" button to create your team's repo
2. Clone your team repo and create feature branches
3. Run `just hit-the-ground-running` to verify everything builds
4. Make your changes
5. Run `just lint` before committing
6. Submit a pull request to your team's `main` branch

## Development

- Rust edition 2024, minimum rust-version 1.94
- `unsafe` code is forbidden
- Clippy warnings are enforced

## Pull Requests

- Keep PRs focused — one logical change per PR
- Write clear commit messages
- Ensure `just check && just test && just lint` passes

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
