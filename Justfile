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

setup: hit-the-ground-running

# Delete all build artifacts and start fresh
clean:
    cargo clean
    rm -rf apps/desktop/node_modules apps/desktop/dist apps/desktop/src-tauri/target

test:
    cargo test --workspace

# Run tests with coverage report (requires cargo-llvm-cov: cargo install cargo-llvm-cov)
test-coverage:
    cargo llvm-cov --workspace --html
    @echo "Coverage report: target/llvm-cov/html/index.html"

build:
    cargo build --workspace

build-server:
    cargo build -p governance-server

install-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    cd apps/desktop && bun install

desktop: install-desktop
    cd apps/desktop && bun run tauri dev; reset

dev:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo run -p governance-server --bin governance-server &
    server_pid=$!
    trap 'kill "$server_pid" 2>/dev/null || true' EXIT
    sleep 2
    cd apps/desktop
    bun run tauri dev

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
    cargo run -p governance-server --bin governance-server

# Seed vendors into a running server from examples/vendor-selection/seed-vendors.json
seed:
    #!/usr/bin/env bash
    set -euo pipefail
    VENDORS=$(cat examples/vendor-selection/seed-vendors.json)
    echo "Seeding vendor-selection truth with $(echo "$VENDORS" | jq length) vendors..."
    curl -fsS -X POST http://127.0.0.1:8080/v1/truths/vendor-selection/execute \
      -H "Content-Type: application/json" \
      -d "{\"inputs\":{\"vendors_json\":$(echo "$VENDORS" | jq -c '.' | jq -Rs .),\"min_score\":\"75\",\"max_risk\":\"30\"},\"persist_projection\":true}" | jq .

# Evaluate specific vendors by name (comma-separated)
eval-vendor NAMES:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Evaluating vendors: {{NAMES}}"
    curl -fsS -X POST http://127.0.0.1:8080/v1/truths/vendor-selection/execute \
      -H "Content-Type: application/json" \
      -d "{\"inputs\":{\"vendors\":\"{{NAMES}}\",\"min_score\":\"75\",\"max_risk\":\"30\"},\"persist_projection\":true}" | jq .

# Check Cedar policy evaluation for vendor selection
policy-check:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running vendor-selection with policy gate..."
    VENDORS=$(cat examples/vendor-selection/seed-vendors.json)
    curl -fsS -X POST http://127.0.0.1:8080/v1/truths/vendor-selection/execute \
      -H "Content-Type: application/json" \
      -d "{\"inputs\":{\"vendors_json\":$(echo "$VENDORS" | jq -c '.' | jq -Rs .),\"min_score\":\"60\",\"max_risk\":\"40\"},\"persist_projection\":false}" | jq '.criteria_outcomes'

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
