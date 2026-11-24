#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cargo bench --locked --bench json_ld --manifest-path "$ROOT/Cargo.toml" -- --sample-size 10 "$@"
