#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cargo test --locked --manifest-path "$ROOT/Cargo.toml"
cargo test --locked --examples --manifest-path "$ROOT/Cargo.toml"
cargo clippy --locked --all-targets --all-features --manifest-path "$ROOT/Cargo.toml" -- -D warnings
cargo doc --locked --no-deps --manifest-path "$ROOT/Cargo.toml"
cargo check --locked --benches --manifest-path "$ROOT/Cargo.toml"
