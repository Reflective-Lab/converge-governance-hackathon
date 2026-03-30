default:
    @just --list

test:
    cargo test --workspace

server:
    cargo run -p governance-server

fmt:
    cargo fmt --all

lint:
    cargo clippy --workspace

check:
    cargo check --workspace
