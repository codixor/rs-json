#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cargo test -p rs-json@0.1.0 --manifest-path "$ROOT/Cargo.toml"
cargo test -p rs-json@0.1.0 --examples --manifest-path "$ROOT/Cargo.toml"
cargo clippy -p rs-json@0.1.0 --all-targets --all-features --manifest-path "$ROOT/Cargo.toml" -- -D warnings
cargo doc -p rs-json@0.1.0 --no-deps --manifest-path "$ROOT/Cargo.toml"
cargo check -p rs-json@0.1.0 --benches --manifest-path "$ROOT/Cargo.toml"
