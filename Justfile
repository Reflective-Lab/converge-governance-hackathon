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

live-test:
    cargo test --features live-tests -p governance-server -- live_ --nocapture

# Run model competition (13 models × 4 roles = 52 runs, ~15-30 min)
competition:
    cargo run -p governance-server --bin model-competition --release

# Run the headless vendor-selection flow demo
demo-flow *ARGS:
    @cargo run -q -p governance-server --bin vendor-selection-demo -- {{ARGS}}

demo-flow-governed:
    @cargo run -q -p governance-server --bin vendor-selection-demo -- --mode=governed

demo-flow-breakout:
    @cargo run -q -p governance-server --bin vendor-selection-demo -- --mode=pareto-breakout

# Run the full Helm AI vendor-selection demo (mock by default; pass -l/--live for real Providers, -v/--verbose for diagnostics)
demo *ARGS:
    @./scripts/demo/presentation.sh {{ARGS}}

# Run the governed Helm AI vendor-selection track with live Providers
demo-live *ARGS:
    @./scripts/demo/today.sh --live {{ARGS}}

# Run the governed Helm AI vendor-selection track with mock Providers
demo-today *ARGS:
    @./scripts/demo/today.sh {{ARGS}}

# Run the governed Helm AI vendor-selection track with live Providers
demo-today-live *ARGS:
    @./scripts/demo/today.sh --live {{ARGS}}

# Run the creative Helm AI vendor-selection Pareto track with mock Providers
demo-creative *ARGS:
    @./scripts/demo/creative.sh {{ARGS}}

# Run the creative Helm AI vendor-selection Pareto track with live Providers
demo-creative-live *ARGS:
    @./scripts/demo/creative.sh --live {{ARGS}}

# Run a single presentation step (1-7)
demo-step STEP:
    @./scripts/demo/presentation.sh step {{STEP}}

# Verify the expected business story for the offline presentation demo
demo-verify:
    @./scripts/demo/verify.sh

# Run governed selection with real AI vendor names
demo-ai-vendors *ARGS:
    @cargo run -q -p governance-server --bin vendor-selection-demo -- --vendors-json=examples/vendor-selection/demo-ai-vendors.json {{ARGS}}

# Run with competition-validated models (8 vendors from the 21-model competition)
demo-competition *ARGS:
    @cargo run -q -p governance-server --bin vendor-selection-demo -- --vendors-json=examples/vendor-selection/demo-competition-vendors.json {{ARGS}}

# Run the two executive strategy candidates
demo-strategy-candidates *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    experience_path="${TMPDIR:-/tmp}/governance-strategy-candidates-experience.json"
    rm -f "$experience_path"
    cargo run -q -p governance-server --bin vendor-selection-demo -- \
        --vendors-json=examples/vendor-selection/demo-ai-strategy-candidates.json \
        --experience-path="$experience_path" \
        {{ARGS}}

# Competition vendors: governed then pareto-breakout side by side
demo-competition-both:
    @echo "=== GOVERNED ===" && cargo run -q -p governance-server --bin vendor-selection-demo -- --vendors-json=examples/vendor-selection/demo-competition-vendors.json --mode=governed
    @echo "" && echo "=== PARETO BREAKOUT ===" && cargo run -q -p governance-server --bin vendor-selection-demo -- --vendors-json=examples/vendor-selection/demo-competition-vendors.json --mode=pareto-breakout

build:
    cargo build --workspace

build-server:
    cargo build -p governance-server

install-desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    cd apps/desktop && bun install

desktop: install-desktop
    cd apps/desktop && bun run tauri dev

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
