default:
    @just --list

# Install all dependencies and verify toolchain (idempotent, safe to re-run)
setup:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "==> Setting up Converge Governance Hackathon..."
    echo ""

    # Check/install Rust
    if ! command -v rustc &> /dev/null; then
        echo "✓ Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    rustc_version=$(rustc --version | awk '{print $2}')
    echo "✓ Rust $rustc_version"

    # Verify Rust 1.94+
    required_version="1.94.0"
    if ! printf '%s\n' "$required_version" "$rustc_version" | sort -V | head -n1 | grep -q "^$required_version"; then
        echo "⚠ Rust $rustc_version < $required_version; updating..."
        rustup update
    fi

    # Check/install just
    if ! command -v just &> /dev/null; then
        echo "✓ Installing just task runner..."
        cargo install just --quiet
    fi
    echo "✓ just $(just --version | cut -d' ' -f2)"

    # Check/install bun
    if ! command -v bun &> /dev/null; then
        echo "✓ Installing Bun package manager..."
        curl -fsSL https://bun.sh/install | bash
        export PATH="$HOME/.bun/bin:$PATH"
    fi
    echo "✓ Bun $(bun --version)"

    # Check system dependencies
    echo "✓ Checking system dependencies..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if ! command -v xcode-select &> /dev/null || ! xcode-select -p &> /dev/null; then
            echo "  Installing Xcode command line tools (may take 5-10 minutes)..."
            xcode-select --install || true
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        if ! dpkg -l | grep -q libwebkit2gtk; then
            echo "  Installing libwebkit2gtk-4.1-dev..."
            sudo apt-get update
            sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential libssl-dev
        fi
    fi

    echo ""
    echo "✓ Building workspace (this may take 2-3 minutes on first run)..."
    cargo build --workspace --quiet

    echo "✓ Running tests..."
    cargo test --workspace --quiet 2>&1 | tail -3

    echo "✓ Lint check..."
    cargo clippy --workspace --quiet 2>&1 || true

    echo ""
    echo "✓ Setting up desktop dependencies..."
    cd apps/desktop
    bun install --quiet
    cd - > /dev/null

    echo ""
    echo "=========================================="
    echo "✓ Ready! Run: just dev"
    echo "=========================================="
    echo ""
    echo "Or try these to explore:"
    echo "  just server          — Start the governance server alone"
    echo "  just desktop         — Launch desktop app (needs server running separately)"
    echo "  just seed            — Populate demo data (needs server running)"
    echo "  just demo-today      — Run headless vendor-selection demo"

# First time setup: build, test, and verify everything works (legacy alias)
hit-the-ground-running: setup

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

# Start server and desktop app together (best experience for participants)
dev:
    #!/usr/bin/env bash
    set -euo pipefail

    # Verify port 8080 is available
    if lsof -i :8080 &> /dev/null; then
        echo "✗ Port 8080 already in use. Kill the process with:"
        echo "  lsof -i :8080 | grep LISTEN | awk '{print \$2}' | xargs kill -9"
        exit 1
    fi

    # Verify desktop dependencies
    desktop_dir="apps/desktop"
    if [[ ! -d "$desktop_dir/node_modules" ]]; then
        echo "✓ Installing desktop dependencies..."
        cd "$desktop_dir"
        bun install --quiet
        cd - > /dev/null
    fi

    echo "✓ Starting governance server..."
    cargo run -p governance-server --bin governance-server 2>&1 &
    server_pid=$!

    # Wait for server to be healthy
    echo "⏳ Waiting for server to be ready..."
    max_attempts=30
    attempt=0
    while [[ $attempt -lt $max_attempts ]]; do
        if curl -fs http://127.0.0.1:8080/health > /dev/null 2>&1; then
            echo "✓ Server ready at http://127.0.0.1:8080"
            break
        fi
        attempt=$((attempt + 1))
        sleep 0.5
    done

    if [[ $attempt -eq $max_attempts ]]; then
        echo "✗ Server failed to start. Last 10 lines of output:"
        kill "$server_pid" 2>/dev/null || true
        exit 1
    fi

    # Clean exit on interrupt
    trap 'kill "$server_pid" 2>/dev/null || true; exit 0' EXIT INT TERM

    echo "✓ Launching desktop app..."
    cd "$desktop_dir"
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

# Populate demo data (vendor scenarios). Server must be running at localhost:8080
seed:
    #!/usr/bin/env bash
    set -euo pipefail

    # Check server is alive
    if ! curl -fs http://127.0.0.1:8080/health > /dev/null 2>&1; then
        echo "✗ Governance server not running at localhost:8080"
        echo "Start it with: just server"
        exit 1
    fi

    # Load vendors from seed file
    seed_file="examples/vendor-selection/seed-vendors.json"
    if [[ ! -f "$seed_file" ]]; then
        echo "✗ Seed file not found: $seed_file"
        exit 1
    fi

    VENDORS=$(cat "$seed_file")
    vendor_count=$(echo "$VENDORS" | jq 'length')

    echo "✓ Seeding $vendor_count vendor scenarios..."
    result=$(curl -fsS -X POST http://127.0.0.1:8080/v1/truths/vendor-selection/execute \
      -H "Content-Type: application/json" \
      -d "{\"inputs\":{\"vendors_json\":$(echo "$VENDORS" | jq -c '.' | jq -Rs .),\"min_score\":\"75\",\"max_risk\":\"30\"},\"persist_projection\":true}")

    if echo "$result" | jq -e '.decision_record' > /dev/null 2>&1; then
        echo "✓ Demo data loaded successfully"
        echo ""
        echo "Next steps:"
        echo "  1. Open the desktop app (just dev)"
        echo "  2. Explore the vendor evaluation in the governance console"
        echo "  3. Try modifying the policy in examples/vendor-selection/vendor-selection-policy.cedar"
    else
        echo "✗ Seeding failed:"
        echo "$result" | jq .
        exit 1
    fi

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
