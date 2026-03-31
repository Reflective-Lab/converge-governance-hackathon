default:
    @just --list

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
    bun run tauri build --bundles app

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
