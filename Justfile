default:
    @just --list

# First time setup: build, test, and verify everything works
hit-the-ground-running:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "==> Checking toolchain"
    rustc --version
    cargo --version
    echo "==> Building workspace"
    cargo build --workspace
    echo "==> Running tests"
    cargo test --workspace
    echo "==> Lint check"
    cargo clippy --workspace
    echo "==> Ready. Run 'just server' to start the local harness."

# Delete all build artifacts and start fresh
clean:
    cargo clean
    rm -rf apps/desktop/node_modules apps/desktop/dist apps/desktop/src-tauri/target

test:
    cargo test --workspace

build:
    cargo build --workspace

build-server:
    cargo build -p governance-server

install-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    bun --cwd apps/desktop install

desktop:
    cd apps/desktop && bun run tauri dev; reset

dev-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    desktop_dir="apps/desktop"
    if [[ ! -f "$desktop_dir/package.json" ]]; then
      echo "Missing $desktop_dir/package.json. Scaffold the Svelte app before running just dev-desktop."
      exit 1
    fi
    if [[ ! -f "$desktop_dir/src-tauri/tauri.conf.json" && ! -f "$desktop_dir/src-tauri/tauri.conf.json5" ]]; then
      echo "Missing $desktop_dir/src-tauri/tauri.conf.json. Scaffold the Tauri shell before running just dev-desktop."
      exit 1
    fi
    cd "$desktop_dir"
    bun run tauri dev

build-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    desktop_dir="apps/desktop"
    if [[ ! -f "$desktop_dir/package.json" ]]; then
      echo "Missing $desktop_dir/package.json. Scaffold the Svelte app before running just build-desktop."
      exit 1
    fi
    if [[ ! -f "$desktop_dir/src-tauri/tauri.conf.json" && ! -f "$desktop_dir/src-tauri/tauri.conf.json5" ]]; then
      echo "Missing $desktop_dir/src-tauri/tauri.conf.json. Scaffold the Tauri shell before running just build-desktop."
      exit 1
    fi
    bun --cwd "$desktop_dir" run build

package-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    desktop_dir="apps/desktop"
    if [[ ! -f "$desktop_dir/package.json" ]]; then
      echo "Missing $desktop_dir/package.json. Scaffold the Svelte app before running just package-desktop."
      exit 1
    fi
    if [[ ! -f "$desktop_dir/src-tauri/tauri.conf.json" && ! -f "$desktop_dir/src-tauri/tauri.conf.json5" ]]; then
      echo "Missing $desktop_dir/src-tauri/tauri.conf.json. Scaffold the Tauri shell before running just package-desktop."
      exit 1
    fi
    cd "$desktop_dir"
    bun run tauri build

deploy:
    @just package-desktop

server:
    cargo run -p governance-server

fmt:
    cargo fmt --all

lint:
    cargo clippy --workspace

check:
    cargo check --workspace

focus:
    scripts/workflow/focus.sh

sync:
    scripts/workflow/sync.sh

status:
    scripts/workflow/status.sh
